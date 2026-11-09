use ntix_rs::models::diff_result::DiffResult;
use ntix_rs::models::ntix_config::NTIXConfig;
use ntix_rs::models::options::{NTIXOptions, ScoopOptions};
use ntix_rs::models::package_entry::PackageEntry;
use ntix_rs::models::package_manager::PackageManager;
use ntix_rs::models::package_spec::PackageSpec;
use ntix_rs::models::state::State;

#[test]
fn package_entry_new_sets_id_without_version() {
    let entry = PackageEntry::new("test-id");
    assert_eq!(entry.id, "test-id");
    assert!(entry.version.is_none());
}

/// The schema version decides how a loaded state file is migrated.
#[test]
fn state_default_writes_schema_version_two() {
    let state = State::default();
    assert_eq!(state.version, 2);
}

#[test]
fn diff_result_is_empty_when_all_empty() {
    let diff = DiffResult::default();
    assert!(diff.is_empty());
}

#[test]
fn diff_result_is_empty_false_when_has_items() {
    let diff = DiffResult {
        to_install: vec![PackageSpec {
            id: "test".to_string(),
            version: None,
            source: PackageManager::Winget,
        }],
        ..Default::default()
    };
    assert!(!diff.is_empty());
}

/// Informational lists (to_skip, to_untracked) never trigger apply work.
#[test]
fn diff_result_is_empty_true_when_only_informational() {
    let diff = DiffResult {
        to_skip: vec![
            PackageSpec {
                id: "pkg1".to_string(),
                version: Some("1.0".to_string()),
                source: PackageManager::Winget,
            },
            PackageSpec {
                id: "pkg2".to_string(),
                version: None,
                source: PackageManager::Chocolatey,
            },
        ],
        to_untracked: vec![PackageSpec {
            id: "pkg3".to_string(),
            version: None,
            source: PackageManager::Scoop,
        }],
        ..Default::default()
    };
    assert!(diff.is_empty());
}

/// Adoption mutates state, so it must not be treated as an empty diff.
#[test]
fn diff_result_is_empty_false_when_only_to_adopt() {
    let diff = DiffResult {
        to_adopt: vec![PackageSpec {
            id: "pkg".to_string(),
            version: None,
            source: PackageManager::Winget,
        }],
        ..Default::default()
    };
    assert!(!diff.is_empty());
}

/// The scoop feature reads buckets off default options when the config
/// declares none, so the default bucket list is behavior.
#[test]
fn scoop_options_default_buckets() {
    let scoop = ScoopOptions::default();
    assert_eq!(scoop.buckets.len(), 3);
    assert_eq!(scoop.buckets[0].name, "main");
    assert_eq!(scoop.buckets[1].name, "extras");
    assert_eq!(scoop.buckets[2].name, "versions");
    assert!(!scoop.enable);
}

/// Managers are opt-in: a default options value must never match any
/// installed-manager capability.
#[test]
fn ntix_options_default_disables_all_managers() {
    let options = NTIXOptions::default();
    assert!(!options.winget.enable);
    assert!(!options.chocolatey.enable);
    assert!(!options.scoop.enable);
}

/// A default config declares no packages, imports or config files.
#[test]
fn ntix_config_default_is_empty() {
    let config = NTIXConfig::default();
    assert!(config.winget_packages.is_empty());
    assert!(config.choco_packages.is_empty());
    assert!(config.scoop_packages.is_empty());
    assert!(config.imports.is_empty());
    assert!(config.config_files.is_empty());
}
