use std::collections::HashMap;

use crate::models::package_manager::PackageManager;

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

impl InstalledPackages {
    pub fn packages(&self, manager: PackageManager) -> &HashMap<String, String> {
        match manager {
            PackageManager::Winget => &self.winget,
            PackageManager::Chocolatey => &self.chocolatey,
            PackageManager::Scoop => &self.scoop,
        }
    }

    pub fn packages_mut(&mut self, manager: PackageManager) -> &mut HashMap<String, String> {
        match manager {
            PackageManager::Winget => &mut self.winget,
            PackageManager::Chocolatey => &mut self.chocolatey,
            PackageManager::Scoop => &mut self.scoop,
        }
    }
}
