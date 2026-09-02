use crate::models::{import_node::ImportNode, package_entry::PackageEntry};

use super::options::NTIXOptions;

#[derive(Debug, Clone, Default)]
pub struct NTIXConfig {
    pub options: NTIXOptions,
    pub winget_packages: Vec<PackageEntry>,
    pub choco_packages: Vec<PackageEntry>,
    pub scoop_packages: Vec<PackageEntry>,
    pub imports: Vec<ImportNode>,
}
