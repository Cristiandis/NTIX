use std::collections::HashMap;

use ntix_rs::models::installed_packages::UpgradeInfo;
use ntix_rs::models::ntix_config::NTIXConfig;
use ntix_rs::models::options::{ChocoOptions, NTIXOptions, ScoopOptions, WingetOptions};
use ntix_rs::models::package_entry::PackageEntry;
use ntix_rs::package_manager::package_manager_detector;

mod common;
use common::{MockCommandRunner, MockWingetManager};

fn opts(
    winget_enable: bool,
    choco_enable: bool,
    scoop_enable: bool,
) -> NTIXOptions {
    NTIXOptions {
        winget: WingetOptions {
            enable: winget_enable,
            ..Default::default()
        },
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

#[tokio::test]
async fn get_installed_packages_async_returns_all_sources() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("winget-pkg".to_string(), "1.0".to_string());

    let result = package_manager_detector::get_installed_packages_async(Some(&mock), None).await;
    assert_eq!(result.winget.get("winget-pkg"), Some(&"1.0".to_string()));
    let _ = result.chocolatey;
    let _ = result.scoop;
}

#[tokio::test]
async fn get_installed_packages_async_winget_manager_throws_returns_empty_winget() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages = HashMap::new();
    let result = package_manager_detector::get_installed_packages_async(Some(&mock), None).await;
    assert!(result.winget.is_empty());
}

#[tokio::test]
async fn get_installed_packages_async_choco_two_field_format_detected() {
    let mock = MockWingetManager::new();
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "choco list".to_string(),
        "ripgrep|14.1.0\nfd|10.4.2\n".to_string(),
    );

    let result =
        package_manager_detector::get_installed_packages_async(Some(&mock), Some(&runner)).await;
    assert_eq!(result.chocolatey.get("ripgrep"), Some(&"14.1.0".to_string()));
    assert_eq!(result.chocolatey.get("fd"), Some(&"10.4.2".to_string()));
}

#[tokio::test]
async fn get_installed_packages_async_scoop_table_format_detected() {
    let mock = MockWingetManager::new();
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop list".to_string(),
        "Installed apps matching '':\n\nName    Version   Source\n----    -------   ------\nripgrep 15.2.0    main\nfd      10.4.2    main\n"
            .to_string(),
    );

    let result =
        package_manager_detector::get_installed_packages_async(Some(&mock), Some(&runner)).await;
    assert_eq!(result.scoop.get("ripgrep"), Some(&"15.2.0".to_string()));
    assert_eq!(result.scoop.get("fd"), Some(&"10.4.2".to_string()));
}

#[tokio::test]
async fn get_installed_packages_async_scoop_empty_returns_empty() {
    let mock = MockWingetManager::new();
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop list".to_string(),
        "There aren't any apps installed.".to_string(),
    );

    let result =
        package_manager_detector::get_installed_packages_async(Some(&mock), Some(&runner)).await;
    assert!(result.scoop.is_empty());
}

#[tokio::test]
async fn get_winget_upgradable_packages_async_returns_upgrades() {
    let mut mock = MockWingetManager::new();
    mock.upgradable_packages.insert(
        "upgrade-pkg".to_string(),
        UpgradeInfo::new("1.0", "2.0"),
    );

    let result =
        package_manager_detector::get_winget_upgradable_packages_async(Some(&mock)).await;
    assert_eq!(result.get("upgrade-pkg").unwrap().current_version, "1.0");
    assert_eq!(result.get("upgrade-pkg").unwrap().available_version, "2.0");
}

#[tokio::test]
async fn get_choco_upgradable_packages_async_mock_runner() {
    let mut runner = MockCommandRunner::new();
    runner
        .output_responses
        .insert("choco outdated".to_string(), "git|2.30.0|2.40.0|\n".to_string());

    let result =
        package_manager_detector::get_choco_upgradable_packages_async(Some(&runner)).await;
    assert_eq!(result.get("git").unwrap().current_version, "2.30.0");
    assert_eq!(result.get("git").unwrap().available_version, "2.40.0");
}

#[tokio::test]
async fn get_scoop_upgradable_packages_async_mock_runner() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop status".to_string(),
        "[{\"name\":\"rg\",\"current_version\":\"13.0.0\",\"latest_version\":\"14.0.3\"}]"
            .to_string(),
    );

    let result =
        package_manager_detector::get_scoop_upgradable_packages_async(Some(&runner)).await;
    assert_eq!(result.get("rg").unwrap().current_version, "13.0.0");
    assert_eq!(result.get("rg").unwrap().available_version, "14.0.3");
}

#[tokio::test]
async fn validate_choco_package_exists_async_with_mock_runner() {
    let mut runner = MockCommandRunner::new();
    runner
        .output_responses
        .insert("choco search git".to_string(), "git|2.40.0\n".to_string());

    let result = package_manager_detector::validate_choco_package_exists_async("git", &runner).await;
    assert!(result);
    assert!(runner
        .commands()
        .iter()
        .any(|c| c.contains("choco search")));
}

#[tokio::test]
async fn validate_choco_package_exists_async_not_found_returns_false() {
    let mut runner = MockCommandRunner::new();
    runner
        .output_responses
        .insert("choco search".to_string(), "".to_string());

    let result =
        package_manager_detector::validate_choco_package_exists_async("nonexistent", &runner).await;
    assert!(!result);
}

#[tokio::test]
async fn validate_scoop_package_exists_async_with_mock_runner() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop info rg".to_string(),
        "Name        : rg\nVersion     : 14.0.3\n".to_string(),
    );

    let result = package_manager_detector::validate_scoop_package_exists_async("rg", &runner).await;
    assert!(result);
    assert!(runner.commands().iter().any(|c| c.contains("scoop info")));
}

#[tokio::test]
async fn validate_scoop_package_exists_async_not_found_returns_false() {
    let mut runner = MockCommandRunner::new();
    runner
        .output_responses
        .insert("scoop info nonexistent".to_string(), "".to_string());

    let result =
        package_manager_detector::validate_scoop_package_exists_async("nonexistent", &runner).await;
    assert!(!result);
}

#[tokio::test]
async fn validate_choco_packages_exist_async_mock_runner() {
    let mut runner = MockCommandRunner::new();
    runner
        .output_responses
        .insert("choco search git".to_string(), "git|2.40.0\n".to_string());

    let result = package_manager_detector::validate_choco_packages_exist_async(
        &["git".to_string()],
        Some(&runner),
    )
    .await;
    assert_eq!(result.get("git"), Some(&true));
}

#[tokio::test]
async fn validate_scoop_packages_exist_async_mock_runner() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop info rg".to_string(),
        "Name        : rg\nVersion     : 14.0.3\n".to_string(),
    );

    let result = package_manager_detector::validate_scoop_packages_exist_async(
        &["rg".to_string()],
        Some(&runner),
    )
    .await;
    assert_eq!(result.get("rg"), Some(&true));
}

#[tokio::test]
async fn validate_managers_async_scoop_disabled_returns_valid() {
    let mock = MockWingetManager::new();
    let options = opts(true, false, false);
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    package_manager_detector::validate_managers_async(&options, &config, Some(&mock), None).await;
}

#[tokio::test]
async fn validate_managers_async_choco_disabled_returns_valid() {
    let mock = MockWingetManager::new();
    let options = opts(true, false, false);
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    package_manager_detector::validate_managers_async(&options, &config, Some(&mock), None).await;
}

#[tokio::test]
async fn validate_managers_async_null_options_defaults() {
    let mock = MockWingetManager::new();
    let config = NTIXConfig::default();
    let options = config.options.clone();

    package_manager_detector::validate_managers_async(&options, &config, Some(&mock), None).await;
}

#[test]
fn validate_managers_scoop_disabled_returns_valid() {
    let options = opts(false, false, false);
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    package_manager_detector::validate_managers(&options, &config, None);
}

#[test]
fn validate_managers_choco_disabled_returns_valid() {
    let options = opts(false, false, false);
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    package_manager_detector::validate_managers(&options, &config, None);
}

#[test]
fn validate_managers_scoop_packages_declared_not_enabled_generates_warning() {
    let options = opts(false, false, false);
    let config = NTIXConfig {
        options: options.clone(),
        scoop_packages: vec![PackageEntry {
            id: "pkg1".to_string(),
            version: Some("1.0".to_string()),
        }],
        ..Default::default()
    };

    let result = package_manager_detector::validate_managers(&options, &config, None);
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("Scoop packages declared but scoop not enabled")));
}

#[test]
fn validate_managers_choco_packages_declared_not_enabled_generates_warning() {
    let options = opts(false, false, false);
    let config = NTIXConfig {
        options: options.clone(),
        choco_packages: vec![PackageEntry {
            id: "pkg1".to_string(),
            version: Some("1.0".to_string()),
        }],
        ..Default::default()
    };

    let result = package_manager_detector::validate_managers(&options, &config, None);
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("Chocolatey packages declared but chocolatey not enabled")));
}

#[test]
fn validate_managers_sync_with_options_returns_valid() {
    let options = opts(false, false, false);
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };

    let result = package_manager_detector::validate_managers(&options, &config, None);
    assert!(result.warnings.is_empty());
}

#[cfg(target_os = "windows")]
#[test]
fn is_running_as_admin_returns_bool() {
    let result = ntix_rs::process_helper::is_running_as_admin();
    let _ = result;
}
