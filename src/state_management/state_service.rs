use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use crate::models::state::State;
use crate::paths;

pub const DEFAULT_MAX_RETRIES: u32 = 3;

pub fn get_state_path() -> Result<PathBuf, Box<dyn Error>> {
    Ok(paths::local_app_data_path()?.join("state.json"))
}

pub fn load_state(path: Option<&Path>) -> Option<State> {
    let state_path = match path {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => get_state_path().ok()?,
    };

    let temp_path = PathBuf::from(format!("{}.tmp", state_path.display()));
    if temp_path.is_file() {
        let _ = fs::remove_file(&temp_path);
    }

    if !state_path.is_file() {
        return None;
    }

    let json = match fs::read_to_string(&state_path) {
        Ok(json) => json,
        Err(e) => {
            eprintln!(
                "[NTIX] Could not read state file {}: {e}",
                state_path.display()
            );
            return None;
        }
    };
    let mut state: State = match serde_json::from_str(&json) {
        Ok(state) => state,
        Err(e) => {
            eprintln!(
                "[NTIX] Could not parse state file {} (corrupt?). Starting from empty state: {e}",
                state_path.display()
            );
            return None;
        }
    };
    if state.version < 2 {
        state.version = 2;
    }
    Some(state)
}

pub fn save_state(
    state: &State,
    path: Option<&Path>,
    max_retries: u32,
) -> Result<bool, Box<dyn Error>> {
    let state_path = match path {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => get_state_path()?,
    };

    let temp_path = PathBuf::from(format!("{}.tmp", state_path.display()));

    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&state)?;

    for attempt in 1..=max_retries {
        let write_result =
            fs::write(&temp_path, &json).and_then(|_| fs::rename(&temp_path, &state_path));

        if temp_path.is_file() {
            let _ = fs::remove_file(&temp_path);
        }

        match write_result {
            Ok(()) => return Ok(true),
            Err(_) => {
                if attempt >= max_retries {
                    return Ok(false);
                }
                thread::sleep(Duration::from_millis(50 * attempt as u64));
            }
        }
    }

    Ok(false)
}
