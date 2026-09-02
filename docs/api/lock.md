# Lock

## LockFile

Prevents concurrent `ntix apply` execution via an exclusive file lock.

```rust
use ntix_rs::lock::lock_file::LockFile;
```

### API

```rust
pub struct LockFile {
    lock_stream: Option<File>,
    lock_path: PathBuf,
}

impl LockFile {
    pub fn new(lock_path: Option<PathBuf>, should_lock: bool)
        -> Result<Self, Box<dyn std::error::Error>>;
    pub fn get_default_lock_path() -> Result<PathBuf, Box<dyn std::error::Error>>;
}

impl Drop for LockFile { /* releases the lock and removes the file */ }
```

| Member | Description |
|--------|-------------|
| `new` | Acquires an exclusive lock at `lock_path` (or the default). Returns `Err` if already locked by another process. Pass `should_lock = false` to create an unlocked instance. |
| `get_default_lock_path` | Returns `%LOCALAPPDATA%/ntix/apply.lock` |
| `Drop` | Releases the lock stream and deletes the lock file |

### Behavior

- On Windows, opens the lock file with `CreateFileW` using a zero share mode, so a second open fails while held
- On non-Windows hosts, uses a create-and-truncate open (advisory)
- The lock file body contains `PID@UnixTimestamp`, which helps identify stale locks
- When locking fails, the error message suggests deleting the lock file if it is stale
