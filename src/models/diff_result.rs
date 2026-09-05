use crate::models::{
    config_file::ConfigFileEntry, manager_validation::ValidationResult, options::ScoopBucket,
    package_spec::PackageSpec,
};

#[derive(Debug, Clone, Default)]
pub struct DiffResult {
    pub to_install: Vec<PackageSpec>,
    pub to_upgrade: Vec<PackageSpec>,
    pub to_skip: Vec<PackageSpec>,
    pub to_remove: Vec<PackageSpec>,
    pub to_adopt: Vec<PackageSpec>,
    /// Packages installed on the system but not tracked in state. Informational
    /// only (not an action): run with `-a`/`--adopt` to adopt them.
    pub to_untracked: Vec<PackageSpec>,
    pub buckets_to_add: Vec<ScoopBucket>,
    pub buckets_to_remove: Vec<ScoopBucket>,
    pub config_files_to_create: Vec<ConfigFileEntry>,
    pub config_files_to_update: Vec<ConfigFileEntry>,
    /// Dest paths still tracked in state but no longer present in the config.
    pub config_files_no_longer_managed: Vec<String>,
    pub warnings: Vec<String>,
    /// Result of the single capability-detection pass for apply/diff runs.
    pub manager_validation: ValidationResult,
}

impl DiffResult {
    pub fn is_empty(&self) -> bool {
        self.to_install.is_empty()
            && self.to_upgrade.is_empty()
            && self.to_remove.is_empty()
            && self.to_adopt.is_empty()
            && self.buckets_to_add.is_empty()
            && self.buckets_to_remove.is_empty()
            && self.config_files_to_create.is_empty()
            && self.config_files_to_update.is_empty()
            && self.config_files_no_longer_managed.is_empty()
    }
}
