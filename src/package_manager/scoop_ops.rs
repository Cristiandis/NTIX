use std::collections::HashMap;

use regex::Regex;

use crate::models::installed_packages::UpgradeInfo;
use crate::package_manager::command_builder;
use crate::package_manager::command_runner::CommandRunner;

pub async fn is_installed(runner: &dyn CommandRunner) -> bool {
    !runner
        .run_output(&command_builder::build_scoop_version(), true)
        .await
        .is_empty()
}

pub async fn get_installed_packages(
    runner: &dyn CommandRunner,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut result = HashMap::new();
    let output = runner
        .run_output(&command_builder::build_scoop_list_installed(), true)
        .await;
    if output.is_empty() {
        return Err("Failed to list packages".into());
    }
    for line in output.split('\n') {
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
                result.insert(id.to_string(), ver.to_string());
            }
        }
    }
    Ok(result)
}

pub async fn get_upgradable_packages(runner: &dyn CommandRunner) -> HashMap<String, UpgradeInfo> {
    let mut result = HashMap::new();
    let output = runner
        .run_output(&command_builder::build_scoop_status(), true)
        .await;
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

pub async fn package_exists(
    runner: &dyn CommandRunner,
    id: &str,
) -> Result<Option<bool>, Box<dyn std::error::Error + Send + Sync>> {
    let command = command_builder::build_scoop_info(id).map_err(|e| e.to_string())?;
    let output = runner.run_output(&command, true).await;
    if Regex::new(r"(?mi)^\s*Name\s*:").unwrap().is_match(&output) {
        return Ok(Some(true));
    }
    if output.to_lowercase().contains("couldn't find manifest") {
        return Ok(Some(false));
    }
    Ok(None)
}
