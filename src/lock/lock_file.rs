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
    /// `None` only during `Drop`, after the handle has been closed.
    file: Option<File>,
    lock_path: PathBuf,
}

impl LockFile {
    /// Acquires the default apply lock. Fails while another holder keeps the
    /// file locked.
    pub fn acquire() -> Result<Self, Box<dyn std::error::Error>> {
        Self::acquire_at(get_default_lock_path()?)
    }

    pub fn acquire_at(lock_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
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
            file: Some(file),
            lock_path,
        })
    }
}

pub fn get_default_lock_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(paths::local_app_data_path()?.join("apply.lock"))
}

impl Drop for LockFile {
    fn drop(&mut self) {
        // Close the handle first: Windows cannot delete an open file.
        drop(self.file.take());
        let _ = fs::remove_file(&self.lock_path);
    }
}

#[cfg(target_os = "windows")]
fn open_exclusive(path: &std::path::Path) -> io::Result<File> {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::FromRawHandle;
    use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};
    use windows::Win32::Storage::FileSystem::{
        CREATE_ALWAYS, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_MODE,
    };
    use windows::core::PCWSTR;

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
