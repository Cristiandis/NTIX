use std::collections::HashMap;

use crate::models::installed_packages::UpgradeInfo;
use crate::package_manager::command_builder;
use crate::package_manager::command_runner::CommandRunner;
use crate::package_manager::table_parser::parse_table;

pub struct WingetPackageEntry {
    pub id: String,
    pub version: String,
    pub available: Option<String>,
}

impl WingetPackageEntry {
    fn is_upgradable(&self) -> bool {
        self.version != "Unknown" && self.available.is_some()
    }
}

async fn list_entries(
    runner: &dyn CommandRunner,
) -> Result<Vec<WingetPackageEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let command = command_builder::build_winget_list();
    let output = runner.run_output(&command, true).await;

    if output.is_empty() {
        return Err("Failed to list packages".into());
    }

    let mut output = output.lines();
    let first_line = output.next();
    let first_line = first_line.and_then(|x| x.split('\r').next_back());
    let lines = first_line.into_iter().chain(output);

    let (column_len, cells) = parse_table(lines).map_err(|e| e.to_string())?;
    if column_len < 4 {
        return Err(format!("Invalid header: {first_line:?}").into());
    }

    let mut entries = Vec::new();
    for columns in cells
        .into_iter()
        .skip(column_len)
        .collect::<Vec<_>>()
        .chunks(column_len)
    {
        if columns.len() < 4 {
            continue;
        }
        let id = columns[1].clone();
        let version = columns[2].clone();
        let available = if column_len >= 5 {
            columns.get(3).filter(|s| !s.is_empty()).cloned()
        } else {
            None
        };
        entries.push(WingetPackageEntry {
            id,
            version,
            available,
        });
    }

    Ok(entries)
}

pub async fn get_installed_packages(
    runner: &dyn CommandRunner,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    let entries = list_entries(runner).await?;
    Ok(entries.into_iter().map(|e| (e.id, e.version)).collect())
}

pub async fn get_upgradable_packages(
    runner: &dyn CommandRunner,
) -> Result<HashMap<String, UpgradeInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let entries = list_entries(runner).await?;
    Ok(entries
        .into_iter()
        .filter(|e| e.is_upgradable())
        .map(|e| {
            let available = e.available.clone().unwrap_or_default();
            (e.id, UpgradeInfo::new(&e.version, &available))
        })
        .collect())
}

pub async fn package_exists(
    runner: &dyn CommandRunner,
    id: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let command = command_builder::build_winget_search(id).map_err(|e| e.to_string())?;
    let text = runner.run_output(&command, true).await;
    Ok(text
        .lines()
        .any(|l| l.split_whitespace().any(|t| t.eq_ignore_ascii_case(id))))
}

pub async fn is_installed(runner: &dyn CommandRunner) -> bool {
    let output = runner
        .run_output(&command_builder::build_winget_version(), true)
        .await;
    !output.is_empty()
}

pub async fn ensure_installed(runner: &dyn CommandRunner) {
    let ps_command = command_builder::build_powershell_install_winget();
    let command =
        format!("powershell.exe -NoProfile -ExecutionPolicy Bypass -Command \"{ps_command}\"");
    let _ = runner.run(&command, None, None).await;
}
