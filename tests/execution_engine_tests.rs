use std::fs;
use std::path::PathBuf;

use ntix_rs::execution::execution_engine::apply_diff;
use ntix_rs::models::diff_result::DiffResult;
use ntix_rs::models::ntix_config::NTIXConfig;
use ntix_rs::models::options::{ChocoOptions, NTIXOptions, ScoopBucket, ScoopOptions, WingetOptions};
use ntix_rs::models::package_spec::PackageSpec;
use ntix_rs::models::state::State;

mod common;
use common::{MockCommandRunner, MockManagerPresence, MockWingetManager};

fn options(
    winget: WingetOptions,
    choco: ChocoOptions,
    scoop: ScoopOptions,
) -> NTIXOptions {
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
            interactive: false,
        },
        ChocoOptions::default(),
        ScoopOptions::default(),
    )
}

fn empty_opts() -> NTIXOptions {
    options(WingetOptions::default(), ChocoOptions::default(), ScoopOptions::default())
}

fn spec(id: &str, version: Option<&str>, source: &str) -> PackageSpec {
    PackageSpec {
        id: id.to_string(),
        version: version.map(|s| s.to_string()),
        source: source.to_string(),
    }
}

fn temp_state_path() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ntix_exec_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join("state.json");
    if p.exists() {
        fs::remove_file(&p).unwrap();
    }
    p
}

fn remove_path(path: &PathBuf) {
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

async fn apply(
    diff: &DiffResult,
    options: &NTIXOptions,
    state: &mut State,
    path: &PathBuf,
    stop_on_failure: bool,
    winget_manager: Option<&MockWingetManager>,
    config: Option<&NTIXConfig>,
    runner: Option<&MockCommandRunner>,
) -> bool {
    apply_diff(
        diff,
        options,
        state,
        path,
        stop_on_failure,
        winget_manager.map(|m| m as &dyn ntix_rs::package_manager::winget_manager_trait::WingetManagerTrait),
        Some(&MockManagerPresence::new()),
        config,
        None,
        None,
        runner.map(|r| r as &dyn ntix_rs::package_manager::command_runner::CommandRunner),
    )
    .await
}

#[tokio::test]
async fn apply_diff_empty_diff_returns_true() {
    let diff = DiffResult::default();
    let options = empty_opts();
    let mut state = State::default();
    let path = temp_state_path();
    let result = apply(&diff, &options, &mut state, &path, false, None, None, None).await;
    assert!(result);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_winget_install_uses_mock_manager() {
    let mut mock = MockWingetManager::new();
    mock.install_result = true;

    let diff = DiffResult {
        to_install: vec![spec("test-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, Some(&mock), None, None).await;
    assert!(result);
    assert_eq!(state.winget.get("test-pkg"), Some(&"1.0".to_string()));
    assert_eq!(mock.install_call_count(), 1);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_winget_upgrade_uses_mock_manager() {
    let mock = MockWingetManager::new();

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

    let result = apply(&diff, &options, &mut state, &path, false, Some(&mock), None, None).await;
    assert!(result);
    assert_eq!(state.winget.get("test-pkg"), Some(&"2.0".to_string()));
    assert_eq!(mock.upgrade_call_count(), 1);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_winget_uninstall_uses_mock_manager() {
    let mock_runner = MockCommandRunner::new();

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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&mock_runner)).await;
    assert!(result);
    assert!(!state.winget.contains_key("test-pkg"));
    assert!(mock_runner
        .commands()
        .iter()
        .any(|c| c.contains("winget uninstall")));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_mixed_sources_works_correctly() {
    let mock_winget = MockWingetManager::new();
    let mock_runner = MockCommandRunner::new();

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
            interactive: false,
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
        Some(&mock_winget),
        None,
        Some(&mock_runner),
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
    let mut mock = MockWingetManager::new();
    mock.install_result = false;

    let diff = DiffResult {
        to_install: vec![spec("fail-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, Some(&mock), None, None).await;
    assert!(!result);
    assert!(!state.winget.contains_key("fail-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_stop_on_false_continues_after_failure() {
    let diff = DiffResult {
        to_install: vec![spec("fail-pkg", Some("1.0"), "winget"), spec("ok-pkg", Some("2.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    // fail-pkg fails, ok-pkg succeeds
    let mut mock = MockWingetManager::new();
    mock.install_per_id = Some(vec![
        ("fail-pkg".to_string(), false),
        ("ok-pkg".to_string(), true),
    ]);

    let result = apply(&diff, &options, &mut state, &path, false, Some(&mock), None, None).await;
    assert!(!result);
    assert_eq!(state.winget.get("ok-pkg"), Some(&"2.0".to_string()));
    assert_eq!(mock.install_call_count(), 2);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_stop_on_true_returns_early_on_failure() {
    let mut mock = MockWingetManager::new();
    mock.install_per_id = Some(vec![
        ("fail-pkg".to_string(), false),
        ("ok-pkg".to_string(), true),
    ]);

    let diff = DiffResult {
        to_install: vec![spec("fail-pkg", Some("1.0"), "winget"), spec("ok-pkg", Some("2.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, true, Some(&mock), None, None).await;
    assert!(!result);
    assert!(!state.winget.contains_key("ok-pkg"));
    assert_eq!(mock.install_call_count(), 1);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_disabled_source_skips_package() {
    let mock = MockWingetManager::new();

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

    let result = apply(&diff, &options, &mut state, &path, false, Some(&mock), None, None).await;
    assert!(result);
    assert!(state.winget.is_empty());
    assert_eq!(mock.install_call_count(), 0);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_null_version_records_latest() {
    let mock = MockWingetManager::new();

    let diff = DiffResult {
        to_install: vec![spec("test-pkg", None, "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, Some(&mock), None, None).await;
    assert!(result);
    assert_eq!(state.winget.get("test-pkg"), Some(&"latest".to_string()));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_diff_with_warnings_still_processes() {
    let mock = MockWingetManager::new();

    let diff = DiffResult {
        to_install: vec![spec("test-pkg", Some("1.0"), "winget")],
        warnings: vec!["some warning".to_string()],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, Some(&mock), None, None).await;
    assert!(result);
    assert!(state.winget.contains_key("test-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_config_missing_choco_warns_and_continues() {
    let mock = MockWingetManager::new();
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
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };
    let mut state = State::default();
    let path = temp_state_path();

    let warnings: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = warnings.clone();

    // Choco enabled but not installed -> warning, processing continues.
    let presence = MockManagerPresence::with_choco(false);
    let result = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        Some(&mock),
        Some(&presence),
        Some(&config),
        None,
        Some(&|msg: &str| captured.lock().unwrap().push(msg.to_string())),
        None,
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
async fn apply_diff_async_unknown_source_is_skipped() {
    let diff = DiffResult {
        to_install: vec![spec("test-pkg", Some("1.0"), "unknown")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, None, None, None).await;
    assert!(result);
    assert!(state.winget.is_empty());
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, None).await;
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, None).await;
    assert!(result);
    assert!(state.scoop.is_empty());
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_config_missing_scoop_warns_and_continues() {
    let mock = MockWingetManager::new();
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
    let config = NTIXConfig {
        options: options.clone(),
        ..Default::default()
    };
    let mut state = State::default();
    let path = temp_state_path();

    let warnings: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = warnings.clone();

    // Scoop enabled but not installed -> warning, processing continues.
    let mut presence = MockManagerPresence::new();
    presence.scoop_installed = false;
    let result = apply_diff(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        Some(&mock),
        Some(&presence),
        Some(&config),
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
    let mock = MockWingetManager::new();

    let diff = DiffResult {
        to_install: vec![spec("winget-pkg", Some("1.0"), "winget")],
        ..Default::default()
    };
    let options = options(
        WingetOptions {
            enable: true,
            accept_agreement: true,
            interactive: false,
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

    let result = apply(
        &diff,
        &options,
        &mut state,
        &path,
        false,
        Some(&mock),
        Some(&config),
        None,
    )
    .await;
    assert!(result);
    assert!(state.winget.contains_key("winget-pkg"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_to_adopt_updates_state_without_install() {
    let mock = MockWingetManager::new();

    let diff = DiffResult {
        to_adopt: vec![spec("manual-pkg", Some("3.0"), "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, Some(&mock), None, None).await;
    assert!(result);
    assert_eq!(state.winget.get("manual-pkg"), Some(&"3.0".to_string()));
    assert_eq!(mock.install_call_count(), 0);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_to_adopt_null_version_records_latest() {
    let mock = MockWingetManager::new();

    let diff = DiffResult {
        to_adopt: vec![spec("manual-pkg", None, "winget")],
        ..Default::default()
    };
    let options = winget_opts();
    let mut state = State::default();
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, Some(&mock), None, None).await;
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
    assert!(result);
    assert_eq!(state.chocolatey.get("choco-pkg"), Some(&"1.0".to_string()));
    assert!(runner.commands().iter().any(|c| c.contains("choco install")));
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
    assert!(result);
    assert_eq!(state.scoop.get("scoop-pkg"), Some(&"2.0".to_string()));
    assert!(runner.commands().iter().any(|c| c.contains("scoop install")));
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
    assert!(result);
    assert_eq!(state.chocolatey.get("choco-pkg"), Some(&"2.0".to_string()));
    assert!(runner.commands().iter().any(|c| c.contains("choco upgrade")));
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
    state.scoop.insert("scoop-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
    assert!(result);
    assert!(!state.chocolatey.contains_key("choco-pkg"));
    assert!(runner.commands().iter().any(|c| c.contains("choco uninstall")));
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
    state.scoop.insert("scoop-pkg".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
    assert!(result);
    assert!(!state.scoop.contains_key("scoop-pkg"));
    assert!(runner.commands().iter().any(|c| c.contains("scoop uninstall")));
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

    let result = apply(&diff, &options, &mut state, &path, true, None, None, Some(&runner)).await;
    assert!(!result);
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_choco_upgrade_failure_continues_on_failure() {
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let runner = MockCommandRunner::new();
    *runner.run_handler.lock().unwrap() = Some(Box::new(move |_: &str| {
        let n = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if n == 0 {
            1
        } else {
            0
        }
    }));

    let diff = DiffResult {
        to_upgrade: vec![spec("choco-fail", Some("2.0"), "chocolatey"), spec("choco-ok", Some("1.0"), "chocolatey")],
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
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

    let result = apply(&diff, &options, &mut state, &path, true, None, None, Some(&runner)).await;
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
        if n == 0 {
            1
        } else {
            0
        }
    }));

    let diff = DiffResult {
        to_remove: vec![spec("choco-fail", Some("1.0"), "chocolatey"), spec("choco-ok", Some("1.0"), "chocolatey")],
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
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

    let result = apply(&diff, &options, &mut state, &path, true, None, None, Some(&runner)).await;
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
        if n == 0 {
            1
        } else {
            0
        }
    }));

    let diff = DiffResult {
        to_upgrade: vec![spec("scoop-fail", Some("2.0"), "scoop"), spec("scoop-ok", Some("1.0"), "scoop")],
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
    state.scoop.insert("scoop-fail".to_string(), "1.0".to_string());
    state.scoop.insert("scoop-ok".to_string(), "1.0".to_string());
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
    assert!(result);
    assert!(!runner
        .commands()
        .iter()
        .any(|c| c.contains("scoop bucket add")));
    assert!(!runner
        .commands()
        .iter()
        .any(|c| c.contains("scoop bucket rm")));
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
    assert!(result);
    assert!(runner
        .commands()
        .iter()
        .any(|c| c.contains("scoop bucket add extras")));
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

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
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
    state
        .scoop_buckets
        .insert("extras".to_string(), None);
    state
        .scoop_buckets
        .insert("versions".to_string(), None);
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
    assert!(result);
    assert!(runner
        .commands()
        .iter()
        .any(|c| c.contains("scoop bucket rm extras")));
    assert!(runner
        .commands()
        .iter()
        .any(|c| c.contains("scoop bucket rm versions")));
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
    state
        .scoop_buckets
        .insert("extras".to_string(), None);
    let path = temp_state_path();

    let result = apply(&diff, &options, &mut state, &path, false, None, None, Some(&runner)).await;
    assert!(!result);
    assert!(state.scoop_buckets.contains_key("extras"));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_on_output_called_for_install() {
    let mock = MockWingetManager::new();
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
        Some(&mock),
        None,
        None,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        None,
        None,
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
    let mut mock = MockWingetManager::new();
    mock.install_result = false;
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
        Some(&mock),
        None,
        None,
        None,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        None,
    )
    .await;

    assert!(!result);
    let msgs = error_messages.lock().unwrap();
    assert!(msgs.iter().any(|m| m.contains("Failed to install")));
    assert!(msgs.iter().any(|m| m.contains("fail-pkg")));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_on_output_called_for_upgrade() {
    let mock = MockWingetManager::new();
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
        Some(&mock),
        None,
        None,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        None,
        None,
    )
    .await;

    assert!(result);
    let msgs = output_messages.lock().unwrap();
    assert!(msgs.iter().any(|m| m.contains("Upgrading")));
    remove_path(&path);
}

#[tokio::test]
async fn apply_diff_async_on_output_called_for_remove() {
    let mock_runner = MockCommandRunner::new();
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
        None,
        None,
        None,
        Some(&|msg: &str| msgs.lock().unwrap().push(msg.to_string())),
        None,
        Some(&mock_runner),
    )
    .await;

    assert!(result);
    let msgs = output_messages.lock().unwrap();
    assert!(msgs.iter().any(|m| m.contains("Removing")));
    remove_path(&path);
}
