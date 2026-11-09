use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use crate::models::state::State;
use crate::paths;

const MAX_RETRIES: u32 = 3;

pub fn get_state_path() -> Result<PathBuf, Box<dyn Error>> {
    Ok(paths::local_app_data_path()?.join("state.json"))
}

/// Loads the state file from the default location. `None` when missing or
/// unreadable (the caller must then treat the system as unmanaged so far).
pub fn load_state() -> Option<State> {
    let path = get_state_path().ok()?;
    load_state_from(&path)
}

pub fn load_state_from(path: &Path) -> Option<State> {
    let temp_path = temp_path(path);
    remove_file_if_exists(&temp_path);

    if !path.is_file() {
        return None;
    }

    let json = match fs::read_to_string(path) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("[NTIX] Could not read state file {}: {e}", path.display());
            return None;
        }
    };
    let mut state: State = match serde_json::from_str(&json) {
        Ok(state) => state,
        Err(e) => {
            eprintln!(
                "[NTIX] Could not parse state file {} (corrupt?). Starting from empty state: {e}",
                path.display()
            );
            return None;
        }
    };
    if state.version < 2 {
        state.version = 2;
    }
    Some(state)
}

pub fn save_state_to(state: &State, path: &Path) -> Result<bool, Box<dyn Error>> {
    let temp_path = temp_path(path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(state)?;

    for attempt in 1..=MAX_RETRIES {
        let write_result = fs::write(&temp_path, &json).and_then(|_| fs::rename(&temp_path, path));

        remove_file_if_exists(&temp_path);

        if write_result.is_ok() {
            return Ok(true);
        }

        if attempt < MAX_RETRIES {
            thread::sleep(Duration::from_millis(50 * attempt as u64));
        }
    }

    Ok(false)
}

fn temp_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.tmp", path.display()))
}

fn remove_file_if_exists(path: &Path) {
    if path.is_file() {
        let _ = fs::remove_file(path);
    }
}
