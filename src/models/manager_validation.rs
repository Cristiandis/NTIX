use crate::models::package_manager::PackageManager;

#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub warnings: Vec<String>,
    pub winget_installed: bool,
    pub choco_installed: bool,
    pub scoop_installed: bool,
}

impl ValidationResult {
    /// Whether `manager` was detected on the system.
    pub fn installed(&self, manager: PackageManager) -> bool {
        match manager {
            PackageManager::Winget => self.winget_installed,
            PackageManager::Chocolatey => self.choco_installed,
            PackageManager::Scoop => self.scoop_installed,
        }
    }
}
