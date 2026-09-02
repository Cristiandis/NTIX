use crate::models::{options::ScoopBucket, package_spec::PackageSpec};

#[derive(Debug, Clone, Default)]
pub struct DiffResult {
    pub to_install: Vec<PackageSpec>,
    pub to_upgrade: Vec<PackageSpec>,
    pub to_skip: Vec<PackageSpec>,
    pub to_remove: Vec<PackageSpec>,
    pub to_adopt: Vec<PackageSpec>,
    pub buckets_to_add: Vec<ScoopBucket>,
    pub buckets_to_remove: Vec<ScoopBucket>,
    pub warnings: Vec<String>,
}

impl DiffResult {
    pub fn is_empty(&self) -> bool {
        self.to_install.is_empty()
            && self.to_upgrade.is_empty()
            && self.to_remove.is_empty()
            && self.to_adopt.is_empty()
            && self.buckets_to_add.is_empty()
            && self.buckets_to_remove.is_empty()
    }
}