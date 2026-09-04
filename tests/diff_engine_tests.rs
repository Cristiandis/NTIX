use std::collections::HashMap;

use indicatif::ProgressBar;
use ntix_rs::diff::diff_engine::compute_diff;
use ntix_rs::models::diff_result::DiffResult;
use ntix_rs::models::installed_packages::{InstalledPackages, UpgradeInfo};
use ntix_rs::models::ntix_config::NTIXConfig;
use ntix_rs::models::options::{ChocoOptions, NTIXOptions, ScoopOptions, WingetOptions};
use ntix_rs::models::package_entry::PackageEntry;
use ntix_rs::models::state::State;

mod common;
use common::{MockCommandRunner, MockWingetManager};

fn progress() -> ProgressBar {
    ProgressBar::new_spinner()
}

fn pkg_entry(id: &str, version: Option<&str>) -> PackageEntry {
    PackageEntry {
        id: id.to_string(),
        version: version.map(|s| s.to_string()),
    }
}

fn ntix_config(winget: WingetOptions, choco: ChocoOptions, scoop: ScoopOptions) -> NTIXConfig {
    NTIXConfig {
        options: NTIXOptions {
            winget,
            chocolatey: choco,
            scoop,
        },
        ..Default::default()
    }
}

/// Computes a diff with common defaults:
/// no winget manager (unless given), no runner, adopt/upgrade/validate configurable.
async fn diff_with(
    config: &NTIXConfig,
    state: &State,
    installed: Option<&InstalledPackages>,
    winget_manager: Option<&MockWingetManager>,
    runner: Option<&MockCommandRunner>,
    validate_packages: bool,
    upgrade_mode: bool,
    adopt_mode: bool,
) -> DiffResult {
    compute_diff(
        config,
        state,
        winget_manager
            .map(|m| m as &dyn ntix_rs::package_manager::winget_manager_trait::WingetManagerTrait),
        Some(true),
        Some(true),
        runner.map(|r| r as &dyn ntix_rs::package_manager::command_runner::CommandRunner),
        adopt_mode,
        upgrade_mode,
        validate_packages,
        installed,
        &progress(),
    )
    .await
    .expect("compute_diff should not error")
}

fn winget_enabled() -> NTIXConfig {
    ntix_config(
        WingetOptions {
            enable: true,
            ..Default::default()
        },
        ChocoOptions::default(),
        ScoopOptions::default(),
    )
}

#[tokio::test]
async fn compute_diff_empty_config_and_state_returns_empty() {
    let config = NTIXConfig::default();
    let state = State::default();
    let diff = diff_with(&config, &state, None, None, None, true, false, false).await;
    assert!(diff.is_empty());
    assert!(diff.to_install.is_empty());
    assert!(diff.to_upgrade.is_empty());
    assert!(diff.to_skip.is_empty());
    assert!(diff.to_remove.is_empty());
}

#[tokio::test]
async fn compute_diff_package_in_config_not_in_state_to_install() {
    let mut mock = MockWingetManager::new();
    mock.package_exists_result = Some(true);
    let config = ntix_config(
        WingetOptions {
            enable: true,
            ..Default::default()
        },
        ChocoOptions::default(),
        ScoopOptions::default(),
    );
    let mut config = config;
    config.winget_packages = vec![pkg_entry("testpkg", None)];
    let state = State::default();
    let diff = diff_with(
        &config,
        &state,
        None,
        Some(&mock),
        None,
        false,
        false,
        false,
    )
    .await;
    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "testpkg");
}

#[tokio::test]
async fn compute_diff_package_in_state_not_in_config_to_remove() {
    let config = NTIXConfig::default();
    let mut state = State::default();
    state.winget.insert("oldpkg".to_string(), "1.0".to_string());
    let diff = diff_with(&config, &state, None, None, None, true, false, false).await;
    assert_eq!(diff.to_remove.len(), 1);
    assert_eq!(diff.to_remove[0].id, "oldpkg");
}

#[tokio::test]
async fn compute_diff_pinned_package_in_state_but_not_installed_to_install() {
    let mut mock = MockWingetManager::new();
    mock.package_exists_result = Some(true);
    let config = ntix_config(
        WingetOptions {
            enable: true,
            ..Default::default()
        },
        ChocoOptions::default(),
        ScoopOptions::default(),
    );
    let mut config = config;
    config.winget_packages = vec![pkg_entry("testpkg", Some("1.0"))];
    let mut state = State::default();
    state
        .winget
        .insert("testpkg".to_string(), "1.0".to_string());
    let diff: DiffResult =
        diff_with(&config, &state, None, Some(&mock), None, true, false, false).await;
    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "testpkg");
}

#[tokio::test]
async fn compute_diff_with_mock_winget_manager_uses_injected_manager() {
    let mut mock = MockWingetManager::new();
    mock.is_installed = true;
    mock.installed_packages
        .insert("mocked-pkg".to_string(), "1.0".to_string());
    mock.upgradable_packages
        .insert("mocked-pkg".to_string(), UpgradeInfo::new("1.0", "2.0"));

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("mocked-pkg", None)];
    let state = State::default();

    let diff = diff_with(&config, &state, None, Some(&mock), None, true, true, false).await;
    assert_eq!(diff.to_upgrade.len(), 1);
    assert_eq!(diff.to_upgrade[0].id, "mocked-pkg");
    assert_eq!(diff.to_upgrade[0].version, Some("2.0".to_string()));
}

#[tokio::test]
async fn compute_diff_chocolatey_pinned_version_in_state_and_not_in_state() {
    let mut installed = InstalledPackages::default();
    installed
        .chocolatey
        .insert("choco-in-state".to_string(), "1.0".to_string());

    let mut config = ntix_config(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    config.choco_packages = vec![
        pkg_entry("choco-in-state", Some("1.0")),
        pkg_entry("choco-not-in-state", Some("1.0")),
    ];

    let mut state = State::default();
    state
        .chocolatey
        .insert("choco-in-state".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        None,
        None,
        false,
        false,
        false,
    )
    .await;
    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(diff.to_skip[0].id, "choco-in-state");
    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "choco-not-in-state");
}

#[tokio::test]
async fn compute_diff_scoop_pinned_version_in_state_and_not_in_state() {
    let mut installed = InstalledPackages::default();
    installed
        .scoop
        .insert("scoop-in-state".to_string(), "1.0".to_string());

    let mut config = ntix_config(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            ..Default::default()
        },
    );
    config.scoop_packages = vec![
        pkg_entry("scoop-in-state", Some("1.0")),
        pkg_entry("scoop-not-in-state", Some("1.0")),
    ];

    let mut state = State::default();
    state
        .scoop
        .insert("scoop-in-state".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        None,
        None,
        false,
        false,
        false,
    )
    .await;
    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(diff.to_skip[0].id, "scoop-in-state");
    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "scoop-not-in-state");
}

#[tokio::test]
async fn compute_diff_unpinned_pkg_with_upgrade_to_upgrade() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("upgradable-pkg".to_string(), "1.0".to_string());
    mock.upgradable_packages
        .insert("upgradable-pkg".to_string(), UpgradeInfo::new("1.0", "2.0"));

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("upgradable-pkg", None)];
    let mut state = State::default();
    state
        .winget
        .insert("upgradable-pkg".to_string(), "1.0".to_string());

    let diff = diff_with(&config, &state, None, Some(&mock), None, true, true, false).await;

    assert_eq!(diff.to_upgrade.len(), 1);
    assert_eq!(diff.to_upgrade[0].id, "upgradable-pkg");
    assert_eq!(diff.to_upgrade[0].version, Some("2.0".to_string()));
    assert!(diff.to_skip.is_empty());
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_unpinned_pkg_installed_no_upgrade_to_skip() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("current-pkg".to_string(), "1.0".to_string());

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("current-pkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("current-pkg", None)];
    let mut state = State::default();
    state
        .winget
        .insert("current-pkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(diff.to_skip[0].id, "current-pkg");
    assert!(diff.to_upgrade.is_empty());
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_no_upgrade_flag_upgradable_pkg_to_skip() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("upgradable-pkg".to_string(), "1.0".to_string());
    mock.upgradable_packages
        .insert("upgradable-pkg".to_string(), UpgradeInfo::new("1.0", "2.0"));

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("upgradable-pkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("upgradable-pkg", None)];
    let mut state = State::default();
    state
        .winget
        .insert("upgradable-pkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(diff.to_skip[0].id, "upgradable-pkg");
    assert!(diff.to_upgrade.is_empty());
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_unpinned_pkg_not_installed_not_in_state_to_install() {
    let mut mock = MockWingetManager::new();
    mock.package_exists_result = Some(true);
    let installed = InstalledPackages::default();

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("new-pkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "new-pkg");
    assert!(diff.to_upgrade.is_empty());
    assert!(diff.to_skip.is_empty());
}

#[tokio::test]
async fn compute_diff_unpinned_pkg_in_state_not_installed_to_install() {
    let mut mock = MockWingetManager::new();
    mock.package_exists_result = Some(true);
    let installed = InstalledPackages::default();

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("drifted-pkg", None)];
    let mut state = State::default();
    state
        .winget
        .insert("drifted-pkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "drifted-pkg");
}

#[tokio::test]
async fn compute_diff_unpinned_pkg_upgradable_but_not_installed_to_install() {
    let mut mock = MockWingetManager::new();
    mock.upgradable_packages
        .insert("upgradable-pkg".to_string(), UpgradeInfo::new("1.0", "2.0"));
    let installed = InstalledPackages::default();

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("upgradable-pkg", None)];
    let mut state = State::default();
    state
        .winget
        .insert("upgradable-pkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        true,
        false,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "upgradable-pkg");
    assert!(diff.to_upgrade.is_empty());
}

#[tokio::test]
async fn compute_diff_unpinned_pkg_id_case_insensitive_match_to_skip() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("foobar".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("FooBar", None)];
    let mut state = State::default();
    state.winget.insert("foobar".to_string(), "1.0".to_string());

    let diff = diff_with(&config, &state, None, Some(&mock), None, true, true, false).await;

    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(diff.to_skip[0].id, "FooBar");
    assert!(diff.to_install.is_empty());
    assert!(diff.to_upgrade.is_empty());
}

#[tokio::test]
async fn compute_diff_pinned_version_mismatch_to_install() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("mismatch-pkg".to_string(), "1.0".to_string());
    mock.package_exists_result = Some(true);

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("mismatch-pkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("mismatch-pkg", Some("2.0"))];
    let mut state = State::default();
    state
        .winget
        .insert("mismatch-pkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "mismatch-pkg");
    assert_eq!(diff.to_install[0].version, Some("2.0".to_string()));
    assert!(diff.to_skip.is_empty());
}

#[tokio::test]
async fn compute_diff_pinned_version_mismatch_case_insensitive_to_install() {
    let mock = MockWingetManager::new();
    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("case-pkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("case-pkg", Some("1.0"))];
    let mut state = State::default();
    state
        .winget
        .insert("case-pkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_skip.len(), 1);
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_disabled_manager_skips_packages() {
    let installed = InstalledPackages::default();
    let mut config = ntix_config(
        WingetOptions::default(),
        ChocoOptions {
            enable: false,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    config.choco_packages = vec![pkg_entry("choco-pkg", Some("1.0"))];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        None,
        None,
        true,
        false,
        false,
    )
    .await;
    assert!(diff.to_install.is_empty());
    assert!(diff.to_upgrade.is_empty());
    assert!(diff.to_skip.is_empty());
    assert!(diff.to_remove.is_empty());
}

#[tokio::test]
async fn compute_diff_multiple_managers_all_enabled() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("winget-current".to_string(), "1.0".to_string());
    mock.package_exists_result = Some(true);

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("winget-current".to_string(), "1.0".to_string());
    installed
        .chocolatey
        .insert("choco-installed".to_string(), "1.0".to_string());

    let mut config = ntix_config(
        WingetOptions {
            enable: true,
            ..Default::default()
        },
        ChocoOptions {
            enable: true,
            ..Default::default()
        },
        ScoopOptions {
            enable: true,
            ..Default::default()
        },
    );
    config.winget_packages = vec![
        pkg_entry("winget-current", None),
        pkg_entry("winget-new", None),
    ];
    config.choco_packages = vec![
        pkg_entry("choco-installed", Some("1.0")),
        pkg_entry("choco-new", Some("1.0")),
    ];
    config.scoop_packages = vec![pkg_entry("scoop-new", None)];

    let mut state = State::default();
    state
        .winget
        .insert("winget-current".to_string(), "1.0".to_string());
    state
        .chocolatey
        .insert("choco-installed".to_string(), "1.0".to_string());
    state
        .chocolatey
        .insert("choco-orphan".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        false,
        false,
        false,
    )
    .await;

    assert!(diff.to_skip.iter().any(|s| s.id == "winget-current"));
    assert!(
        diff.to_install
            .iter()
            .any(|s| s.id == "winget-new" && s.source == "winget")
    );
    assert!(diff.to_skip.iter().any(|s| s.id == "choco-installed"));
    assert!(
        diff.to_install
            .iter()
            .any(|s| s.id == "choco-new" && s.source == "chocolatey")
    );
    assert!(
        diff.to_install
            .iter()
            .any(|s| s.id == "scoop-new" && s.source == "scoop")
    );
    assert!(diff.to_remove.iter().any(|s| s.id == "choco-orphan"));
}

#[tokio::test]
async fn compute_diff_nonexistent_winget_package_becomes_warning() {
    let mut mock = MockWingetManager::new();
    mock.package_exists_result = None;
    mock.package_exists_error = None;

    let installed = InstalledPackages::default();
    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("real-pkg", None), pkg_entry("fake-pkg", None)];
    let state = State::default();

    // Configure per-id results: real exists, fake doesn't
    let mut exists_map = HashMap::new();
    exists_map.insert("real-pkg".to_string(), true);
    exists_map.insert("fake-pkg".to_string(), false);
    mock.package_exists_by_id = Some(exists_map);

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "real-pkg");
    assert!(diff.warnings.iter().any(|w| w.contains("fake-pkg")));
}

#[tokio::test]
async fn compute_diff_nonexistent_winget_package_removed_from_to_install() {
    let mut mock = MockWingetManager::new();
    let mut exists_map = HashMap::new();
    exists_map.insert("only-fake".to_string(), false);
    mock.package_exists_by_id = Some(exists_map);

    let installed = InstalledPackages::default();
    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("only-fake", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert!(diff.to_install.is_empty());
    assert!(diff.warnings.iter().any(|w| w.contains("only-fake")));
}

#[tokio::test]
async fn compute_diff_installed_package_not_validated() {
    let mock = MockWingetManager::new();
    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("existing-pkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("existing-pkg", None)];
    let mut state = State::default();
    state
        .winget
        .insert("existing-pkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(mock.package_exists_call_count(), 0);
}

#[tokio::test]
async fn compute_diff_winget_validation_throws_graceful_degradation() {
    let mut mock = MockWingetManager::new();
    let mut exists_map = HashMap::new();
    exists_map.insert("some-pkg".to_string(), true);
    mock.package_exists_by_id = Some(exists_map);
    mock.package_exists_throw = true;

    let installed = InstalledPackages::default();
    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("some-pkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "some-pkg");
    assert!(diff.warnings.iter().any(|w| w.contains("Could not verify")));
}

#[tokio::test]
async fn compute_diff_invalid_managers_returns_warning_in_result() {
    let config = ntix_config(
        WingetOptions::default(),
        ChocoOptions {
            enable: false,
            ..Default::default()
        },
        ScoopOptions {
            enable: false,
            ..Default::default()
        },
    );
    let mut config = config;
    config.choco_packages = vec![pkg_entry("pkg1", Some("1.0"))];
    let state = State::default();

    let diff = diff_with(&config, &state, None, None, None, true, false, false).await;
    assert!(
        diff.warnings
            .iter()
            .any(|w| w.contains("Chocolatey packages declared but chocolatey not enabled"))
    );
}

#[tokio::test]
async fn compute_diff_scoop_disabled_with_packages_generates_warning() {
    let config = ntix_config(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: false,
            ..Default::default()
        },
    );
    let mut config = config;
    config.scoop_packages = vec![pkg_entry("pkg1", Some("1.0"))];
    let state = State::default();

    let diff = diff_with(&config, &state, None, None, None, true, false, false).await;
    assert!(
        diff.warnings
            .iter()
            .any(|w| w.contains("Scoop packages declared but scoop not enabled"))
    );
}

#[tokio::test]
async fn compute_diff_known_package_skips_validation() {
    let mock = MockWingetManager::new();
    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("known-pkg", None)];
    let mut state = State::default();
    state
        .winget
        .insert("known-pkg".to_string(), "1.0".to_string());

    let diff = diff_with(&config, &state, None, Some(&mock), None, true, false, false).await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(mock.package_exists_call_count(), 0);
}

#[tokio::test]
async fn compute_diff_new_package_validates() {
    let mut mock = MockWingetManager::new();
    let mut exists_map = HashMap::new();
    exists_map.insert("new-pkg".to_string(), true);
    mock.package_exists_by_id = Some(exists_map);

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("new-pkg", None)];
    let state = State::default();

    let diff = diff_with(&config, &state, None, Some(&mock), None, true, false, false).await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(mock.package_exists_call_count(), 1);
}

#[tokio::test]
async fn compute_diff_adopt_mode_installed_not_in_state_to_adopt() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("manual-pkg".to_string(), "3.0".to_string());

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("manual-pkg".to_string(), "3.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("manual-pkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        true,
    )
    .await;

    assert_eq!(diff.to_adopt.len(), 1);
    assert_eq!(diff.to_adopt[0].id, "manual-pkg");
    assert!(diff.to_skip.is_empty());
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_no_adopt_mode_installed_not_in_state_is_untracked() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("manual-pkg".to_string(), "3.0".to_string());

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("manual-pkg".to_string(), "3.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("manual-pkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        false,
    )
    .await;

    assert!(diff.to_skip.is_empty(), "not actually managed");
    assert!(diff.to_adopt.is_empty());
    assert_eq!(diff.to_untracked.len(), 1);
    assert_eq!(diff.to_untracked[0].id, "manual-pkg");
}

#[tokio::test]
async fn compute_diff_adopt_mode_pinned_version_matches_to_adopt() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("pinned-pkg".to_string(), "1.0".to_string());

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("pinned-pkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("pinned-pkg", Some("1.0"))];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        true,
        false,
        true,
    )
    .await;

    assert_eq!(diff.to_adopt.len(), 1);
    assert_eq!(diff.to_adopt[0].id, "pinned-pkg");
    assert_eq!(diff.to_adopt[0].version, Some("1.0".to_string()));
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_adopt_mode_pinned_version_mismatch_to_install() {
    let mut mock = MockWingetManager::new();
    mock.installed_packages
        .insert("pinned-pkg".to_string(), "1.0".to_string());

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("pinned-pkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("pinned-pkg", Some("2.0"))];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&mock),
        None,
        false,
        false,
        true,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "pinned-pkg");
    assert_eq!(diff.to_install[0].version, Some("2.0".to_string()));
    assert!(diff.to_adopt.is_empty());
}

#[tokio::test]
async fn compute_diff_choco_new_pkg_validation_not_found_adds_warning() {
    let mock_runner = MockCommandRunner::new();
    let installed = InstalledPackages::default();
    let mut config = ntix_config(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    config.choco_packages = vec![pkg_entry("nonexistent-choco", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        None,
        Some(&mock_runner),
        true,
        false,
        false,
    )
    .await;
    assert!(
        diff.warnings
            .iter()
            .any(|w| w.contains("nonexistent-choco"))
    );
}

#[tokio::test]
async fn compute_diff_scoop_new_pkg_validation_not_found_adds_warning() {
    let mock_runner = MockCommandRunner::new();
    let installed = InstalledPackages::default();
    let mut config = ntix_config(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            ..Default::default()
        },
    );
    config.scoop_packages = vec![pkg_entry("nonexistent-scoop", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        None,
        Some(&mock_runner),
        true,
        false,
        false,
    )
    .await;
    assert!(
        diff.warnings
            .iter()
            .any(|w| w.contains("nonexistent-scoop"))
    );
}

#[tokio::test]
async fn compute_diff_with_progress_reports_steps() {
    let mock = MockWingetManager::new();
    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("test-pkg", None)];
    let state = State::default();

    compute_diff(
        &config,
        &state,
        Some(&mock),
        None,
        None,
        None,
        false,
        false,
        true,
        None,
        &progress(),
    )
    .await
    .expect("should not error");
}

fn temp_cfg_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ntix_diff_cfg_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cfg_entry(
    dest: &std::path::Path,
    src: &std::path::Path,
) -> ntix_rs::models::config_file::ConfigFileEntry {
    ntix_rs::models::config_file::ConfigFileEntry {
        dest: dest.to_path_buf(),
        src: src.to_path_buf(),
    }
}

#[test]
fn compute_config_files_diff_classifies_create_update_skip_orphan() {
    use ntix_rs::diff::diff_engine::compute_config_files_diff;
    use ntix_rs::models::ntix_config::NTIXConfig;

    let dir = temp_cfg_dir();
    let dest = dir.join("dest");

    let src_new = dir.join("new.conf");
    let src_upd = dir.join("upd.conf");
    std::fs::write(&src_new, "content-create").unwrap();
    std::fs::write(&src_upd, "content-v1").unwrap();

    let dest_new = dest.join("new.conf");
    let dest_upd = dest.join("upd.conf");
    let dest_same = dest.join("same.conf");
    let orphan = dest.join("orphan.conf");

    let mut state = State::default();
    // orphan tracked but not in config
    state
        .config_files
        .insert(orphan.to_string_lossy().to_string(), "x".into());
    // upd tracked with different hash
    state
        .config_files
        .insert(dest_upd.to_string_lossy().to_string(), "stale-hash".into());

    let mut config = NTIXConfig::default();
    config.config_files.push(cfg_entry(&dest_new, &src_new));
    config.config_files.push(cfg_entry(&dest_upd, &src_upd));
    // same source content as a pre-tracked dest
    let same_src = dir.join("same.conf");
    std::fs::write(&same_src, "content-same").unwrap();
    state.config_files.insert(
        dest_same.to_string_lossy().to_string(),
        ntix_rs::hash::sha256_hex(b"content-same"),
    );
    config.config_files.push(cfg_entry(&dest_same, &same_src));

    let mut diff = DiffResult::default();
    compute_config_files_diff(&mut diff, &config, &state);

    assert_eq!(diff.config_files_to_create.len(), 1);
    assert_eq!(diff.config_files_to_create[0].dest, dest_new);
    assert_eq!(diff.config_files_to_update.len(), 1);
    assert_eq!(diff.config_files_to_update[0].dest, dest_upd);
    let same_dest = dest_same.to_string_lossy();
    assert!(
        !diff
            .config_files_to_update
            .iter()
            .any(|e| e.dest.to_string_lossy() == same_dest)
            && !diff
                .config_files_to_create
                .iter()
                .any(|e| e.dest.to_string_lossy() == same_dest)
    );
    assert_eq!(
        diff.config_files_no_longer_managed,
        vec![orphan.to_string_lossy().to_string()]
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn compute_config_files_diff_unreadable_source_adds_warning() {
    use ntix_rs::diff::diff_engine::compute_config_files_diff;
    use ntix_rs::models::ntix_config::NTIXConfig;

    let dir = temp_cfg_dir();
    let dest = dir.join("dest").join("t.conf");

    let mut config = NTIXConfig::default();
    // source file does not exist on disk -> read fails -> warning
    let missing_src = dir.join("missing.conf");
    config.config_files.push(cfg_entry(&dest, &missing_src));

    let state = State::default();
    let mut diff = DiffResult::default();
    compute_config_files_diff(&mut diff, &config, &state);

    assert_eq!(diff.warnings.len(), 1);
    assert!(
        diff.warnings[0].contains("Could not read config file source"),
        "expected unreadable-source warning, got: {:?}",
        diff.warnings
    );
    assert!(diff.config_files_to_create.is_empty());

    std::fs::remove_dir_all(&dir).unwrap();
}
