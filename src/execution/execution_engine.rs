use std::path::Path;

use crate::models::{
    diff_result::DiffResult, ntix_config::NTIXConfig, options::NTIXOptions,
    package_spec::PackageSpec, state::State,
};
use crate::package_manager::{
    command_builder,
    command_runner::{CommandRunner, LineCallback},
    package_manager_detector,
    process_command_runner::ProcessCommandRunner,
    winget_manager::WingetManager,
    winget_manager_trait::WingetManagerTrait,
};
use crate::state_management::state_service;

const DEFAULT_MAX_RETRIES: u32 = 3;

async fn run_built_command(
    cmd: &dyn CommandRunner,
    build_result: Result<String, Box<dyn std::error::Error>>,
    on_output: Option<LineCallback<'_>>,
    on_error: Option<LineCallback<'_>>,
) -> bool {
    match build_result {
        Ok(built_cmd) => cmd.run(&built_cmd, on_output, on_error).await == 0,
        Err(e) => {
            if let Some(cb) = on_error {
                cb(&format!("Failed to build command: {e}"));
            }
            false
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn apply_diff(
    diff: &DiffResult,
    options: &NTIXOptions,
    state: &mut State,
    state_path: &Path,
    stop_on_failure: bool,
    winget_manager: Option<&dyn WingetManagerTrait>,
    choco_installed: Option<bool>,
    scoop_installed: Option<bool>,
    config: Option<&NTIXConfig>,
    apply_config: bool,
    on_output: Option<LineCallback<'_>>,
    on_error: Option<LineCallback<'_>>,
    runner: Option<&dyn CommandRunner>,
) -> bool {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    if let Some(config) = config {
        let validation = package_manager_detector::validate_managers(
            options,
            config,
            choco_installed,
            scoop_installed,
        );
        for w in &validation.warnings {
            if !diff.warnings.contains(w)
                && let Some(cb) = on_error
            {
                cb(w);
            }
        }
    }

    let mut all_ok = true;
    let manager: &dyn WingetManagerTrait = winget_manager.unwrap_or(&WingetManager);

    if options.scoop.enable
        && !apply_buckets(cmd, state, state_path, diff, on_output, on_error).await
    {
        all_ok = false;
    }

    for (pkgs, operation) in [
        (&diff.to_install, Operation::Install),
        (&diff.to_upgrade, Operation::Upgrade),
        (&diff.to_remove, Operation::Remove),
    ] {
        for pkg in pkgs {
            if !run_operation(
                cmd, manager, options, state, state_path, operation, pkg, on_output, on_error,
            )
            .await
            {
                all_ok = false;
                if stop_on_failure {
                    return false;
                }
            }
        }
    }

    for pkg in &diff.to_adopt {
        if let Some(cb) = on_output {
            cb(&format!("Adopting {}:{}...", pkg.source, pkg.id));
        }
        update_state(state, pkg, true);
    }

    if !diff.to_adopt.is_empty() {
        save_state_or_warn(state, state_path, on_error);
    }

    if apply_config {
        apply_config_files(
            state,
            &diff.config_files_to_create,
            &diff.config_files_to_update,
            &diff.config_files_no_longer_managed,
            on_output,
            on_error,
        );
        save_state_or_warn(state, state_path, on_error);
    }

    all_ok
}

fn save_state_or_warn(
    state: &State,
    state_path: &Path,
    on_error: Option<LineCallback<'_>>,
) -> bool {
    let ok = match state_service::save_state(state, Some(state_path), DEFAULT_MAX_RETRIES) {
        Ok(true) => true,
        Ok(false) | Err(_) => false,
    };
    if !ok && let Some(cb) = on_error {
        cb("Failed to save state to disk. Changes may not persist across runs.");
    }
    ok
}

/// Copies managed config files from their resolved sources to their destinations
/// and updates the tracked hashes in `state`. Orphaned (no longer managed) files
/// are dropped from tracking without touching the file on disk.
fn apply_config_files(
    state: &mut State,
    to_create: &[crate::models::config_file::ConfigFileEntry],
    to_update: &[crate::models::config_file::ConfigFileEntry],
    no_longer_managed: &[String],
    on_output: Option<LineCallback<'_>>,
    on_error: Option<LineCallback<'_>>,
) {
    for entry in to_create.iter().chain(to_update.iter()) {
        let dest_str = entry.dest.to_string_lossy().to_string();
        if let Some(cb) = on_output {
            cb(&format!("Applying config file: {dest_str}"));
        }

        match std::fs::read(&entry.src) {
            Ok(src_bytes) => {
                if let Err(msg) = ensure_parent_dir(&entry.dest) {
                    if let Some(cb) = on_error {
                        cb(&format!(
                            "Failed to create directory for config file {dest_str}: {msg}"
                        ));
                    }
                    continue;
                }

                match std::fs::write(&entry.dest, &src_bytes) {
                    Ok(()) => {
                        state
                            .config_files
                            .insert(dest_str, crate::hash::sha256_hex(&src_bytes));
                    }
                    Err(e) => {
                        if let Some(cb) = on_error {
                            cb(&format!("Failed to write config file {dest_str}: {e}"));
                        }
                    }
                }
            }
            Err(e) => {
                if let Some(cb) = on_error {
                    cb(&format!(
                        "Failed to read config file source '{}': {e}",
                        entry.src.display()
                    ));
                }
            }
        }
    }

    for dest in no_longer_managed {
        state.config_files.remove(dest);
    }
}

/// Creates the parent directory of `dest` if it exists and is not the
/// filesystem root. Returns an error message on failure.
fn ensure_parent_dir(dest: &std::path::Path) -> Result<(), String> {
    let Some(parent) = dest.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())
}

async fn apply_buckets(
    cmd: &dyn CommandRunner,
    state: &mut State,
    state_path: &Path,
    diff: &DiffResult,
    on_output: Option<LineCallback<'_>>,
    on_error: Option<LineCallback<'_>>,
) -> bool {
    let mut all_ok = true;

    for bucket in &diff.buckets_to_add {
        if let Some(cb) = on_output {
            cb(&format!("Adding scoop bucket: {}...", bucket.name));
        }
        let build_result =
            command_builder::build_scoop_bucket_add(&bucket.name, bucket.url.as_deref());
        let ok = run_built_command(cmd, build_result, None, None).await;
        if !ok {
            if let Some(cb) = on_error {
                cb(&format!("Failed to add scoop bucket: {}", bucket.name));
            }
            all_ok = false;
        } else {
            state
                .scoop_buckets
                .insert(bucket.name.clone(), bucket.url.clone());
            save_state_or_warn(state, state_path, on_error);
        }
    }

    for bucket in &diff.buckets_to_remove {
        if let Some(cb) = on_output {
            cb(&format!("Removing scoop bucket: {}...", bucket.name));
        }
        let build_result = command_builder::build_scoop_bucket_remove(&bucket.name);
        let ok = run_built_command(cmd, build_result, None, None).await;
        if !ok {
            if let Some(cb) = on_error {
                cb(&format!("Failed to remove scoop bucket: {}", bucket.name));
            }
            all_ok = false;
        } else {
            state.scoop_buckets.remove(&bucket.name);
            save_state_or_warn(state, state_path, on_error);
        }
    }

    all_ok
}

#[derive(Clone, Copy)]
enum Operation {
    Install,
    Upgrade,
    Remove,
}

#[allow(clippy::too_many_arguments)]
async fn run_operation(
    cmd: &dyn CommandRunner,
    manager: &dyn WingetManagerTrait,
    options: &NTIXOptions,
    state: &mut State,
    state_path: &Path,
    operation: Operation,
    pkg: &PackageSpec,
    on_output: Option<LineCallback<'_>>,
    on_error: Option<LineCallback<'_>>,
) -> bool {
    let installs = matches!(operation, Operation::Install | Operation::Upgrade);
    if installs && !is_enabled(&pkg.source, options) {
        return true;
    }

    let verb = match operation {
        Operation::Install => "Installing",
        Operation::Upgrade => "Upgrading",
        Operation::Remove => "Removing",
    };

    if let Some(cb) = on_output {
        cb(&format!("{verb} {}:{}...", pkg.source, pkg.id));
    }

    let success = match pkg.source.as_str() {
        "winget" => match operation {
            Operation::Upgrade => {
                manager
                    .upgrade(&pkg.id, options.winget, on_output, on_error)
                    .await
            }
            Operation::Install => {
                manager
                    .install(
                        &pkg.id,
                        pkg.version.as_deref(),
                        options.winget,
                        on_output,
                        on_error,
                    )
                    .await
            }
            Operation::Remove => {
                let build_result = command_builder::build_winget_uninstall(&pkg.id, options.winget);
                run_built_command(cmd, build_result, on_output, on_error).await
            }
        },
        "chocolatey" => {
            let build_result = match operation {
                Operation::Install => command_builder::build_choco_install(
                    &pkg.id,
                    pkg.version.as_deref(),
                    options.chocolatey.clone(),
                ),
                Operation::Upgrade => {
                    command_builder::build_choco_upgrade(&pkg.id, options.chocolatey.clone())
                }
                Operation::Remove => {
                    command_builder::build_choco_uninstall(&pkg.id, options.chocolatey.clone())
                }
            };
            run_built_command(cmd, build_result, on_output, on_error).await
        }
        "scoop" => {
            let build_result = match operation {
                Operation::Install => command_builder::build_scoop_install(
                    &pkg.id,
                    pkg.version.as_deref(),
                    options.scoop.clone(),
                ),
                Operation::Upgrade => {
                    command_builder::build_scoop_upgrade(&pkg.id, options.scoop.clone())
                }
                Operation::Remove => {
                    command_builder::build_scoop_uninstall(&pkg.id, options.scoop.clone())
                }
            };
            run_built_command(cmd, build_result, on_output, on_error).await
        }
        _ => false,
    };

    if success {
        update_state(state, pkg, installs);
        save_state_or_warn(state, state_path, on_error);
        true
    } else {
        if let Some(cb) = on_error {
            cb(&format!(
                "Failed to {} {}:{}",
                verb.to_lowercase(),
                pkg.source,
                pkg.id
            ));
        }
        false
    }
}

fn is_enabled(source: &str, options: &NTIXOptions) -> bool {
    match source {
        "winget" => options.winget.enable,
        "chocolatey" => options.chocolatey.enable,
        "scoop" => options.scoop.enable,
        _ => false,
    }
}

fn update_state(state: &mut State, pkg: &PackageSpec, installed: bool) {
    let dict = match pkg.source.as_str() {
        "winget" => &mut state.winget,
        "chocolatey" => &mut state.chocolatey,
        "scoop" => &mut state.scoop,
        other => panic!("Unknown source: {other}"),
    };

    if installed {
        dict.insert(
            pkg.id.clone(),
            pkg.version.clone().unwrap_or_else(|| "latest".to_string()),
        );
    } else {
        dict.remove(&pkg.id);
    }
}
