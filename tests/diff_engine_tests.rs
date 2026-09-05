use indicatif::ProgressBar;
use ntix_rs::diff::diff_engine::compute_diff;
use ntix_rs::models::diff_result::DiffResult;
use ntix_rs::models::installed_packages::InstalledPackages;
use ntix_rs::models::ntix_config::NTIXConfig;
use ntix_rs::models::options::{ChocoOptions, NTIXOptions, ScoopOptions, WingetOptions};
use ntix_rs::models::package_entry::PackageEntry;
use ntix_rs::models::package_manager::PackageManager;
use ntix_rs::models::state::State;

mod common;
use common::{
    MockCommandRunner, unique_tag, winget_list_command, winget_list_table, winget_search_command,
};

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
/// no runner, adopt/upgrade/validate configurable.
async fn diff_with(
    config: &NTIXConfig,
    state: &State,
    installed: Option<&InstalledPackages>,
    runner: Option<&MockCommandRunner>,
    validate_packages: bool,
    upgrade_mode: bool,
    adopt_mode: bool,
) -> DiffResult {
    compute_diff(
        config,
        state,
        Some(true),
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
    let diff = diff_with(&config, &state, None, None, true, false, false).await;
    assert!(diff.is_empty());
    assert!(diff.to_install.is_empty());
    assert!(diff.to_upgrade.is_empty());
    assert!(diff.to_skip.is_empty());
    assert!(diff.to_remove.is_empty());
}

#[tokio::test]
async fn compute_diff_package_in_config_not_in_state_to_install() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_search_command("testpkg"),
        "testpkg  testpkg  1.0  winget".to_string(),
    );
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
    let diff = diff_with(&config, &state, None, Some(&runner), false, false, false).await;
    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "testpkg");
}

#[tokio::test]
async fn compute_diff_package_in_state_not_in_config_to_remove() {
    let config = NTIXConfig::default();
    let mut state = State::default();
    state.winget.insert("oldpkg".to_string(), "1.0".to_string());
    let diff = diff_with(&config, &state, None, None, true, false, false).await;
    assert_eq!(diff.to_remove.len(), 1);
    assert_eq!(diff.to_remove[0].id, "oldpkg");
}

#[tokio::test]
async fn compute_diff_pinned_package_in_state_but_not_installed_to_install() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_search_command("testpkg"),
        "testpkg  testpkg  1.0  winget".to_string(),
    );
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
        diff_with(&config, &state, None, Some(&runner), true, false, false).await;
    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "testpkg");
}

#[tokio::test]
async fn compute_diff_with_mock_winget_manager_uses_injected_manager() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("mpkg", "1.0", Some("2.0"))]),
    );

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("mpkg", None)];
    let state = State::default();

    let diff = diff_with(&config, &state, None, Some(&runner), true, true, false).await;
    assert_eq!(diff.to_upgrade.len(), 1);
    assert_eq!(diff.to_upgrade[0].id, "mpkg");
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

    let diff = diff_with(&config, &state, Some(&installed), None, false, false, false).await;
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

    let diff = diff_with(&config, &state, Some(&installed), None, false, false, false).await;
    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(diff.to_skip[0].id, "scoop-in-state");
    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "scoop-not-in-state");
}

#[tokio::test]
async fn compute_diff_unpinned_pkg_with_upgrade_to_upgrade() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("upkg", "1.0", Some("2.0"))]),
    );

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("upkg", None)];
    let mut state = State::default();
    state.winget.insert("upkg".to_string(), "1.0".to_string());

    let diff = diff_with(&config, &state, None, Some(&runner), true, true, false).await;

    assert_eq!(diff.to_upgrade.len(), 1);
    assert_eq!(diff.to_upgrade[0].id, "upkg");
    assert_eq!(diff.to_upgrade[0].version, Some("2.0".to_string()));
    assert!(diff.to_skip.is_empty());
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_unpinned_pkg_installed_no_upgrade_to_skip() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("cpkg", "1.0", None)]),
    );

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("cpkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("cpkg", None)];
    let mut state = State::default();
    state.winget.insert("cpkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(diff.to_skip[0].id, "cpkg");
    assert!(diff.to_upgrade.is_empty());
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_no_upgrade_flag_upgradable_pkg_to_skip() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("upkg", "1.0", Some("2.0"))]),
    );

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("upkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("upkg", None)];
    let mut state = State::default();
    state.winget.insert("upkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(diff.to_skip[0].id, "upkg");
    assert!(diff.to_upgrade.is_empty());
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_unpinned_pkg_not_installed_not_in_state_to_install() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_search_command("new-pkg"),
        "new-pkg  new-pkg  1.0  winget".to_string(),
    );
    let installed = InstalledPackages::default();

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("new-pkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
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
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_search_command("drifted-pkg"),
        "drifted-pkg  drifted-pkg  1.0  winget".to_string(),
    );
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
        Some(&runner),
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
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("upkg", "1.0", Some("2.0"))]),
    );
    let installed = InstalledPackages::default();

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("upkg", None)];
    let mut state = State::default();
    state.winget.insert("upkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        true,
        false,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "upkg");
    assert!(diff.to_upgrade.is_empty());
}

#[tokio::test]
async fn compute_diff_unpinned_pkg_id_case_insensitive_match_to_skip() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("fbar", "1.0", None)]),
    );

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("FBar", None)];
    let mut state = State::default();
    state.winget.insert("fbar".to_string(), "1.0".to_string());

    let diff = diff_with(&config, &state, None, Some(&runner), true, true, false).await;

    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(diff.to_skip[0].id, "FBar");
    assert!(diff.to_install.is_empty());
    assert!(diff.to_upgrade.is_empty());
}

#[tokio::test]
async fn compute_diff_pinned_version_mismatch_to_install() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("mpkg", "1.0", None)]),
    );
    runner.output_responses.insert(
        winget_search_command("mpkg"),
        "mpkg  mpkg  1.0  winget".to_string(),
    );

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("mpkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("mpkg", Some("2.0"))];
    let mut state = State::default();
    state.winget.insert("mpkg".to_string(), "1.0".to_string());

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "mpkg");
    assert_eq!(diff.to_install[0].version, Some("2.0".to_string()));
    assert!(diff.to_skip.is_empty());
}

#[tokio::test]
async fn compute_diff_pinned_version_mismatch_case_insensitive_to_install() {
    let runner = MockCommandRunner::new();
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
        Some(&runner),
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

    let diff = diff_with(&config, &state, Some(&installed), None, true, false, false).await;
    assert!(diff.to_install.is_empty());
    assert!(diff.to_upgrade.is_empty());
    assert!(diff.to_skip.is_empty());
    assert!(diff.to_remove.is_empty());
}

#[tokio::test]
async fn compute_diff_multiple_managers_all_enabled() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("wcur", "1.0", None)]),
    );

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("wcur".to_string(), "1.0".to_string());
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
    config.winget_packages = vec![pkg_entry("wcur", None), pkg_entry("wnew", None)];
    config.choco_packages = vec![
        pkg_entry("choco-installed", Some("1.0")),
        pkg_entry("choco-new", Some("1.0")),
    ];
    config.scoop_packages = vec![pkg_entry("scoop-new", None)];

    let mut state = State::default();
    state.winget.insert("wcur".to_string(), "1.0".to_string());
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
        Some(&runner),
        false,
        false,
        false,
    )
    .await;

    assert!(diff.to_skip.iter().any(|s| s.id == "wcur"));
    assert!(
        diff.to_install
            .iter()
            .any(|s| s.id == "wnew" && s.source == PackageManager::Winget)
    );
    assert!(diff.to_skip.iter().any(|s| s.id == "choco-installed"));
    assert!(
        diff.to_install
            .iter()
            .any(|s| s.id == "choco-new" && s.source == PackageManager::Chocolatey)
    );
    assert!(
        diff.to_install
            .iter()
            .any(|s| s.id == "scoop-new" && s.source == PackageManager::Scoop)
    );
    assert!(diff.to_remove.iter().any(|s| s.id == "choco-orphan"));
}

#[tokio::test]
async fn compute_diff_nonexistent_winget_package_becomes_warning() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_search_command("real-pkg"),
        "real-pkg  real-pkg  1.0  winget".to_string(),
    );
    runner.output_responses.insert(
        winget_search_command("fake-pkg"),
        "No package found matching input".to_string(),
    );

    let installed = InstalledPackages::default();
    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("real-pkg", None), pkg_entry("fake-pkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
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
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_search_command("only-fake"),
        "No package found matching input".to_string(),
    );

    let installed = InstalledPackages::default();
    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("only-fake", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert!(diff.to_install.is_empty());
    assert!(diff.warnings.iter().any(|w| w.contains("only-fake")));
}

#[tokio::test]
async fn compute_diff_choco_verification_failure_warns_and_keeps_install() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "choco search".to_string(),
        "ERROR: Failed to access the source".to_string(),
    );

    let installed = InstalledPackages::default();
    let mut config = ntix_config(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    config.choco_packages = vec![pkg_entry("choco-pkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "choco-pkg");
    assert!(
        diff.warnings
            .iter()
            .any(|w| w.contains("Could not verify package in chocolatey"))
    );
}

#[tokio::test]
async fn compute_diff_choco_not_found_removes_from_to_install() {
    let mut runner = MockCommandRunner::new();
    runner
        .output_responses
        .insert("choco search".to_string(), "".to_string());

    let installed = InstalledPackages::default();
    let mut config = ntix_config(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    config.choco_packages = vec![pkg_entry("fake-choco", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert!(diff.to_install.is_empty());
    assert!(
        diff.warnings
            .iter()
            .any(|w| w.contains("Package not found in chocolatey"))
    );
}

#[tokio::test]
async fn compute_diff_scoop_verification_failure_warns_and_keeps_install() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop info".to_string(),
        "Error: could not reach the remote".to_string(),
    );

    let installed = InstalledPackages::default();
    let mut config = ntix_config(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            ..Default::default()
        },
    );
    config.scoop_packages = vec![pkg_entry("scoop-pkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "scoop-pkg");
    assert!(
        diff.warnings
            .iter()
            .any(|w| w.contains("Could not verify package in scoop"))
    );
}

#[tokio::test]
async fn compute_diff_scoop_not_found_removes_from_to_install() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop info".to_string(),
        "Couldn't find manifest for 'fake-scoop'".to_string(),
    );

    let installed = InstalledPackages::default();
    let mut config = ntix_config(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            ..Default::default()
        },
    );
    config.scoop_packages = vec![pkg_entry("fake-scoop", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert!(diff.to_install.is_empty());
    assert!(
        diff.warnings
            .iter()
            .any(|w| w.contains("Package not found in scoop"))
    );
}

#[tokio::test]
async fn compute_diff_installed_package_not_validated() {
    let runner = MockCommandRunner::new();
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
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert_eq!(diff.to_skip.len(), 1);
    assert_eq!(
        runner
            .commands()
            .iter()
            .filter(|c| c.contains("winget search"))
            .count(),
        0
    );
}

#[tokio::test]
async fn compute_diff_winget_validation_throws_graceful_degradation() {
    let runner = MockCommandRunner::new();

    let installed = InstalledPackages::default();
    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("some-pkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert!(diff.to_install.is_empty());
    assert!(
        diff.warnings
            .iter()
            .any(|w| w.contains("Package not found"))
    );
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

    let diff = diff_with(&config, &state, None, None, true, false, false).await;
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

    let diff = diff_with(&config, &state, None, None, true, false, false).await;
    assert!(
        diff.warnings
            .iter()
            .any(|w| w.contains("Scoop packages declared but scoop not enabled"))
    );
}

#[tokio::test]
async fn compute_diff_known_package_skips_validation() {
    let runner = MockCommandRunner::new();
    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("known-pkg", None)];
    let mut state = State::default();
    state
        .winget
        .insert("known-pkg".to_string(), "1.0".to_string());

    let diff = diff_with(&config, &state, None, Some(&runner), true, false, false).await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(
        runner
            .commands()
            .iter()
            .filter(|c| c.contains("winget search"))
            .count(),
        0
    );
}

#[tokio::test]
async fn compute_diff_new_package_validates() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_search_command("new-pkg"),
        "new-pkg  new-pkg  1.0  winget".to_string(),
    );

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("new-pkg", None)];
    let state = State::default();

    let diff = diff_with(&config, &state, None, Some(&runner), true, false, false).await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(
        runner
            .commands()
            .iter()
            .filter(|c| c.contains("winget search"))
            .count(),
        1
    );
}

#[tokio::test]
async fn compute_diff_adopt_mode_installed_not_in_state_to_adopt() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("mpkg", "3.0", None)]),
    );

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("mpkg".to_string(), "3.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("mpkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        true,
    )
    .await;

    assert_eq!(diff.to_adopt.len(), 1);
    assert_eq!(diff.to_adopt[0].id, "mpkg");
    assert!(diff.to_skip.is_empty());
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_no_adopt_mode_installed_not_in_state_is_untracked() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("mpkg", "3.0", None)]),
    );

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("mpkg".to_string(), "3.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("mpkg", None)];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        false,
    )
    .await;

    assert!(diff.to_skip.is_empty(), "not actually managed");
    assert!(diff.to_adopt.is_empty());
    assert_eq!(diff.to_untracked.len(), 1);
    assert_eq!(diff.to_untracked[0].id, "mpkg");
}

#[tokio::test]
async fn compute_diff_adopt_mode_pinned_version_matches_to_adopt() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("ppkg", "1.0", None)]),
    );

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("ppkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("ppkg", Some("1.0"))];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        true,
        false,
        true,
    )
    .await;

    assert_eq!(diff.to_adopt.len(), 1);
    assert_eq!(diff.to_adopt[0].id, "ppkg");
    assert_eq!(diff.to_adopt[0].version, Some("1.0".to_string()));
    assert!(diff.to_install.is_empty());
}

#[tokio::test]
async fn compute_diff_adopt_mode_pinned_version_mismatch_to_install() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("ppkg", "1.0", None)]),
    );

    let mut installed = InstalledPackages::default();
    installed
        .winget
        .insert("ppkg".to_string(), "1.0".to_string());

    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("ppkg", Some("2.0"))];
    let state = State::default();

    let diff = diff_with(
        &config,
        &state,
        Some(&installed),
        Some(&runner),
        false,
        false,
        true,
    )
    .await;

    assert_eq!(diff.to_install.len(), 1);
    assert_eq!(diff.to_install[0].id, "ppkg");
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
    let mut config = winget_enabled();
    config.winget_packages = vec![pkg_entry("test-pkg", None)];
    let state = State::default();

    compute_diff(
        &config,
        &state,
        Some(true),
        Some(true),
        Some(true),
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
    let dir = std::env::temp_dir().join(unique_tag("diff_cfg"));
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
