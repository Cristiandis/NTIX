use std::collections::HashMap;

use regex::Regex;

use crate::models::installed_packages::UpgradeInfo;
use crate::package_manager::command_builder;
use crate::package_manager::command_runner::CommandRunner;

pub async fn is_installed(runner: &dyn CommandRunner) -> bool {
    !runner
        .run_output(&command_builder::build_choco_version(), true)
        .await
        .is_empty()
}

pub async fn get_installed_packages(
    runner: &dyn CommandRunner,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut result = HashMap::new();
    let output = runner
        .run_output(&command_builder::build_choco_list_installed(), true)
        .await;
    if output.is_empty() {
        return Err("Failed to list packages".into());
    }
    let regex = Regex::new(r"(?m)^([^|]+)\|([^|]+)$").unwrap();
    for cap in regex.captures_iter(&output) {
        let id = cap[1].trim();
        let ver = cap[2].trim();
        if !id.is_empty() && !ver.is_empty() {
            result.insert(id.to_string(), ver.to_string());
        }
    }
    Ok(result)
}

pub async fn get_upgradable_packages(runner: &dyn CommandRunner) -> HashMap<String, UpgradeInfo> {
    let mut result = HashMap::new();
    let output = runner
        .run_output(&command_builder::build_choco_outdated(), true)
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

pub async fn package_exists(
    runner: &dyn CommandRunner,
    id: &str,
) -> Result<Option<bool>, Box<dyn std::error::Error + Send + Sync>> {
    let command = command_builder::build_choco_search(id).map_err(|e| e.to_string())?;
    let output = runner.run_output(&command, true).await;
    if output.is_empty() {
        return Ok(Some(false));
    }
    let pattern = format!(r"(?mi)^{}\|", regex::escape(id));
    if Regex::new(&pattern).unwrap().is_match(&output) {
        Ok(Some(true))
    } else {
        Ok(None)
    }
}
