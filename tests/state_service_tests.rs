use std::collections::HashMap;
use std::fs;

use ntix_rs::models::state::State;
use ntix_rs::state_management::state_service;

fn temp_json_path(tag: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ntix_state_{}_{}_{}.json",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_file(&path);
    path
}

fn sample_state() -> State {
    State {
        winget: HashMap::from([("pkg1".to_string(), "1.0".to_string())]),
        chocolatey: HashMap::from([("pkg2".to_string(), "2.0".to_string())]),
        scoop: HashMap::from([("pkg3".to_string(), "3.0".to_string())]),
        ..State::default()
    }
}

#[test]
fn load_state_non_existent_returns_none() {
    let path = std::env::temp_dir().join("nonexistent_path_state.json");
    let result = state_service::load_state(Some(&path));
    assert!(result.is_none());
}

#[test]
fn save_and_load_state_round_trip() {
    let path = temp_json_path("roundtrip");
    let state = sample_state();
    let saved = state_service::save_state(&state, Some(&path), 3).unwrap();
    assert!(saved);

    let loaded = state_service::load_state(Some(&path)).expect("should load");
    assert_eq!(loaded.winget.get("pkg1"), Some(&"1.0".to_string()));
    assert_eq!(loaded.chocolatey.get("pkg2"), Some(&"2.0".to_string()));
    assert_eq!(loaded.scoop.get("pkg3"), Some(&"3.0".to_string()));
    let _ = fs::remove_file(&path);
}

#[test]
fn load_state_corrupt_json_returns_none() {
    let path = temp_json_path("corrupt");
    fs::write(&path, "not valid json {{{").unwrap();
    let loaded = state_service::load_state(Some(&path));
    assert!(loaded.is_none());
    let _ = fs::remove_file(&path);
}

#[test]
fn load_state_cleans_orphan_tmp() {
    let path = temp_json_path("orphan");
    let tmp_path = std::path::PathBuf::from(format!("{}.tmp", path.display()));
    let state = sample_state();
    assert!(state_service::save_state(&state, Some(&path), 3).unwrap());

    fs::write(&tmp_path, "orphan data").unwrap();
    assert!(tmp_path.is_file());

    let loaded = state_service::load_state(Some(&path)).expect("should load");
    assert_eq!(loaded.winget.get("pkg1"), Some(&"1.0".to_string()));
    assert!(!tmp_path.exists());
    let _ = fs::remove_file(&path);
}

#[test]
fn save_state_directory_creation_fails_returns_err() {
    let bad_path = std::env::temp_dir()
        .join("ntix_test_impossible<dir")
        .join("state.json");
    let state = State::default();
    let result = state_service::save_state(&state, Some(&bad_path), 3);
    assert!(result.is_err());
}

#[test]
fn save_state_exhausts_retries_returns_false() {
    let temp_dir = std::env::temp_dir().join(format!(
        "ntix_state_retries_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap();
    let state_file_path = temp_dir.join("state.json");
    fs::create_dir_all(&state_file_path).unwrap();

    let state = sample_state();
    let result = state_service::save_state(&state, Some(&state_file_path), 2).unwrap();
    assert!(!result);

    let _ = fs::remove_dir_all(&state_file_path);
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn load_state_v1_migrates_to_v2_with_empty_config_files() {
    let path = temp_json_path("v1migrate");
    fs::write(
        &path,
        r#"{
            "version": 1,
            "winget": { "Winget.Package": "1.0" },
            "chocolatey": {},
            "scoop": {},
            "scoopBuckets": {}
        }"#,
    )
    .unwrap();

    let loaded = state_service::load_state(Some(&path)).expect("should load");
    assert_eq!(loaded.version, 2);
    assert!(loaded.config_files.is_empty());
    assert_eq!(
        loaded.winget.get("Winget.Package").map(|s| s.as_str()),
        Some("1.0")
    );

    fs::remove_file(&path).unwrap();
}

#[test]
fn state_round_trip_preserves_config_files() {
    let mut state = sample_state();
    state
        .config_files
        .insert("C:/dest/app.conf".to_string(), "abc123".to_string());
    let path = temp_json_path("cfroundtrip");
    assert!(state_service::save_state(&state, Some(&path), 3).unwrap());
    let loaded = state_service::load_state(Some(&path)).expect("should load");
    assert_eq!(loaded.version, 2);
    assert_eq!(
        loaded
            .config_files
            .get("C:/dest/app.conf")
            .map(|s| s.as_str()),
        Some("abc123")
    );
    fs::remove_file(&path).unwrap();
}
