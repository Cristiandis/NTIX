use std::path::Path;

use crate::models::{
    config_file::ConfigFileEntry, diff_result::DiffResult, manager_validation::ValidationResult,
    options::NTIXOptions, package_manager::PackageManager, package_spec::PackageSpec, state::State,
};
use crate::package_manager::{
    choco_ops, command_builder,
    command_runner::{CommandRunner, LineCallback},
    process_command_runner::ProcessCommandRunner,
    scoop_ops, winget_ops,
};
use crate::state_management::state_service;

/// Output sinks for an apply run: where progress lines and error lines go.
#[derive(Default)]
pub struct Output<'a> {
    pub line: Option<LineCallback<'a>>,
    pub error: Option<LineCallback<'a>>,
}

async fn run_built_command(
    cmd: &dyn CommandRunner,
    build_result: Result<String, Box<dyn std::error::Error>>,
    output: &Output<'_>,
) -> bool {
    match build_result {
        Ok(built_cmd) => cmd.run(&built_cmd, output.line, output.error).await == 0,
        Err(e) => {
            if let Some(cb) = output.error {
                cb(&format!("Failed to build command: {e}"));
            }
            false
        }
    }
}

/// `Some(true)` when the installed list contains `id`, `Some(false)` when it
/// does not, `None` when the list could not be read.
async fn installed_state(
    cmd: &dyn CommandRunner,
    source: PackageManager,
    id: &str,
) -> Option<bool> {
    let packages = match source {
        PackageManager::Winget => winget_ops::get_installed_packages(cmd).await,
        PackageManager::Chocolatey => choco_ops::get_installed_packages(cmd).await,
        PackageManager::Scoop => scoop_ops::get_installed_packages(cmd).await,
    }
    .ok()?;
    Some(packages.keys().any(|k| k.eq_ignore_ascii_case(id)))
}

#[derive(Clone, Copy)]
enum Operation {
    Install,
    Upgrade,
    Remove,
}

fn build_package_command(
    operation: Operation,
    pkg: &PackageSpec,
    options: &NTIXOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    match pkg.source {
        PackageManager::Winget => match operation {
            Operation::Install => command_builder::build_winget_install(
                &pkg.id,
                pkg.version.as_deref(),
                options.winget,
            ),
            Operation::Upgrade => command_builder::build_winget_upgrade(&pkg.id, options.winget),
            Operation::Remove => command_builder::build_winget_uninstall(&pkg.id, options.winget),
        },
        PackageManager::Chocolatey => match operation {
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
        },
        PackageManager::Scoop => match operation {
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
        },
    }
}

/// Runs the command for one package operation. Managers exit nonzero for
/// "already at requested state", so failure falls back to checking the
/// installed list before the operation counts as failed.
async fn run_package_operation(
    cmd: &dyn CommandRunner,
    operation: Operation,
    pkg: &PackageSpec,
    options: &NTIXOptions,
    output: &Output<'_>,
) -> bool {
    let verb = match operation {
        Operation::Install => "Installing",
        Operation::Upgrade => "Upgrading",
        Operation::Remove => "Removing",
    };
    if let Some(cb) = output.line {
        cb(&format!("{verb} {}:{}...", pkg.source, pkg.id));
    }

    let mut success =
        run_built_command(cmd, build_package_command(operation, pkg, options), output).await;
    if !success {
        let installed = installed_state(cmd, pkg.source, &pkg.id).await;
        success = match operation {
            Operation::Remove => installed == Some(false),
            Operation::Install | Operation::Upgrade => installed == Some(true),
        };
    }
    success
}

#[allow(clippy::too_many_arguments)]
pub async fn apply_diff(
    diff: &DiffResult,
    options: &NTIXOptions,
    state: &mut State,
    state_path: &Path,
    stop_on_failure: bool,
    validation: &ValidationResult,
    apply_config: bool,
    output: Output<'_>,
    runner: Option<&dyn CommandRunner>,
) -> bool {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    // `diff` usually already carries these warnings (it was built from the same
    // validation); only print the ones it does not have, so none appears twice.
    for warning in &validation.warnings {
        if !diff.warnings.contains(warning)
            && let Some(cb) = output.error
        {
            cb(warning);
        }
    }

    let mut all_ok = true;

    if options.scoop.enable && !apply_buckets(cmd, state, state_path, diff, &output).await {
        all_ok = false;
    }

    for (pkgs, operation) in [
        (&diff.to_install, Operation::Install),
        (&diff.to_upgrade, Operation::Upgrade),
        (&diff.to_remove, Operation::Remove),
    ] {
        for pkg in pkgs {
            if !run_operation(
                cmd, options, validation, state, state_path, operation, pkg, &output,
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
        if let Some(cb) = output.line {
            cb(&format!("Adopting {}:{}...", pkg.source, pkg.id));
        }
        update_state(state, pkg, true);
    }

    if !diff.to_adopt.is_empty() {
        save_state_or_warn(state, state_path, output.error);
    }

    if apply_config {
        apply_config_files(
            state,
            &diff.config_files_to_create,
            &diff.config_files_to_update,
            &diff.config_files_no_longer_managed,
            &output,
        );
        save_state_or_warn(state, state_path, output.error);
    }

    all_ok
}

fn save_state_or_warn(state: &State, state_path: &Path, on_error: Option<LineCallback<'_>>) {
    let saved = state_service::save_state_to(state, state_path).unwrap_or(false);
    if !saved && let Some(cb) = on_error {
        cb("Failed to save state to disk. Changes may not persist across runs.");
    }
}

/// Copies managed config files from their resolved sources to their destinations
/// and updates the tracked hashes in `state`. Orphaned (no longer managed) files
/// are dropped from tracking without touching the file on disk.
fn apply_config_files(
    state: &mut State,
    to_create: &[ConfigFileEntry],
    to_update: &[ConfigFileEntry],
    no_longer_managed: &[String],
    output: &Output<'_>,
) {
    for entry in to_create.iter().chain(to_update.iter()) {
        let dest_str = entry.dest.to_string_lossy().to_string();
        if let Some(cb) = output.line {
            cb(&format!("Applying config file: {dest_str}"));
        }

        match std::fs::read(&entry.src) {
            Ok(src_bytes) => {
                if let Err(msg) = ensure_parent_dir(&entry.dest) {
                    if let Some(cb) = output.error {
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
                        if let Some(cb) = output.error {
                            cb(&format!("Failed to write config file {dest_str}: {e}"));
                        }
                    }
                }
            }
            Err(e) => {
                if let Some(cb) = output.error {
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
fn ensure_parent_dir(dest: &Path) -> Result<(), String> {
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
    output: &Output<'_>,
) -> bool {
    #[derive(Clone, Copy)]
    enum BucketOperation {
        Add,
        Remove,
    }

    let mut all_ok = true;
    // Bucket commands stay silent; only progress and failure lines are shown.
    let silent = Output::default();

    let operations = diff
        .buckets_to_add
        .iter()
        .map(|bucket| (bucket, BucketOperation::Add))
        .chain(
            diff.buckets_to_remove
                .iter()
                .map(|bucket| (bucket, BucketOperation::Remove)),
        );

    for (bucket, operation) in operations {
        let (progress_verb, failure_verb) = match operation {
            BucketOperation::Add => ("Adding", "add"),
            BucketOperation::Remove => ("Removing", "remove"),
        };
        if let Some(cb) = output.line {
            cb(&format!("{progress_verb} scoop bucket: {}...", bucket.name));
        }

        let build_result = match operation {
            BucketOperation::Add => {
                command_builder::build_scoop_bucket_add(&bucket.name, bucket.url.as_deref())
            }
            BucketOperation::Remove => command_builder::build_scoop_bucket_remove(&bucket.name),
        };

        if run_built_command(cmd, build_result, &silent).await {
            match operation {
                BucketOperation::Add => {
                    state
                        .scoop_buckets
                        .insert(bucket.name.clone(), bucket.url.clone());
                }
                BucketOperation::Remove => {
                    state.scoop_buckets.remove(&bucket.name);
                }
            }
            save_state_or_warn(state, state_path, output.error);
        } else {
            if let Some(cb) = output.error {
                cb(&format!(
                    "Failed to {failure_verb} scoop bucket: {}",
                    bucket.name
                ));
            }
            all_ok = false;
        }
    }

    all_ok
}

#[allow(clippy::too_many_arguments)]
async fn run_operation(
    cmd: &dyn CommandRunner,
    options: &NTIXOptions,
    validation: &ValidationResult,
    state: &mut State,
    state_path: &Path,
    operation: Operation,
    pkg: &PackageSpec,
    output: &Output<'_>,
) -> bool {
    let installs = matches!(operation, Operation::Install | Operation::Upgrade);
    if installs && !(options.enabled(pkg.source) && validation.installed(pkg.source)) {
        return true;
    }

    let success = run_package_operation(cmd, operation, pkg, options, output).await;

    if success {
        update_state(state, pkg, installs);
        save_state_or_warn(state, state_path, output.error);
        true
    } else {
        let verb = match operation {
            Operation::Install => "install",
            Operation::Upgrade => "upgrade",
            Operation::Remove => "remove",
        };
        if let Some(cb) = output.error {
            cb(&format!("Failed to {verb} {}:{}", pkg.source, pkg.id));
        }
        false
    }
}

fn update_state(state: &mut State, pkg: &PackageSpec, installed: bool) {
    let packages = state.packages_mut(pkg.source);

    if installed {
        packages.insert(
            pkg.id.to_lowercase(),
            pkg.version.clone().unwrap_or_else(|| "latest".to_string()),
        );
    } else {
        packages.retain(|key, _| !key.eq_ignore_ascii_case(&pkg.id));
    }
}
