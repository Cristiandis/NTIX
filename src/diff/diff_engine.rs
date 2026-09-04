use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use indicatif::ProgressBar;
use regex::Regex;

use crate::{
    models::{
        diff_result::DiffResult, installed_packages::InstalledPackages,
        installed_packages::UpgradeInfo, ntix_config::NTIXConfig, options::ScoopBucket,
        package_entry::PackageEntry, package_spec::PackageSpec, state::State,
    },
    package_manager::{
        command_builder, command_runner::CommandRunner, package_manager_detector,
        process_command_runner::ProcessCommandRunner, winget_manager_trait::WingetManagerTrait,
    },
};

#[allow(clippy::too_many_arguments)]
pub async fn compute_diff(
    config: &NTIXConfig,
    state: &State,
    winget_manager: Option<&dyn WingetManagerTrait>,
    choco_installed: Option<bool>,
    scoop_installed: Option<bool>,
    runner: Option<&dyn CommandRunner>,
    adopt_mode: bool,
    upgrade_mode: bool,
    validate_packages: bool,
    installed: Option<&InstalledPackages>,
    progress: &ProgressBar,
) -> Result<DiffResult, Box<dyn Error>> {
    progress.set_message("Checking package managers...");
    let validation = package_manager_detector::validate_managers_async(
        &config.options,
        config,
        winget_manager,
        choco_installed,
        scoop_installed,
    )
    .await;

    let mut result = DiffResult {
        warnings: validation.warnings,
        ..Default::default()
    };

    let installed_pkgs = match installed {
        Some(installed) => installed.clone(),
        None => {
            progress.set_message("Discovering installed packages...");
            package_manager_detector::get_installed_packages_async(winget_manager, runner).await
        }
    };

    let has_winget_unpinned = config.winget_packages.iter().any(|p| p.version.is_none());
    let has_choco_unpinned = config.choco_packages.iter().any(|p| p.version.is_none());
    let has_scoop_unpinned = config.scoop_packages.iter().any(|p| p.version.is_none());

    let winget_enabled = config.options.winget.enable && validation.winget_installed;
    let choco_enabled = config.options.chocolatey.enable && validation.choco_installed;
    let scoop_enabled = config.options.scoop.enable && validation.scoop_installed;

    progress.set_message("Checking for updates...");

    let winget_upgradable = if upgrade_mode && has_winget_unpinned && winget_enabled {
        package_manager_detector::get_winget_upgradable_packages_async(winget_manager).await
    } else {
        HashMap::new()
    };
    let choco_upgradable = if upgrade_mode && has_choco_unpinned && choco_enabled {
        package_manager_detector::get_choco_upgradable_packages_async(runner).await
    } else {
        HashMap::new()
    };
    let scoop_upgradable = if upgrade_mode && has_scoop_unpinned && scoop_enabled {
        package_manager_detector::get_scoop_upgradable_packages_async(runner).await
    } else {
        HashMap::new()
    };

    classify_packages(
        &mut result,
        &config.winget_packages,
        "winget",
        winget_enabled,
        &installed_pkgs.winget,
        &state.winget,
        &winget_upgradable,
        adopt_mode,
    );
    classify_packages(
        &mut result,
        &config.choco_packages,
        "chocolatey",
        choco_enabled,
        &installed_pkgs.chocolatey,
        &state.chocolatey,
        &choco_upgradable,
        adopt_mode,
    );
    classify_packages(
        &mut result,
        &config.scoop_packages,
        "scoop",
        scoop_enabled,
        &installed_pkgs.scoop,
        &state.scoop,
        &scoop_upgradable,
        adopt_mode,
    );

    if validate_packages {
        progress.set_message("Validating packages...");
        validate_package_availability(
            &mut result,
            state,
            winget_manager,
            winget_enabled,
            choco_enabled,
            scoop_enabled,
        )
        .await;
    }

    progress.set_message("Finding orphans...");
    if validation.winget_installed {
        find_orphans(
            &mut result,
            &state.winget,
            &config.winget_packages,
            "winget",
        );
    }
    if validation.choco_installed {
        find_orphans(
            &mut result,
            &state.chocolatey,
            &config.choco_packages,
            "chocolatey",
        );
    }
    if validation.scoop_installed {
        find_orphans(&mut result, &state.scoop, &config.scoop_packages, "scoop");
    }

    if scoop_enabled && !config.options.scoop.buckets.is_empty() {
        progress.set_message("Checking scoop buckets...");
        compute_bucket_diff(
            &mut result,
            &config.options.scoop.buckets,
            &state.scoop_buckets,
            runner,
        )
        .await;
    }

    Ok(result)
}

async fn compute_bucket_diff(
    result: &mut DiffResult,
    configured_buckets: &[ScoopBucket],
    state_buckets: &HashMap<String, Option<String>>,
    runner: Option<&dyn CommandRunner>,
) {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let mut system_buckets: HashSet<String> = HashSet::new();

    let output = cmd
        .run_output(&command_builder::build_scoop_bucket_list(), false)
        .await;

    let re = Regex::new(r"^(\S+)").unwrap();

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('-') {
            continue;
        }

        if let Some(caps) = re.captures(trimmed) {
            let name = &caps[1];

            if !name.eq_ignore_ascii_case("Name") {
                system_buckets.insert(name.to_lowercase());
            }
        }
    }

    let configured_names: HashSet<String> = configured_buckets
        .iter()
        .map(|b| b.name.to_lowercase())
        .collect();

    for bucket in configured_buckets {
        if !system_buckets.contains(&bucket.name.to_lowercase()) {
            result.buckets_to_add.push(bucket.clone());
        }
    }

    for name in state_buckets.keys() {
        if !configured_names.contains(&name.to_lowercase()) {
            result.buckets_to_remove.push(ScoopBucket {
                name: name.to_string(),
                url: None,
            });
        }
    }
}

fn ci_lookup<'a, V>(map: &'a HashMap<String, V>, key: &str) -> Option<&'a V> {
    map.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, v)| v)
}

#[allow(clippy::too_many_arguments)]
fn classify_packages(
    result: &mut DiffResult,
    packages: &[PackageEntry],
    source_name: &str,
    enabled: bool,
    installed_dict: &HashMap<String, String>,
    state_dict: &HashMap<String, String>,
    upgradable: &HashMap<String, UpgradeInfo>,
    adopt_mode: bool,
) {
    if !enabled {
        return;
    }

    for pkg in packages {
        let mut spec = PackageSpec {
            id: pkg.id.clone(),
            version: pkg.version.clone(),
            source: source_name.to_string(),
        };
        let is_installed = ci_lookup(installed_dict, &pkg.id).is_some();
        let in_state = ci_lookup(state_dict, &pkg.id).is_some();

        if let Some(pkg_version) = &pkg.version {
            if in_state {
                let state_version =
                    ci_lookup(state_dict, &pkg.id).expect("in_state implies a state entry");
                if !state_version.eq_ignore_ascii_case(pkg_version) {
                    result.to_install.push(spec);
                } else if is_installed {
                    result.to_skip.push(spec);
                } else {
                    result.to_install.push(spec);
                }
            } else if is_installed && adopt_mode {
                let installed_version = ci_lookup(installed_dict, &pkg.id)
                    .expect("is_installed and adopt_mode imply an installed entry");
                if installed_version.eq_ignore_ascii_case(pkg_version) {
                    spec.version = Some(installed_version.clone());
                    result.to_adopt.push(spec);
                } else {
                    result.to_install.push(spec);
                }
            } else {
                result.to_install.push(spec);
            }
        } else if is_installed && let Some(upgrade) = ci_lookup(upgradable, &pkg.id) {
            spec.version = Some(upgrade.available_version.clone());
            result.to_upgrade.push(spec);
        } else if !is_installed && !in_state {
            result.to_install.push(spec);
        } else if is_installed && in_state {
            result.to_skip.push(spec);
        } else if is_installed && adopt_mode {
            result.to_adopt.push(spec);
        } else if is_installed {
            // Installed on the system but not tracked: report as unmanaged
            // rather than counting it as "already managed".
            result.to_untracked.push(spec);
        } else if in_state {
            result.to_install.push(spec);
        }
    }
}

fn find_orphans(
    result: &mut DiffResult,
    state_dict: &HashMap<String, String>,
    config_packages: &[PackageEntry],
    source_name: &str,
) {
    for (id, ver) in state_dict {
        if !config_packages
            .iter()
            .any(|p| p.id.eq_ignore_ascii_case(id))
        {
            result.to_remove.push(PackageSpec {
                id: id.to_string(),
                version: Some(ver.to_string()),
                source: source_name.to_string(),
            });
        }
    }
}

async fn validate_package_availability(
    result: &mut DiffResult,
    state: &State,
    winget_manager: Option<&dyn WingetManagerTrait>,
    winget_enabled: bool,
    choco_enabled: bool,
    scoop_enabled: bool,
) {
    let winget_pkgs: Vec<PackageSpec> = extract_pkgs_to_install(result, "winget");
    let choco_pkgs: Vec<PackageSpec> = extract_pkgs_to_install(result, "chocolatey");
    let scoop_pkgs: Vec<PackageSpec> = extract_pkgs_to_install(result, "scoop");

    let new_winget_ids: Vec<String> = new_pkg_ids(&winget_pkgs, &state.winget);
    let new_choco_ids: Vec<String> = new_pkg_ids(&choco_pkgs, &state.chocolatey);
    let new_scoop_ids: Vec<String> = new_pkg_ids(&scoop_pkgs, &state.scoop);

    let (winget_results, choco_results, scoop_results) = tokio::join!(
        async {
            if winget_enabled && !new_winget_ids.is_empty() {
                package_manager_detector::validate_winget_packages_exist_async(
                    &new_winget_ids,
                    winget_manager,
                )
                .await
            } else {
                HashMap::new()
            }
        },
        async {
            if choco_enabled && !new_choco_ids.is_empty() {
                package_manager_detector::validate_choco_packages_exist_async(&new_choco_ids, None)
                    .await
                    .into_iter()
                    .map(|(id, ok)| (id, Some(ok)))
                    .collect()
            } else {
                HashMap::new()
            }
        },
        async {
            if scoop_enabled && !new_scoop_ids.is_empty() {
                package_manager_detector::validate_scoop_packages_exist_async(&new_scoop_ids, None)
                    .await
                    .into_iter()
                    .map(|(id, ok)| (id, Some(ok)))
                    .collect()
            } else {
                HashMap::new()
            }
        },
    );

    let mut invalid: HashSet<String> = HashSet::new();
    for (pkgs, results, source) in [
        (&winget_pkgs, &winget_results, "winget"),
        (&choco_pkgs, &choco_results, "chocolatey"),
        (&scoop_pkgs, &scoop_results, "scoop"),
    ] {
        for pkg in pkgs {
            match results.get(&pkg.id) {
                Some(Some(false)) => {
                    result
                        .warnings
                        .push(format!("Package not found in {source}: {}", pkg.id));
                    invalid.insert(pkg.id.clone());
                }
                Some(None) | None => {
                    result
                        .warnings
                        .push(format!("Could not verify package in {source}: {}", pkg.id));
                }
                Some(Some(true)) => {}
            }
        }
    }

    result.to_install.retain(|p| !invalid.contains(&p.id));
}

fn new_pkg_ids(pkgs: &[PackageSpec], state_dict: &HashMap<String, String>) -> Vec<String> {
    pkgs.iter()
        .filter(|p| ci_lookup(state_dict, &p.id).is_none())
        .map(|p| p.id.clone())
        .collect()
}

fn extract_pkgs_to_install(result: &DiffResult, source: &str) -> Vec<PackageSpec> {
    result
        .to_install
        .iter()
        .filter(|p| p.source == source)
        .cloned()
        .collect()
}

/// Classifies the config-file state for `config` relative to `state`, populating
/// the `config_files_*` lists on `result`. Only called when the caller opted in
/// with `-c`/`--apply-configs`; otherwise config files are ignored entirely.
pub fn compute_config_files_diff(result: &mut DiffResult, config: &NTIXConfig, state: &State) {
    let mut seen_dests: HashSet<String> = HashSet::new();

    for entry in &config.config_files {
        let dest_str = entry.dest.to_string_lossy().to_string();
        seen_dests.insert(dest_str.clone());

        match std::fs::read(&entry.src) {
            Ok(src_bytes) => {
                let src_hash = crate::hash::sha256_hex(&src_bytes);
                match state.config_files.get(&dest_str) {
                    Some(stored) if *stored == src_hash => {}
                    Some(_) => {
                        result.config_files_to_update.push(entry.clone());
                    }
                    None => {
                        result.config_files_to_create.push(entry.clone());
                    }
                }
            }
            Err(_) => {
                result.warnings.push(format!(
                    "Could not read config file source '{}' for '{}'",
                    entry.src.display(),
                    dest_str
                ));
            }
        }
    }

    for dest in state.config_files.keys() {
        if !seen_dests.contains(dest) {
            result.config_files_no_longer_managed.push(dest.clone());
        }
    }
}
