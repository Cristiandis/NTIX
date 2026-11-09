use crate::models::{
    config_file::ConfigFileEntry, import_node::ImportNode, package_entry::PackageEntry,
};

use super::options::NTIXOptions;
use super::package_manager::PackageManager;

#[derive(Debug, Clone, Default)]
pub struct NTIXConfig {
    pub options: NTIXOptions,
    pub winget_packages: Vec<PackageEntry>,
    pub choco_packages: Vec<PackageEntry>,
    pub scoop_packages: Vec<PackageEntry>,
    pub config_files: Vec<ConfigFileEntry>,
    pub imports: Vec<ImportNode>,
}

impl NTIXConfig {
    pub fn packages(&self, manager: PackageManager) -> &[PackageEntry] {
        match manager {
            PackageManager::Winget => &self.winget_packages,
            PackageManager::Chocolatey => &self.choco_packages,
            PackageManager::Scoop => &self.scoop_packages,
        }
    }
}
