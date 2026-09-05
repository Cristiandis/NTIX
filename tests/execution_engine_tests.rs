use std::fs;
use std::path::Path;
use std::path::PathBuf;

use ntix_rs::execution::execution_engine::apply_diff;
use ntix_rs::models::diff_result::DiffResult;
use ntix_rs::models::manager_validation::ValidationResult;
use ntix_rs::models::ntix_config::NTIXConfig;
use ntix_rs::models::options::{
    ChocoOptions, NTIXOptions, ScoopBucket, ScoopOptions, WingetOptions,
};
use ntix_rs::models::package_manager::PackageManager;
use ntix_rs::models::package_spec::PackageSpec;
use ntix_rs::models::state::State;

mod common;
use common::{MockCommandRunner, winget_list_table};

fn options(winget: WingetOptions, choco: ChocoOptions, scoop: ScoopOptions) -> NTIXOptions {
    NTIXOptions {
        winget,
        chocolatey: choco,
        scoop,
    }
}

fn winget_opts() -> NTIXOptions {
    options(
        WingetOptions {
            enable: true,
            accept_agreement: true,
            silent: false,
            disable_interactivity: true,
        },
        ChocoOptions::default(),
        ScoopOptions::default(),
    )
}

fn empty_opts() -> NTIXOptions {
    options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions::default(),
    )
}

fn spec(id: &str, version: Option<&str>, source: &str) -> PackageSpec {
    PackageSpec {
        id: id.to_string(),
        version: version.map(|s| s.to_string()),
        source: PackageManager::from_name(source).expect("valid manager source"),
    }
}

fn temp_state_path() -> PathBuf {
    let dir = std::env::temp_dir().join(common::unique_tag("exec"));
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join("state.json");
    if p.exists() {
        fs::remove_file(&p).unwrap();
    }
    p
}

fn remove_path(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

async fn apply(
    diff: &DiffResult,
    options: &NTIXOptions,
    state: &mut State,
    path: &Path,
    stop_on_failure: bool,
    config: Option<&NTIXConfig>,
    runner: Option<&MockCommandRunner>,
) -> bool {
    let validation = ValidationResult {
        winget_installed: true,
        choco_installed: true,
        scoop_installed: true,
        warnings: validation_warnings(config, options),
    };
    apply_diff(
        diff,
        options,
        state,
        path,
        stop_on_failure,
        &validation,
        false,
        None,
        None,
        runner.map(|r| r as &dyn ntix_rs::package_manager::command_runner::CommandRunner),
    )
    .await
}

/// Mirrors the warnings `package_manager_detector` would raise for a
/// fully-installed manager setup, given the config's packaged declarations.
fn validation_warnings(config: Option<&NTIXConfig>, options: &NTIXOptions) -> Vec<String> {
    let Some(config) = config else {
        return Vec::new();
    };
    let mut warnings = Vec::new();
    if !config.choco_packages.is_empty() && !options.chocolatey.enable {
        warnings.push(
            "[warn] Chocolatey packages declared but chocolatey not enabled in options".to_string(),
        );
    }
    if !config.scoop_packages.is_empty() && !options.scoop.enable {
        warnings
            .push("[warn] Scoop packages declared but scoop not enabled in options".to_string());
    }
    if !config.winget_packages.is_empty() && !options.winget.enable {
        warnings
            .push("[warn] Winget packages declared but winget not enabled in options".to_string());
    }
    warnings
}

#[tokio::test]
async fn apply_diff_empty_diff_returns_true() {
    let diff = DiffResult::default();
    let options = empty_opts();
    let mut state = State::default();
    let path = temp_state_path();
    let result = apply(&diff, &options, &mut state, &path, false, None, None).await;
    assert!(result);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_winget_install_uses_mock_runner() {
    let runner = MockCommandRunner::new();

    let diff = DiffResult {
        to_install: vec![spec("test-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.winget.get("test-pkg"), Some(&"1.0".to_string()));
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("winget install") && c.contains("--id test-pkg"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_winget_upgrade_uses_mock_runner() {
    let runner = MockCommandRunner::new();

    let diff = DiffResult {
        to_upgrade: vec![spec("test-pkg", Some("2.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    state
        .winget
        .insert("test-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.winget.get("test-pkg"), Some(&"2.0".to_string()));
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("winget upgrade") && c.contains("--id test-pkg"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_winget_uninstall_uses_mock_runner() {
    let runner = MockCommandRunner::new();

    let diff = DiffResult {
        to_remove: vec![spec("test-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    state
        .winget
        .insert("test-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("winget uninstall") && c.contains("--id test-pkg"))
    );
    assert!(!state.winget.contains_key("test-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_mixed_sources_works_correctly() {
    let runner = MockCommandRunner::new();

    let diff = DiffResult {
        to_install: vec![spec("winget-pkg", Some("1.0"), "winget")],
        to_upgrade: vec![spec("winget-upgrade", Some("2.0"), "winget")],
        to_remove: vec![spec("winget-remove", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = options(
        WingetOptions {
            enable: true,
            accept_agreement: true,
            silent: false,
            disable_interactivity: true,
        },
        ChocoOptions {
            enable: true,
            yes: true,
            ..Default::default()
        },
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    state
        .winget
        .insert("winget-upgrade".to_string(), "1.0".to_string());
    state
        .winget
        .insert("winget-remove".to_string(), "1.0".to_string());
    state
        .chocolatey
        .insert("choco-pkg".to_string(), "1.0".to_string());
    state
        .scoop
        .insert("scoop-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.winget.get("winget-pkg"), Some(&"1.0".to_string()));
    assert_eq!(state.winget.get("winget-upgrade"), Some(&"2.0".to_string()));
    assert!(!state.winget.contains_key("winget-remove"));
    assert!(state.chocolatey.contains_key("choco-pkg"));
    assert!(state.scoop.contains_key("scoop-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_winget_install_failure_sets_all_ok_false() {
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() =
        Some(Box::new(
            |cmd: &str| {
                if cmd.contains("winget") { 1 } else { 0 }
            },
        ));

    let diff = DiffResult {
        to_install: vec![spec("fail-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    assert!(!state.winget.contains_key("fail-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_winget_install_already_installed_counts_as_success() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "winget list".to_string(),
        winget_list_table(&[("test-pkg", "1.0", None)]),
    );
    *runner.run_handler.lock().unwrap() = Some(Box::new(|cmd: &str| {
        if cmd.starts_with("winget install") {
            1
        } else {
            0
        }
    }));

    let diff = DiffResult {
        to_install: vec![spec("test-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.winget.get("test-pkg"), Some(&"1.0".to_string()));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_winget_remove_already_gone_counts_as_success() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "winget list".to_string(),
        winget_list_table(&[("other-pkg", "1.0", None)]),
    );
    *runner.run_handler.lock().unwrap() = Some(Box::new(|cmd: &str| {
        if cmd.starts_with("winget uninstall") {
            1
        } else {
            0
        }
    }));

    let diff = DiffResult {
        to_remove: vec![spec("gone-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    state
        .winget
        .insert("gone-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(!state.winget.contains_key("gone-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_remove_already_gone_counts_as_success() {
    let mut runner = MockCommandRunner::new();
    runner
        .output_responses
        .insert("scoop list".to_string(), "other-pkg 1.0\n".to_string());
    *runner.run_handler.lock().unwrap() = Some(Box::new(|cmd: &str| {
        if cmd.starts_with("scoop uninstall") {
            1
        } else {
            0
        }
    }));

    let diff = DiffResult {
        to_remove: vec![spec("gone-pkg", Some("1.0"), "scoop")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            ..Default::default()
        },
    );
    let mut state = State::default();
    state
        .scoop
        .insert("gone-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(!state.scoop.contains_key("gone-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_stop_on_false_continues_after_failure() {
    let diff = DiffResult {
        to_install: vec![
            spec("fail-pkg", Some("1.0"), "winget"),
            spec("ok-pkg", Some("2.0"), "winget"),
        ],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() =
        Some(Box::new(
            |cmd: &str| {
                if cmd.contains("fail-pkg") { 1 } else { 0 }
            },
        ));

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    assert_eq!(state.winget.get("ok-pkg"), Some(&"2.0".to_string()));
    assert_eq!(runner.commands().len(), 3);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_stop_on_true_returns_early_on_failure() {
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() =
        Some(Box::new(
            |cmd: &str| {
                if cmd.contains("fail-pkg") { 1 } else { 0 }
            },
        ));

    let diff = DiffResult {
        to_install: vec![
            spec("fail-pkg", Some("1.0"), "winget"),
            spec("ok-pkg", Some("2.0"), "winget"),
        ],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        true,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    assert!(!state.winget.contains_key("ok-pkg"));
    assert_eq!(runner.commands().len(), 2);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_disabled_source_skips_package() {
    let runner = MockCommandRunner::new();

    let diff = DiffResult {
        to_install: vec![spec("test-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = options(
        WingetOptions {
            enable: false,
            ..Default::default()
        },
        ChocoOptions::default(),
        ScoopOptions::default(),
    );
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(state.winget.is_empty());
    assert!(runner.commands().is_empty());
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_null_version_records_latest() {
    let runner = MockCommandRunner::new();

    let diff = DiffResult {
        to_install: vec![spec("test-pkg", None, "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.winget.get("test-pkg"), Some(&"latest".to_string()));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_diff_with_warnings_still_processes() {
    let runner = MockCommandRunner::new();

    let diff = DiffResult {
        to_install: vec![spec("test-pkg", Some("1.0"), "winget")],
        warnings: vec!["some warning".to_string()],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(state.winget.contains_key("test-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_config_missing_choco_warns_and_continues() {
    let diff = DiffResult {
        to_install: vec![spec("test-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = options(
        WingetOptions {
            enable: true,
            ..Default::default()
        },
        ChocoOptions {
            enable: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    let _config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };
    let mut state = State::default();
    let path = temp_state_path();

    let warnings: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = warnings.clone();

    // Choco enabled but not installed -> warning, processing continues.
    let mock_runner = MockCommandRunner::new();
    let result = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult {
            winget_installed: true,
            choco_installed: false,
            scoop_installed: false,
            warnings: vec![
                "[warn] Chocolatey is enabled but not installed. Skipping Chocolatey packages. Install from https://chocolatey.org/install".to_string(),
            ],
        },
        false,
        None,
        Some(&|msg: &str| captured.lock().unwrap().push(msg.to_string())),
        Some(&mock_runner),
    )
    .await;
    assert!(result);
    assert!(state.winget.contains_key("test-pkg"));
    let warnings = warnings.lock().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("Chocolatey is enabled but not installed"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_choco_disabled_skips_choco_package() {
    let diff = DiffResult {
        to_install: vec![spec("choco-pkg", Some("1.0"), "chocolatey")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions {
            enable: false,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, None, None).await;
    assert!(result);
    assert!(state.chocolatey.is_empty());
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_disabled_skips_scoop_package() {
    let diff = DiffResult {
        to_install: vec![spec("scoop-pkg", Some("1.0"), "scoop")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: false,
            ..Default::default()
        },
    );
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, None, None).await;
    assert!(result);
    assert!(state.scoop.is_empty());
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_config_missing_scoop_warns_and_continues() {
    let diff = DiffResult {
        to_install: vec![spec("test-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            ..Default::default()
        },
    );
    let _config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };
    let mut state = State::default();
    let path = temp_state_path();

    let warnings: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = warnings.clone();

    // Scoop enabled but not installed -> warning, processing continues.
    let result = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult {
            winget_installed: true,
            choco_installed: false,
            scoop_installed: false,
            warnings: vec![
                "[warn] Scoop is enabled but not installed. Skipping Scoop packages and buckets. Install from https://scoop.sh".to_string(),
            ],
        },
        false,
        None,
        Some(&|msg: &str| captured.lock().unwrap().push(msg.to_string())),
        None,
    )
    .await;
    assert!(result);
    let warnings = warnings.lock().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("Scoop is enabled but not installed"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_config_validation_with_warnings_still_processes() {
    let diff = DiffResult {
        to_install: vec![spec("winget-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = options(
        WingetOptions {
            enable: true,
            accept_agreement: true,
            silent: false,
            disable_interactivity: true,
        },
        ChocoOptions {
            enable: false,
            ..Default::default()
        },
        ScoopOptions {
            enable: false,
            ..Default::default()
        },
    );
    let config = NTIXConfig {
        options: options.clone(),
        choco_packages: vec![ntix_rs::models::package_entry::PackageEntry {
            id: "choco-declared".to_string(),
            version: Some("1.0".to_string()),
        }],
        scoop_packages: vec![ntix_rs::models::package_entry::PackageEntry {
            id: "scoop-declared".to_string(),
            version: Some("1.0".to_string()),
        }],
        ..Default::default()
    };
    let mut state = State::default();
    let path = temp_state_path();

    let runner = MockCommandRunner::new();
    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        Some(&config),
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(state.winget.contains_key("winget-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_to_adopt_updates_state_without_install() {
    let runner = MockCommandRunner::new();

    let diff = DiffResult {
        to_adopt: vec![spec("manual-pkg", Some("3.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.winget.get("manual-pkg"), Some(&"3.0".to_string()));
    assert!(
        !runner
            .commands()
            .iter()
            .any(|c| c.contains("winget install"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_to_adopt_null_version_records_latest() {
    let runner = MockCommandRunner::new();

    let diff = DiffResult {
        to_adopt: vec![spec("manual-pkg", None, "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.winget.get("manual-pkg"), Some(&"latest".to_string()));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_choco_install_success() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        to_install: vec![spec("choco-pkg", Some("1.0"), "chocolatey")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            yes: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.chocolatey.get("choco-pkg"), Some(&"1.0".to_string()));
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("choco install"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_install_success() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        to_install: vec![spec("scoop-pkg", Some("2.0"), "scoop")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.scoop.get("scoop-pkg"), Some(&"2.0".to_string()));
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("scoop install"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_choco_upgrade_success() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        to_upgrade: vec![spec("choco-pkg", Some("2.0"), "chocolatey")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            yes: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    let mut state = State::default();
    state
        .chocolatey
        .insert("choco-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.chocolatey.get("choco-pkg"), Some(&"2.0".to_string()));
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("choco upgrade"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_upgrade_success() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        to_upgrade: vec![spec("scoop-pkg", Some("3.0"), "scoop")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    state
        .scoop
        .insert("scoop-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert_eq!(state.scoop.get("scoop-pkg"), Some(&"3.0".to_string()));
    assert!(runner.commands().iter().any(|c| c.contains("scoop update")));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_choco_remove_success() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        to_remove: vec![spec("choco-pkg", Some("1.0"), "chocolatey")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            yes: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    let mut state = State::default();
    state
        .chocolatey
        .insert("choco-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(!state.chocolatey.contains_key("choco-pkg"));
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("choco uninstall"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_remove_success() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        to_remove: vec![spec("scoop-pkg", Some("1.0"), "scoop")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    state
        .scoop
        .insert("scoop-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(!state.scoop.contains_key("scoop-pkg"));
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("scoop uninstall"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_choco_upgrade_failure_stop_on_failure() {
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() = Some(Box::new(|_: &str| 1));
    let diff = DiffResult {
        to_upgrade: vec![spec("choco-pkg", Some("2.0"), "chocolatey")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            yes: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    let mut state = State::default();
    state
        .chocolatey
        .insert("choco-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        true,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_choco_upgrade_failure_continues_on_failure() {
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() = Some(Box::new(move |_: &str| {
        let n = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if n == 0 { 1 } else { 0 }
    }));

    let diff = DiffResult {
        to_upgrade: vec![
            spec("choco-fail", Some("2.0"), "chocolatey"),
            spec("choco-ok", Some("1.0"), "chocolatey"),
        ],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            yes: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    let mut state = State::default();
    state
        .chocolatey
        .insert("choco-fail".to_string(), "1.0".to_string());
    state
        .chocolatey
        .insert("choco-ok".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    assert_eq!(state.chocolatey.get("choco-ok"), Some(&"1.0".to_string()));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_remove_failure_stop_on_failure() {
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() = Some(Box::new(|_: &str| 1));
    let diff = DiffResult {
        to_remove: vec![spec("choco-pkg", Some("1.0"), "chocolatey")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            yes: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    let mut state = State::default();
    state
        .chocolatey
        .insert("choco-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        true,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    assert!(state.chocolatey.contains_key("choco-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_remove_failure_continues_on_failure() {
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() = Some(Box::new(move |_: &str| {
        let n = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if n == 0 { 1 } else { 0 }
    }));

    let diff = DiffResult {
        to_remove: vec![
            spec("choco-fail", Some("1.0"), "chocolatey"),
            spec("choco-ok", Some("1.0"), "chocolatey"),
        ],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            yes: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    let mut state = State::default();
    state
        .chocolatey
        .insert("choco-fail".to_string(), "1.0".to_string());
    state
        .chocolatey
        .insert("choco-ok".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    assert!(!state.chocolatey.contains_key("choco-ok"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_install_failure_stop_on_failure() {
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() = Some(Box::new(|_: &str| 1));
    let diff = DiffResult {
        to_install: vec![spec("scoop-pkg", Some("1.0"), "scoop")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        true,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    assert!(!state.scoop.contains_key("scoop-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_upgrade_failure_continues_on_failure() {
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() = Some(Box::new(move |_: &str| {
        let n = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if n == 0 { 1 } else { 0 }
    }));

    let diff = DiffResult {
        to_upgrade: vec![
            spec("scoop-fail", Some("2.0"), "scoop"),
            spec("scoop-ok", Some("1.0"), "scoop"),
        ],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    state
        .scoop
        .insert("scoop-fail".to_string(), "1.0".to_string());
    state
        .scoop
        .insert("scoop-ok".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    assert_eq!(state.scoop.get("scoop-ok"), Some(&"1.0".to_string()));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_buckets_already_added_skips() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        to_install: vec![spec("scoop-pkg", Some("1.0"), "scoop")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(
        !runner
            .commands()
            .iter()
            .any(|c| c.contains("scoop bucket add"))
    );
    assert!(
        !runner
            .commands()
            .iter()
            .any(|c| c.contains("scoop bucket rm"))
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_buckets_add_success_records_in_state() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        buckets_to_add: vec![ScoopBucket::new("extras")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main"), ScoopBucket::new("extras")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("scoop bucket add extras"))
    );
    assert!(state.scoop_buckets.contains_key("extras"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_buckets_add_fails_reports_error() {
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() = Some(Box::new(|cmd: &str| {
        if cmd.contains("scoop bucket add") {
            1
        } else {
            0
        }
    }));
    let diff = DiffResult {
        buckets_to_add: vec![ScoopBucket::new("extras")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main"), ScoopBucket::new("extras")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    assert!(!state.scoop_buckets.contains_key("extras"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_buckets_remove_orphans() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        buckets_to_remove: vec![ScoopBucket::new("extras"), ScoopBucket::new("versions")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    state.scoop_buckets.insert("extras".to_string(), None);
    state.scoop_buckets.insert("versions".to_string(), None);
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(result);
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("scoop bucket rm extras"))
    );
    assert!(
        runner
            .commands()
            .iter()
            .any(|c| c.contains("scoop bucket rm versions"))
    );
    assert!(!state.scoop_buckets.contains_key("extras"));
    assert!(!state.scoop_buckets.contains_key("versions"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_scoop_buckets_remove_fails_reports_error() {
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() = Some(Box::new(|cmd: &str| {
        if cmd.contains("scoop bucket rm") {
            1
        } else {
            0
        }
    }));
    let diff = DiffResult {
        buckets_to_remove: vec![ScoopBucket::new("extras")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions::default(),
        ScoopOptions {
            enable: true,
            buckets: vec![ScoopBucket::new("main")],
            ..Default::default()
        },
    );
    let mut state = State::default();
    state.scoop_buckets.insert("extras".to_string(), None);
    let path = temp_state_path();

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        None,
        Some(&runner),
    )
    .await;
    assert!(!result);
    assert!(state.scoop_buckets.contains_key("extras"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_on_output_called_for_install() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        to_install: vec![spec("test-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let output_messages: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let msgs = output_messages.clone();

    let result = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult {
            winget_installed: true,
            choco_installed: false,
            scoop_installed: false,
            warnings: Vec::new(),
        },
        false,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        None,
        Some(&runner),
    )
    .await;

    assert!(result);
    let msgs = output_messages.lock().unwrap();
    assert!(msgs.iter().any(|m| m.contains("Installing")));
    assert!(msgs.iter().any(|m| m.contains("test-pkg")));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_on_error_called_for_failure() {
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() =
        Some(Box::new(
            |cmd: &str| {
                if cmd.contains("winget") { 1 } else { 0 }
            },
        ));
    let diff = DiffResult {
        to_install: vec![spec("fail-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let error_messages: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let msgs = error_messages.clone();

    let result = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult {
            winget_installed: true,
            choco_installed: false,
            scoop_installed: false,
            warnings: Vec::new(),
        },
        false,
        None,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        Some(&runner),
    )
    .await;

    assert!(!result);
    let msgs = error_messages.lock().unwrap();
    assert!(msgs.iter().any(|m| m.contains("Failed to install")));
    assert!(msgs.iter().any(|m| m.contains("fail-pkg")));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_choco_invalid_id_build_error_calls_on_error() {
    let diff = DiffResult {
        to_install: vec![spec("bad id", Some("1.0"), "chocolatey")],
        ..Default::default()
    };
    let options = options(
        WingetOptions::default(),
        ChocoOptions {
            enable: true,
            ..Default::default()
        },
        ScoopOptions::default(),
    );
    let mut state = State::default();
    let path = temp_state_path();

    let error_messages: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let msgs = error_messages.clone();

    let result = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult {
            winget_installed: false,
            choco_installed: true,
            scoop_installed: false,
            warnings: Vec::new(),
        },
        false,
        None,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        None,
    )
    .await;

    assert!(!result);
    assert!(!state.chocolatey.contains_key("bad id"));
    let msgs = error_messages.lock().unwrap();
    assert!(
        msgs.iter().any(|m| m.contains("Failed to build command")),
        "expected build-error callback, got: {msgs:?}"
    );
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_on_output_called_for_upgrade() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        to_upgrade: vec![spec("test-pkg", Some("2.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    state
        .winget
        .insert("test-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let output_messages: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let msgs = output_messages.clone();

    let result = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult {
            winget_installed: true,
            choco_installed: false,
            scoop_installed: false,
            warnings: Vec::new(),
        },
        false,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        None,
        Some(&runner),
    )
    .await;

    assert!(result);
    let msgs = output_messages.lock().unwrap();
    assert!(msgs.iter().any(|m| m.contains("Upgrading")));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_on_output_called_for_remove() {
    let runner = MockCommandRunner::new();
    let diff = DiffResult {
        to_remove: vec![spec("test-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    state
        .winget
        .insert("test-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let output_messages: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let msgs = output_messages.clone();

    let result = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult {
            winget_installed: true,
            choco_installed: false,
            scoop_installed: false,
            warnings: Vec::new(),
        },
        false,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        None,
        Some(&runner),
    )
    .await;

    assert!(result);
    let msgs = output_messages.lock().unwrap();
    assert!(msgs.iter().any(|m| m.contains("Removing")));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_apply_config_copies_files_and_updates_state() {
    let dir = std::env::temp_dir().join(common::unique_tag("exec_cfg"));
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("kitty.conf");
    std::fs::write(&src, "font_size 12").unwrap();
    let dest = dir.join("nested").join("dest").join("kitty.conf");

    let entry = ntix_rs::models::config_file::ConfigFileEntry {
        dest: dest.clone(),
        src: src.clone(),
    };
    let mut diff = DiffResult::default();
    diff.config_files_to_create.push(entry);

    let options = empty_opts();
    let mut state = State::default();
    let path = temp_state_path();
    let _config = NTIXConfig::default();

    let ok = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult::default(),
        true,
        None,
        None,
        None,
    )
    .await;
    assert!(ok);
    let bytes = std::fs::read(&dest).unwrap();
    assert_eq!(bytes, b"font_size 12");
    assert!(
        state
            .config_files
            .contains_key(&dest.to_string_lossy().to_string())
    );
    assert_eq!(
        state
            .config_files
            .get(&dest.to_string_lossy().to_string())
            .unwrap(),
        &ntix_rs::hash::sha256_hex(b"font_size 12")
    );

    // The applied config file must be persisted to the state file on disk.
    let persisted = ntix_rs::state_management::state_service::load_state(Some(&path))
        .expect("state should be saved to disk");
    assert_eq!(
        persisted
            .config_files
            .get(&dest.to_string_lossy().to_string())
            .map(|h| h.as_str()),
        Some(ntix_rs::hash::sha256_hex(b"font_size 12").as_str())
    );

    remove_path(&path);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn apply_diff_apply_config_missing_source_calls_on_error() {
    let dir = std::env::temp_dir().join(common::unique_tag("exec_cfg_missing"));
    std::fs::create_dir_all(&dir).unwrap();
    let missing_src = dir.join("does_not_exist.conf");
    let dest = dir.join("nested").join("dest").join("t.conf");

    let entry = ntix_rs::models::config_file::ConfigFileEntry {
        dest: dest.clone(),
        src: missing_src.clone(),
    };
    let mut diff = DiffResult::default();
    diff.config_files_to_create.push(entry);

    let options = empty_opts();
    let mut state = State::default();
    let path = temp_state_path();
    let _config = NTIXConfig::default();

    let error_messages: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let msgs = error_messages.clone();

    let ok = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult::default(),
        true,
        None,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        None,
    )
    .await;
    assert!(ok);
    let msgs = error_messages.lock().unwrap();
    assert!(
        msgs.iter()
            .any(|m| m.contains("Failed to read config file source")),
        "expected read-failure error, got: {msgs:?}"
    );
    assert!(
        !state
            .config_files
            .contains_key(&dest.to_string_lossy().to_string())
    );

    remove_path(&path);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn apply_diff_apply_config_write_failure_calls_on_error() {
    let dir = std::env::temp_dir().join(common::unique_tag("exec_cfg_write"));
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("src.conf");
    std::fs::write(&src, "content").unwrap();
    // dest is an existing directory -> fs::write fails
    let dest = dir.join("dest_dir");
    std::fs::create_dir_all(&dest).unwrap();

    let entry = ntix_rs::models::config_file::ConfigFileEntry {
        dest: dest.clone(),
        src: src.clone(),
    };
    let mut diff = DiffResult::default();
    diff.config_files_to_create.push(entry);

    let options = empty_opts();
    let mut state = State::default();
    let path = temp_state_path();
    let _config = NTIXConfig::default();

    let error_messages: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let msgs = error_messages.clone();

    let ok = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult::default(),
        true,
        None,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        None,
    )
    .await;
    assert!(ok);
    let msgs = error_messages.lock().unwrap();
    assert!(
        msgs.iter()
            .any(|m| m.contains("Failed to write config file")),
        "expected write-failure error, got: {msgs:?}"
    );
    assert!(
        !state
            .config_files
            .contains_key(&dest.to_string_lossy().to_string())
    );

    remove_path(&path);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn apply_diff_apply_config_drops_orphans_keeps_file() {
    let dir = std::env::temp_dir().join(common::unique_tag("exec_orphan"));
    std::fs::create_dir_all(&dir).unwrap();
    let orphan_file = dir.join("orphan.conf");
    std::fs::write(&orphan_file, "keep me").unwrap();

    let mut diff = DiffResult::default();
    diff.config_files_no_longer_managed
        .push(orphan_file.to_string_lossy().to_string());

    let options = empty_opts();
    let mut state = State::default();
    state
        .config_files
        .insert(orphan_file.to_string_lossy().to_string(), "hash".into());
    let path = temp_state_path();
    let _config = NTIXConfig::default();

    let ok = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        &ValidationResult::default(),
        true,
        None,
        None,
        None,
    )
    .await;
    assert!(ok);
    assert!(orphan_file.is_file(), "orphan file must remain on disk");
    assert!(state.config_files.is_empty());

    remove_path(&path);
    std::fs::remove_dir_all(&dir).unwrap();
}
