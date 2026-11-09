use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use indicatif::ProgressBar;
use regex::Regex;

use crate::models::{
    diff_result::DiffResult,
    installed_packages::{InstalledPackages, UpgradeInfo},
    manager_validation::ValidationResult,
    ntix_config::NTIXConfig,
    options::ScoopBucket,
    package_entry::PackageEntry,
    package_manager::PackageManager,
    package_spec::PackageSpec,
    state::State,
};
use crate::package_manager::{
    command_builder, command_runner::CommandRunner, package_manager_detector, probes::Probes,
    process_command_runner::ProcessCommandRunner,
};

/// Everything `compute_diff` needs about one manager, assembled once so the
/// per-manager passes do not each redo the same lookups.
struct ManagerView<'a> {
    manager: PackageManager,
    configured: &'a [PackageEntry],
    installed: &'a HashMap<String, String>,
    tracked: &'a HashMap<String, String>,
    /// Detected on the system (gates orphan removal).
    detected: bool,
    /// Enabled in options and detected (gates install/upgrade classification).
    enabled: bool,
}

fn manager_views<'a>(
    config: &'a NTIXConfig,
    validation: &ValidationResult,
    installed: &'a InstalledPackages,
    state: &'a State,
) -> [ManagerView<'a>; 3] {
    PackageManager::ALL.map(|manager| {
        let detected = validation.installed(manager);
        ManagerView {
            manager,
            configured: config.packages(manager),
            installed: installed.packages(manager),
            tracked: state.packages(manager),
            detected,
            enabled: config.options.enabled(manager) && detected,
        }
    })
}

pub async fn compute_diff(
    config: &NTIXConfig,
    state: &State,
    probes: &Probes<'_>,
    adopt_mode: bool,
    upgrade_mode: bool,
    progress: &ProgressBar,
) -> Result<DiffResult, Box<dyn Error>> {
    progress.set_message("Checking package managers...");
    let validation =
        package_manager_detector::validate_managers(config, probes.runner, probes.presence).await;

    let mut result = DiffResult {
        warnings: validation.warnings.clone(),
        manager_validation: validation.clone(),
        ..Default::default()
    };

    let installed_pkgs = match probes.installed {
        Some(installed) => installed.clone(),
        None => {
            progress.set_message("Discovering installed packages...");
            package_manager_detector::get_installed_packages(probes.runner).await
        }
    };

    let views = manager_views(config, &validation, &installed_pkgs, state);

    progress.set_message("Checking for updates...");
    for view in &views {
        let upgradable = if upgrade_mode
            && view.enabled
            && view.configured.iter().any(|p| p.version.is_none())
        {
            package_manager_detector::get_upgradable_packages(view.manager, probes.runner).await
        } else {
            HashMap::new()
        };

        classify_packages(&mut result, view, &upgradable, adopt_mode);
    }

    if !probes.skip_package_validation {
        progress.set_message("Validating packages...");
        validate_package_availability(&mut result, &views, probes.runner).await;
    }

    progress.set_message("Finding orphans...");
    for view in &views {
        if view.detected {
            find_orphans(&mut result, view);
        }
    }

    if config.options.scoop.enable
        && validation.scoop_installed
        && !config.options.scoop.buckets.is_empty()
    {
        progress.set_message("Checking scoop buckets...");
        compute_bucket_diff(
            &mut result,
            &config.options.scoop.buckets,
            &state.scoop_buckets,
            probes.runner,
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

    let re = Regex::new(r"^(\S+)").unwrap();

    for line in cmd
        .run_output(&command_builder::build_scoop_bucket_list(), false)
        .await
        .lines()
    {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('-') {
            continue;
        }

        if let Some(name) = re
            .captures(trimmed)
            .and_then(|c| c.get(1))
            .filter(|name| !name.as_str().eq_ignore_ascii_case("Name"))
        {
            system_buckets.insert(name.as_str().to_lowercase());
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

/// Classifies every configured package of one manager into the diff lists.
fn classify_packages(
    result: &mut DiffResult,
    view: &ManagerView<'_>,
    upgradable: &HashMap<String, UpgradeInfo>,
    adopt_mode: bool,
) {
    if !view.enabled {
        return;
    }

    for pkg in view.configured {
        let mut spec = PackageSpec {
            id: pkg.id.clone(),
            version: pkg.version.clone(),
            source: view.manager,
        };
        let installed_version = ci_lookup(view.installed, &pkg.id);
        let state_version = ci_lookup(view.tracked, &pkg.id);
        let is_installed = installed_version.is_some();
        let in_state = state_version.is_some();

        if let Some(pkg_version) = &pkg.version {
            if let Some(state_version) = state_version {
                if !state_version.eq_ignore_ascii_case(pkg_version) {
                    result.to_install.push(spec);
                } else if is_installed {
                    result.to_skip.push(spec);
                } else {
                    result.to_install.push(spec);
                }
            } else if is_installed && adopt_mode {
                let installed_version = installed_version.expect("is_installed implies an entry");
                if installed_version.eq_ignore_ascii_case(pkg_version) {
                    spec.version = Some(installed_version.clone());
                    result.to_adopt.push(spec);
                } else {
                    result.to_install.push(spec);
                }
            } else {
                result.to_install.push(spec);
            }
        } else {
            // Any installed version satisfies an unpinned entry.
            if is_installed {
                if let Some(upgrade) = ci_lookup(upgradable, &pkg.id) {
                    spec.version = Some(upgrade.available_version.clone());
                    result.to_upgrade.push(spec);
                } else if in_state {
                    result.to_skip.push(spec);
                } else if adopt_mode {
                    result.to_adopt.push(spec);
                } else {
                    // Installed on the system but not tracked: report as
                    // unmanaged rather than counting it as "already managed".
                    result.to_untracked.push(spec);
                }
            } else {
                result.to_install.push(spec);
            }
        }
    }
}

fn find_orphans(result: &mut DiffResult, view: &ManagerView<'_>) {
    for (id, ver) in view.tracked {
        if !view
            .configured
            .iter()
            .any(|p| p.id.eq_ignore_ascii_case(id))
        {
            result.to_remove.push(PackageSpec {
                id: id.to_string(),
                version: Some(ver.to_string()),
                source: view.manager,
            });
        }
    }
}

/// Verifies only packages that are new to state (already-tracked entries are
/// known to exist), querying all managers concurrently.
async fn validate_package_availability(
    result: &mut DiffResult,
    views: &[ManagerView<'_>],
    runner: Option<&dyn CommandRunner>,
) {
    let checks: Vec<(PackageManager, Vec<String>)> = views
        .iter()
        .filter_map(|view| {
            let new_ids: Vec<String> = result
                .to_install
                .iter()
                .filter(|p| p.source == view.manager)
                .filter(|p| ci_lookup(view.tracked, &p.id).is_none())
                .map(|p| p.id.clone())
                .collect();
            (!new_ids.is_empty()).then_some((view.manager, new_ids))
        })
        .collect();

    let outcomes = futures::future::join_all(checks.iter().map(|(manager, ids)| async move {
        (
            *manager,
            package_manager_detector::validate_packages_exist(*manager, ids, runner).await,
        )
    }))
    .await;

    for ((manager, ids), (_, manager_outcomes)) in checks.iter().zip(outcomes) {
        let mut invalid_ids: Vec<String> = Vec::new();
        for id in ids {
            match manager_outcomes.get(id) {
                Some(Some(false)) => {
                    result
                        .warnings
                        .push(format!("Package not found in {manager}: {id}"));
                    invalid_ids.push(id.clone());
                }
                Some(None) | None => {
                    result
                        .warnings
                        .push(format!("Could not verify package in {manager}: {id}"));
                }
                Some(Some(true)) => {}
            }
        }

        if !invalid_ids.is_empty() {
            result
                .to_install
                .retain(|p| !(p.source == *manager && invalid_ids.contains(&p.id)));
        }
    }
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
