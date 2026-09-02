use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UpgradeInfo {
    pub current_version: String,
    pub available_version: String,
}

impl UpgradeInfo {
    pub fn new(current_version: impl Into<String>, available_version: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
            available_version: available_version.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InstalledPackages {
    pub winget: HashMap<String, String>,
    pub chocolatey: HashMap<String, String>,
    pub scoop: HashMap<String, String>,
}
