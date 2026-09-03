use ntix_rs::models::diff_result::DiffResult;
use ntix_rs::models::installed_packages::{InstalledPackages, UpgradeInfo};
use ntix_rs::models::ntix_config::NTIXConfig;
use ntix_rs::models::options::{NTIXOptions, ScoopOptions, WingetOptions};
use ntix_rs::models::package_entry::PackageEntry;
use ntix_rs::models::package_spec::PackageSpec;
use ntix_rs::models::state::State;
use ntix_rs::package_manager::command_builder;

#[test]
fn package_entry_default_version_is_none() {
    let entry = PackageEntry::new("test-id");
    assert_eq!(entry.id, "test-id");
    assert!(entry.version.is_none());
}

#[test]
fn package_entry_with_version_has_version() {
    let entry = PackageEntry {
        id: "test-id".to_string(),
        version: Some("1.0.0".to_string()),
    };
    assert_eq!(entry.id, "test-id");
    assert_eq!(entry.version, Some("1.0.0".to_string()));
}

#[test]
fn state_default_empty_dictionaries() {
    let state = State::default();
    assert!(state.winget.is_empty());
    assert!(state.chocolatey.is_empty());
    assert!(state.scoop.is_empty());
    assert_eq!(state.version, 2);
}

#[test]
fn state_add_package_tracks_package() {
    let mut state = State::default();
    state.winget.insert("test".to_string(), "1.0".to_string());
    assert_eq!(state.winget.get("test"), Some(&"1.0".to_string()));
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
            source: "winget".to_string(),
        }],
        ..Default::default()
    };
    assert!(!diff.is_empty());
}

#[test]
fn ntix_options_default_values() {
    let options = NTIXOptions::default();
    assert!(!options.winget.enable);
    assert!(!options.winget.accept_agreement);
    assert!(!options.winget.silent);
    assert!(!options.winget.disable_interactivity);
    assert!(!options.chocolatey.enable);
    assert!(!options.chocolatey.yes);
    assert!(!options.scoop.enable);
}

#[test]
fn scoop_options_default_buckets() {
    let scoop = ScoopOptions::default();
    assert_eq!(scoop.buckets.len(), 3);
    assert_eq!(scoop.buckets[0].name, "main");
    assert_eq!(scoop.buckets[1].name, "extras");
    assert_eq!(scoop.buckets[2].name, "versions");
}

#[test]
fn diff_result_is_empty_false_when_to_adopt_not_empty() {
    let diff = DiffResult {
        to_adopt: vec![PackageSpec {
            id: "manual-pkg".to_string(),
            version: Some("1.0".to_string()),
            source: "winget".to_string(),
        }],
        ..Default::default()
    };
    assert!(!diff.is_empty());
}

#[test]
fn diff_result_default_to_adopt_is_empty() {
    let diff = DiffResult::default();
    assert!(diff.to_adopt.is_empty());
    assert!(diff.is_empty());
}

#[test]
fn diff_result_warnings_defaults_to_empty() {
    let diff = DiffResult::default();
    assert!(diff.warnings.is_empty());
}

#[test]
fn diff_result_is_empty_true_when_only_to_skip() {
    let diff = DiffResult {
        to_skip: vec![
            PackageSpec {
                id: "pkg1".to_string(),
                version: Some("1.0".to_string()),
                source: "winget".to_string(),
            },
            PackageSpec {
                id: "pkg2".to_string(),
                version: None,
                source: "chocolatey".to_string(),
            },
        ],
        ..Default::default()
    };
    assert!(diff.is_empty());
}

#[test]
fn build_winget_uninstall_default_flags() {
    let cmd = command_builder::build_winget_uninstall("Git.Git", WingetOptions::default()).unwrap();
    // Both flags default to false: no interaction flag is added.
    assert_eq!(cmd, "winget uninstall --id Git.Git --exact");
}

#[test]
fn build_winget_uninstall_with_accept_agreements() {
    let opts = WingetOptions {
        accept_agreement: true,
        ..Default::default()
    };
    let cmd = command_builder::build_winget_uninstall("Git.Git", opts).unwrap();
    assert!(cmd.contains("--accept-source-agreements"));
    assert!(!cmd.contains("--accept-package-agreements"));
}

#[test]
fn build_winget_uninstall_fully_interactive() {
    let opts = WingetOptions {
        silent: false,
        disable_interactivity: false,
        ..Default::default()
    };
    let cmd = command_builder::build_winget_uninstall("Git.Git", opts).unwrap();
    assert!(!cmd.contains("--silent"));
    assert!(!cmd.contains("--disable-interactivity"));
    assert!(!cmd.contains("--accept"));
}

#[test]
fn build_winget_uninstall_disable_interactivity() {
    let opts = WingetOptions {
        silent: false,
        disable_interactivity: true,
        ..Default::default()
    };
    let cmd = command_builder::build_winget_uninstall("Git.Git", opts).unwrap();
    assert!(cmd.contains("--disable-interactivity"));
    assert!(!cmd.contains("--silent"));
}

#[test]
fn build_winget_uninstall_silent_takes_precedence() {
    let opts = WingetOptions {
        silent: true,
        disable_interactivity: true,
        ..Default::default()
    };
    let cmd = command_builder::build_winget_uninstall("Git.Git", opts).unwrap();
    assert!(cmd.contains("--silent"));
    assert!(!cmd.contains("--disable-interactivity"));
}

#[test]
fn installed_packages_defaults_empty() {
    let pkg = InstalledPackages::default();
    assert!(pkg.winget.is_empty());
    assert!(pkg.chocolatey.is_empty());
    assert!(pkg.scoop.is_empty());
}

#[test]
fn upgrade_info_properties() {
    let info = UpgradeInfo::new("1.0", "2.0");
    assert_eq!(info.current_version, "1.0");
    assert_eq!(info.available_version, "2.0");
}

#[test]
fn ntix_config_default_is_empty() {
    let config = NTIXConfig::default();
    assert!(config.winget_packages.is_empty());
    assert!(config.choco_packages.is_empty());
    assert!(config.scoop_packages.is_empty());
    assert!(config.imports.is_empty());
}
