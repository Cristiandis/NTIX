use std::collections::HashMap;

use futures::future::join_all;

use crate::models::installed_packages::{InstalledPackages, UpgradeInfo};
use crate::models::manager_validation::ValidationResult;
use crate::models::ntix_config::NTIXConfig;
use crate::models::package_manager::PackageManager;
use crate::package_manager::choco_ops;
use crate::package_manager::command_runner::CommandRunner;
use crate::package_manager::probes::Presence;
use crate::package_manager::process_command_runner::ProcessCommandRunner;
use crate::package_manager::scoop_ops;
use crate::package_manager::winget_ops;

pub async fn validate_managers(
    config: &NTIXConfig,
    runner: Option<&dyn CommandRunner>,
    presence: Option<Presence>,
) -> ValidationResult {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let (winget_installed, choco_installed, scoop_installed) = match presence {
        Some(presence) => (presence.winget, presence.choco, presence.scoop),
        None => {
            let winget = detect_winget(cmd, config.options.winget.enable).await;
            let (choco, scoop) =
                tokio::join!(choco_ops::is_installed(cmd), scoop_ops::is_installed(cmd));
            (winget, choco, scoop)
        }
    };

    let mut result = ValidationResult {
        warnings: Vec::new(),
        winget_installed,
        choco_installed,
        scoop_installed,
    };
    result.warnings = collect_warnings(config, &result);
    result
}

/// Winget is the only manager ntix installs on the user's behalf, so it has an
/// extra detect -> install -> detect pass when enabled but missing.
async fn detect_winget(cmd: &dyn CommandRunner, enabled: bool) -> bool {
    let detected = winget_ops::is_installed(cmd).await;
    if enabled && !detected {
        winget_ops::ensure_installed(cmd).await;
        winget_ops::is_installed(cmd).await
    } else {
        detected
    }
}

pub fn collect_warnings(config: &NTIXConfig, validation: &ValidationResult) -> Vec<String> {
    const INSTALL_HINTS: [(PackageManager, &str, &str, &str); 3] = [
        (
            PackageManager::Chocolatey,
            "https://chocolatey.org/install",
            "",
            "",
        ),
        (
            PackageManager::Scoop,
            "https://scoop.sh",
            "",
            " and buckets",
        ),
        (
            PackageManager::Winget,
            "https://github.com/microsoft/winget-cli",
            " Auto-install failed.",
            "",
        ),
    ];

    let mut warnings = Vec::new();

    for (manager, url, install_note, skip_scope) in INSTALL_HINTS {
        if config.options.enabled(manager) && !validation.installed(manager) {
            let name = manager.display_name();
            warnings.push(format!(
                "[warn] {name} is enabled but not installed.{install_note} Skipping {name} packages{skip_scope}. Install from {url}"
            ));
        }
    }

    for (manager, _, _, _) in INSTALL_HINTS {
        if !config.packages(manager).is_empty() && !config.options.enabled(manager) {
            warnings.push(format!(
                "[warn] {} packages declared but {} not enabled in options",
                manager.display_name(),
                manager
            ));
        }
    }

    warnings
}

pub async fn get_installed_packages(runner: Option<&dyn CommandRunner>) -> InstalledPackages {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let (winget, choco, scoop) = tokio::join!(
        winget_ops::get_installed_packages(cmd),
        choco_ops::get_installed_packages(cmd),
        scoop_ops::get_installed_packages(cmd)
    );

    let mut result = InstalledPackages::default();
    for (manager, outcome) in [
        (PackageManager::Winget, winget),
        (PackageManager::Chocolatey, choco),
        (PackageManager::Scoop, scoop),
    ] {
        match outcome {
            Ok(packages) => {
                result.packages_mut(manager).extend(packages);
            }
            Err(e) => eprintln!("[NTIX] {} detection failed: {e}", manager.display_name()),
        }
    }

    result
}

/// Returns the upgradable packages reported by `manager`; a failed winget
/// query degrades to an empty list.
pub async fn get_upgradable_packages(
    manager: PackageManager,
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, UpgradeInfo> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);
    match manager {
        PackageManager::Winget => winget_ops::get_upgradable_packages(cmd)
            .await
            .unwrap_or_default(),
        PackageManager::Chocolatey => choco_ops::get_upgradable_packages(cmd).await,
        PackageManager::Scoop => scoop_ops::get_upgradable_packages(cmd).await,
    }
}

/// Checks that each of `ids` exists in `manager`'s repository. Value `None`
/// means the check itself could not be trusted; `Some(false)` means not found.
pub async fn validate_packages_exist(
    manager: PackageManager,
    ids: &[String],
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, Option<bool>> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let tasks = ids.iter().map(|id| async move {
        let exists = match manager {
            PackageManager::Winget => winget_ops::package_exists(cmd, id).await.ok(),
            PackageManager::Chocolatey => choco_ops::package_exists(cmd, id).await.ok().flatten(),
            PackageManager::Scoop => scoop_ops::package_exists(cmd, id).await.ok().flatten(),
        };
        (id.clone(), exists)
    });

    join_all(tasks).await.into_iter().collect()
}
