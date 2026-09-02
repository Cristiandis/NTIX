use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::paths;

#[cfg(not(target_os = "windows"))]
use std::fs::OpenOptions;

pub struct LockFile {
    lock_stream: Option<File>,
    lock_path: PathBuf,
}

impl LockFile {
    pub fn new(
        lock_path: Option<PathBuf>,
        should_lock: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let lock_path = lock_path.unwrap_or(Self::get_default_lock_path()?);

        if !should_lock {
            return Ok(Self {
                lock_stream: None,
                lock_path,
            });
        }

        if let Some(dir) = lock_path.parent() {
            fs::create_dir_all(dir)?;
        }

        let mut file = open_exclusive(&lock_path).map_err(|e| {
            format!(
                "Another ntix apply is running (lock file is locked). \
                 If this is stale, delete {}: {e}",
                lock_path.display()
            )
        })?;

        let pid = std::process::id();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let lock_info = format!("{pid}@{timestamp}");

        file.write_all(lock_info.as_bytes())?;
        file.flush()?;

        Ok(Self {
            lock_stream: Some(file),
            lock_path,
        })
    }

    pub fn get_default_lock_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(paths::local_app_data_path()?.join("apply.lock"))
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        if self.lock_stream.take().is_some() {
            let _ = fs::remove_file(&self.lock_path);
        }
    }
}

/// Opens the lock file for writing, truncating it, holding it without
/// sharing on Windows so a concurrent `LockFile::new` fails.
#[cfg(target_os = "windows")]
fn open_exclusive(path: &std::path::Path) -> io::Result<File> {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::FromRawHandle;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_MODE,
    };

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            // dwShareMode = 0 -> no sharing; a second open fails while held.
            FILE_SHARE_MODE(0),
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
    .map_err(|_| io::Error::last_os_error())?;

    Ok(unsafe { File::from_raw_handle(handle.0 as _) })
}

#[cfg(not(target_os = "windows"))]
fn open_exclusive(path: &std::path::Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
}
