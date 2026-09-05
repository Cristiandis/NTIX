use crate::models::package_manager::PackageManager;

#[derive(Debug, Clone, Default)]
pub struct PackageSpec {
    pub id: String,
    pub version: Option<String>,
    pub source: PackageManager,
}
