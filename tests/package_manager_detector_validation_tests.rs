use ntix_rs::models::ntix_config::NTIXConfig;
use ntix_rs::models::options::{ChocoOptions, NTIXOptions, ScoopOptions, WingetOptions};
use ntix_rs::models::package_entry::PackageEntry;
use ntix_rs::package_manager::package_manager_detector;

mod common;
use common::MockManagerPresence;

fn opts(choco_enable: bool, scoop_enable: bool) -> NTIXOptions {
    NTIXOptions {
        winget: WingetOptions::default(),
        chocolatey: ChocoOptions {
            enable: choco_enable,
            ..Default::default()
        },
        scoop: ScoopOptions {
            enable: scoop_enable,
            ..Default::default()
        },
    }
}

#[test]
fn validate_managers_chocolatey_enabled_not_installed_warns_and_continues() {
    let presence = MockManagerPresence::with_choco(false);

    let options = opts(true, false);
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    let result = package_manager_detector::validate_managers(&options, &config, Some(&presence));
    assert!(!result.choco_installed);
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("Chocolatey is enabled but not installed")
            && w.contains("chocolatey.org/install")));
}

#[test]
fn validate_managers_chocolatey_enabled_installed_returns_valid() {
    let presence = MockManagerPresence::new();

    let options = opts(true, false);
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    let result = package_manager_detector::validate_managers(&options, &config, Some(&presence));
    assert!(result.choco_installed);
    assert!(result.warnings.is_empty());
}

#[test]
fn validate_managers_scoop_enabled_not_installed_warns_and_continues() {
    let mut presence = MockManagerPresence::new();
    presence.scoop_installed = false;

    let options = opts(false, true);
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    let result = package_manager_detector::validate_managers(&options, &config, Some(&presence));
    assert!(!result.scoop_installed);
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("Scoop is enabled but not installed") && w.contains("scoop.sh")));
}

#[test]
fn validate_managers_scoop_enabled_installed_returns_valid() {
    let presence = MockManagerPresence::new();

    let options = opts(false, true);
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    let result = package_manager_detector::validate_managers(&options, &config, Some(&presence));
    assert!(result.scoop_installed);
    assert!(result.warnings.is_empty());
}

#[test]
fn validate_managers_chocolatey_packages_not_enabled_returns_warning() {
    let options = opts(false, false);
    let config = NTIXConfig {
        options: options.clone(),
        choco_packages: vec![PackageEntry {
            id: "git".to_string(),
            version: Some("1.0".to_string()),
        }],
        ..Default::default()
    };

    let result = package_manager_detector::validate_managers(&options, &config, None);
    assert!(result.warnings.iter().any(|w| w.contains(
        "[warn] Chocolatey packages declared but chocolatey not enabled in options"
    )));
}

#[test]
fn validate_managers_scoop_packages_not_enabled_returns_warning() {
    let options = opts(false, false);
    let config = NTIXConfig {
        options: options.clone(),
        scoop_packages: vec![PackageEntry {
            id: "fd".to_string(),
            version: Some("1.0".to_string()),
        }],
        ..Default::default()
    };

    let result = package_manager_detector::validate_managers(&options, &config, None);
    assert!(result.warnings.iter().any(|w| w.contains(
        "[warn] Scoop packages declared but scoop not enabled in options"
    )));
}

#[test]
fn validate_managers_all_disabled_no_packages_returns_success() {
    let options = NTIXOptions::default();
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    let result = package_manager_detector::validate_managers(&options, &config, None);
    assert!(result.warnings.is_empty());
}

#[test]
fn validate_managers_null_options_handles_gracefully() {
    let options = NTIXOptions::default();
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    package_manager_detector::validate_managers(&options, &config, None);
}
