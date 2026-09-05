use std::collections::HashMap;

use crate::models::installed_packages::InstalledPackages;
use crate::models::installed_packages::UpgradeInfo;
use crate::models::{ntix_config::NTIXConfig, options::NTIXOptions};
use crate::package_manager::choco_ops;
use crate::package_manager::command_runner::CommandRunner;
use crate::package_manager::process_command_runner::ProcessCommandRunner;
use crate::package_manager::scoop_ops;
use crate::package_manager::winget_ops;

pub use crate::models::manager_validation::ValidationResult;

pub async fn validate_managers_async(
    options: &NTIXOptions,
    config: &NTIXConfig,
    winget_installed: Option<bool>,
    choco_installed: Option<bool>,
    scoop_installed: Option<bool>,
    runner: Option<&dyn CommandRunner>,
) -> ValidationResult {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);
    let (choco_installed, scoop_installed) =
        check_choco_scoop(cmd, choco_installed, scoop_installed).await;

    let winget_installed = match winget_installed {
        Some(v) => v,
        None => {
            let detected = winget_ops::is_installed(cmd).await;
            if options.winget.enable && !detected {
                winget_ops::ensure_installed(cmd).await;
                winget_ops::is_installed(cmd).await
            } else {
                detected
            }
        }
    };

    ValidationResult {
        warnings: collect_warnings(
            options,
            config,
            winget_installed,
            choco_installed,
            scoop_installed,
        ),
        winget_installed,
        choco_installed,
        scoop_installed,
    }
}

async fn check_choco_scoop(
    runner: &dyn CommandRunner,
    choco_override: Option<bool>,
    scoop_override: Option<bool>,
) -> (bool, bool) {
    let choco_installed = match choco_override {
        Some(v) => v,
        None => choco_ops::is_installed(runner).await,
    };
    let scoop_installed = match scoop_override {
        Some(v) => v,
        None => scoop_ops::is_installed(runner).await,
    };
    (choco_installed, scoop_installed)
}

pub fn collect_warnings(
    options: &NTIXOptions,
    config: &NTIXConfig,
    winget_installed: bool,
    choco_installed: bool,
    scoop_installed: bool,
) -> Vec<String> {
    let mut warnings = Vec::new();

    if options.chocolatey.enable && !choco_installed {
        warnings.push(
            "[warn] Chocolatey is enabled but not installed. Skipping Chocolatey packages. Install from https://chocolatey.org/install".to_string(),
        );
    }

    if options.scoop.enable && !scoop_installed {
        warnings.push(
            "[warn] Scoop is enabled but not installed. Skipping Scoop packages and buckets. Install from https://scoop.sh".to_string(),
        );
    }

    if options.winget.enable && !winget_installed {
        warnings.push(
            "[warn] Winget is enabled but not installed. Auto-install failed. Skipping Winget packages. Install from https://github.com/microsoft/winget-cli".to_string(),
        );
    }

    if !config.choco_packages.is_empty() && !options.chocolatey.enable {
        warnings.push(
            "[warn] Chocolatey packages declared but chocolatey not enabled in options".to_string(),
        );
    }

    if !config.scoop_packages.is_empty() && !options.scoop.enable {
        warnings
            .push("[warn] Scoop packages declared but scoop not enabled in options".to_string());
    }

    if !config.winget_packages.is_empty() && !options.winget.enable {
        warnings
            .push("[warn] Winget packages declared but winget not enabled in options".to_string());
    }

    warnings
}

pub async fn get_installed_packages_async(runner: Option<&dyn CommandRunner>) -> InstalledPackages {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let mut result = InstalledPackages::default();

    match winget_ops::get_installed_packages(cmd).await {
        Ok(pkgs) => {
            for (id, ver) in pkgs {
                result.winget.insert(id, ver);
            }
        }
        Err(e) => {
            eprintln!("[NTIX] Winget detection failed: {e}");
        }
    }

    let (choco_pkgs, scoop_pkgs) = tokio::join!(
        async {
            match choco_ops::get_installed_packages(cmd).await {
                Ok(pkgs) => pkgs,
                Err(e) => {
                    eprintln!("[NTIX] Chocolatey detection failed: {e}");
                    HashMap::new()
                }
            }
        },
        async {
            match scoop_ops::get_installed_packages(cmd).await {
                Ok(pkgs) => pkgs,
                Err(e) => {
                    eprintln!("[NTIX] Scoop detection failed: {e}");
                    HashMap::new()
                }
            }
        }
    );
    for (id, ver) in choco_pkgs {
        result.chocolatey.insert(id, ver);
    }
    for (id, ver) in scoop_pkgs {
        result.scoop.insert(id, ver);
    }

    result
}

pub async fn get_winget_upgradable_packages_async(
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, UpgradeInfo> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);
    winget_ops::get_upgradable_packages(cmd)
        .await
        .unwrap_or_default()
}

pub async fn get_choco_upgradable_packages_async(
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, UpgradeInfo> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);
    choco_ops::get_upgradable_packages(cmd).await
}

pub async fn get_scoop_upgradable_packages_async(
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, UpgradeInfo> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);
    scoop_ops::get_upgradable_packages(cmd).await
}

pub async fn validate_choco_package_exists_async(
    id: &str,
    runner: &dyn CommandRunner,
) -> Option<bool> {
    choco_ops::package_exists(runner, id).await.ok().flatten()
}

pub async fn validate_scoop_package_exists_async(
    id: &str,
    runner: &dyn CommandRunner,
) -> Option<bool> {
    scoop_ops::package_exists(runner, id).await.ok().flatten()
}

pub async fn validate_winget_packages_exist_async(
    ids: &[String],
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, Option<bool>> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let tasks = ids.iter().map(|id| async move {
        let exists = winget_ops::package_exists(cmd, id).await.ok();
        (id.clone(), exists)
    });

    futures::future::join_all(tasks).await.into_iter().collect()
}

pub async fn validate_choco_packages_exist_async(
    ids: &[String],
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, Option<bool>> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let tasks = ids.iter().map(|id| async move {
        let exists = validate_choco_package_exists_async(id, cmd).await;
        (id.clone(), exists)
    });

    futures::future::join_all(tasks).await.into_iter().collect()
}

pub async fn validate_scoop_packages_exist_async(
    ids: &[String],
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, Option<bool>> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let tasks = ids.iter().map(|id| async move {
        let exists = validate_scoop_package_exists_async(id, cmd).await;
        (id.clone(), exists)
    });

    futures::future::join_all(tasks).await.into_iter().collect()
}
