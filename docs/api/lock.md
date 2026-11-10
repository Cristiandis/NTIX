# Lock

## LockFile

Prevents concurrent `ntix apply` execution via an exclusive file lock.

```rust
use ntix_rs::lock::lock_file::LockFile;
```

### API

```rust
pub struct LockFile {
    file: Option<File>,   // None only during Drop
    lock_path: PathBuf,
}

impl LockFile {
    pub fn acquire() -> Result<Self, Box<dyn std::error::Error>>;
    pub fn acquire_at(lock_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>>;
}

pub fn get_default_lock_path() -> Result<PathBuf, Box<dyn std::error::Error>>;

impl Drop for LockFile { /* closes the handle, then removes the file */ }
```

| Member | Description |
|--------|-------------|
| `LockFile::acquire` | Acquires the lock at the default path. Returns `Err` while another holder keeps it locked |
| `LockFile::acquire_at` | Acquires the lock at an explicit path (creates the parent directory if needed) |
| `get_default_lock_path` | Returns `%LOCALAPPDATA%/ntix/apply.lock` |
| `Drop` | Closes the file handle first (Windows cannot delete an open file), then deletes the lock file |

### Behavior

- On Windows, opens the lock file with `CreateFileW` using a zero share mode, so a second open fails while held
- On non-Windows hosts, uses a create-and-truncate open (advisory)
- The lock file body contains `PID@UnixTimestamp`, which helps identify stale locks
- When locking fails, the error message suggests deleting the lock file if it is stale
