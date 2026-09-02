use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::process::Command;

use regex::Regex;

use crate::models::installed_packages::InstalledPackages;
use crate::models::installed_packages::UpgradeInfo;
use crate::models::{ntix_config::NTIXConfig, options::NTIXOptions};
use crate::package_manager::command_builder;
use crate::package_manager::command_runner::CommandRunner;
use crate::package_manager::manager_presence::ManagerPresence;
use crate::package_manager::process_command_runner::ProcessCommandRunner;
use crate::package_manager::winget_manager::WingetManager;
use crate::package_manager::winget_manager_trait::WingetManagerTrait;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn is_chocolatey_installed() -> bool {
    run_process("choco --version").is_some()
}

pub fn is_scoop_installed() -> bool {
    run_process("scoop --version").is_some()
}

pub struct ValidationResult {
    pub warnings: Vec<String>,
    pub winget_installed: bool,
    pub choco_installed: bool,
    pub scoop_installed: bool,
}

pub async fn validate_managers_async(
    options: &NTIXOptions,
    config: &NTIXConfig,
    winget_manager: Option<&dyn WingetManagerTrait>,
    presence: Option<&dyn ManagerPresence>,
) -> ValidationResult {
    let (choco_installed, scoop_installed) = check_choco_scoop(presence);

    let mut winget_installed = true;
    if options.winget.enable {
        let mgr: &dyn WingetManagerTrait = winget_manager.unwrap_or(&WingetManager);
        winget_installed = mgr.is_installed();
        if !winget_installed {
            mgr.ensure_installed().await;
            winget_installed = mgr.is_installed();
        }
    }

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

pub fn validate_managers(
    options: &NTIXOptions,
    config: &NTIXConfig,
    presence: Option<&dyn ManagerPresence>,
) -> ValidationResult {
    let (choco_installed, scoop_installed) = check_choco_scoop(presence);

    ValidationResult {
        warnings: collect_warnings(options, config, true, choco_installed, scoop_installed),
        winget_installed: true,
        choco_installed,
        scoop_installed,
    }
}

fn check_choco_scoop(presence: Option<&dyn ManagerPresence>) -> (bool, bool) {
    let choco_installed = match presence {
        Some(p) => p.is_chocolatey_installed(),
        None => is_chocolatey_installed(),
    };
    let scoop_installed = match presence {
        Some(p) => p.is_scoop_installed(),
        None => is_scoop_installed(),
    };
    (choco_installed, scoop_installed)
}

fn collect_warnings(
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

    warnings
}

pub async fn get_installed_packages_async(
    winget_manager: Option<&dyn WingetManagerTrait>,
    runner: Option<&dyn CommandRunner>,
) -> InstalledPackages {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let mut result = InstalledPackages::default();

    let winget_manager: &dyn WingetManagerTrait = winget_manager.unwrap_or(&WingetManager);

    match winget_manager.get_installed_packages().await {
        Ok(pkgs) => {
            for (id, ver) in pkgs {
                result.winget.insert(id, ver);
            }
        }
        Err(e) => {
            eprintln!("[NTIX] Winget detection failed: {e}");
        }
    }

    let choco_out = cmd
        .run_output("choco list -r --local-only --limit-output 2>nul", true)
        .await;
    if !choco_out.is_empty() {
        let regex = Regex::new(r"(?m)^([^|]+)\|([^|]+)$").unwrap();
        for cap in regex.captures_iter(&choco_out) {
            let id = cap[1].trim();
            let ver = cap[2].trim();
            if !id.is_empty() && !ver.is_empty() {
                result.chocolatey.insert(id.to_string(), ver.to_string());
            }
        }
    }

    let scoop_out = cmd.run_output("scoop list 2>nul", true).await;
    if !scoop_out.is_empty() {
        for line in scoop_out.split('\n') {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('-')
                || trimmed.starts_with("Installed")
                || trimmed.starts_with("Name")
                || trimmed.contains("aren't any")
            {
                continue;
            }
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let id = parts[0];
                let ver = parts[1];
                if !id.is_empty() && !ver.is_empty() {
                    result.scoop.insert(id.to_string(), ver.to_string());
                }
            }
        }
    }

    result
}

pub async fn get_winget_upgradable_packages_async(
    winget_manager: Option<&dyn WingetManagerTrait>,
) -> HashMap<String, UpgradeInfo> {
    let manager: &dyn WingetManagerTrait = winget_manager.unwrap_or(&WingetManager);
    manager.get_upgradable_packages().await.unwrap_or_default()
}

pub async fn get_choco_upgradable_packages_async(
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, UpgradeInfo> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let mut result = HashMap::new();
    let output = cmd
        .run_output("choco outdated --limit-output 2>nul", true)
        .await;
    if output.is_empty() {
        return result;
    }

    let regex = Regex::new(r"(?m)^([^|]+)\|([^|]+)\|([^|]+)\|.*$").unwrap();
    for cap in regex.captures_iter(&output) {
        let id = cap[1].trim();
        let cur = cap[2].trim();
        let avail = cap[3].trim();
        if !id.is_empty() && !cur.is_empty() && !avail.is_empty() {
            result.insert(id.to_string(), UpgradeInfo::new(cur, avail));
        }
    }

    result
}

pub async fn get_scoop_upgradable_packages_async(
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, UpgradeInfo> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let mut result = HashMap::new();
    let output = cmd.run_output("scoop status --json 2>nul", true).await;
    if output.is_empty() {
        return result;
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&output)
        && let Some(items) = value.as_array()
    {
        for item in items {
            let id = item.get("name").and_then(|v| v.as_str());
            let cur = item.get("current_version").and_then(|v| v.as_str());
            let avail = item.get("latest_version").and_then(|v| v.as_str());
            if let (Some(id), Some(cur), Some(avail)) = (id, cur, avail)
                && !id.is_empty()
                && !cur.is_empty()
                && !avail.is_empty()
                && cur != avail
            {
                result.insert(id.to_string(), UpgradeInfo::new(cur, avail));
            }
        }
    }

    result
}

pub async fn validate_choco_package_exists_async(id: &str, runner: &dyn CommandRunner) -> bool {
    let output = runner
        .run_output(&command_builder::build_choco_search(id).unwrap(), true)
        .await;
    if output.is_empty() {
        return false;
    }
    let pattern = format!(r"(?mi)^{}\|", regex::escape(id));
    Regex::new(&pattern).unwrap().is_match(&output)
}

pub async fn validate_scoop_package_exists_async(id: &str, runner: &dyn CommandRunner) -> bool {
    let output = runner
        .run_output(&command_builder::build_scoop_info(id).unwrap(), true)
        .await;
    if output.is_empty() {
        return false;
    }
    let pattern = Regex::new(r"(?mi)^\s*Name\s*:").unwrap();
    pattern.is_match(&output)
}

pub async fn validate_winget_packages_exist_async(
    ids: &[String],
    winget_manager: Option<&dyn WingetManagerTrait>,
) -> HashMap<String, Option<bool>> {
    let mgr: &dyn WingetManagerTrait = winget_manager.unwrap_or(&WingetManager);

    let tasks = ids.iter().map(|id| async move {
        let exists = mgr.package_exists(id).await.ok();
        (id.clone(), exists)
    });

    futures::future::join_all(tasks).await.into_iter().collect()
}

pub async fn validate_choco_packages_exist_async(
    ids: &[String],
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, bool> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let tasks = ids.iter().map(|id| async move {
        let exists = validate_choco_package_exists_async(id, cmd).await;
        (id.to_lowercase(), exists)
    });

    futures::future::join_all(tasks).await.into_iter().collect()
}

pub async fn validate_scoop_packages_exist_async(
    ids: &[String],
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, bool> {
    let cmd: &dyn CommandRunner = runner.unwrap_or(&ProcessCommandRunner);

    let tasks = ids.iter().map(|id| async move {
        let exists = validate_scoop_package_exists_async(id, cmd).await;
        (id.to_lowercase(), exists)
    });

    futures::future::join_all(tasks).await.into_iter().collect()
}

fn run_process(cmd: &str) -> Option<String> {
    let output = Command::new("cmd.exe")
        .arg("/c")
        .raw_arg(format!("{cmd} 2>nul"))
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
