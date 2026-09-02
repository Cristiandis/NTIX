# State

## StateService

Manages the JSON state file that tracks NTIX-managed packages.

```rust
use ntix_rs::state_management::state_service;
```

### Items

| Item | Signature | Description |
|------|-----------|-------------|
| `get_state_path` | `fn get_state_path() -> Result<PathBuf, Box<dyn Error>>` | Returns `%LOCALAPPDATA%/ntix/state.json` |
| `load_state` | `fn load_state(path: Option<&Path>) -> Option<State>` | Loads state from disk; `None` if missing or corrupt |
| `save_state` | `fn save_state(state: &State, path: Option<&Path>, max_retries: u32) -> Result<bool, Box<dyn Error>>` | Atomic save with retry; returns `true` on success |

### Behavior

- **Atomic writes**: writes to a `.tmp` file, then renames it to the target
- **Retry**: up to `max_retries` attempts with linear backoff (`50ms * attempt`)
- **Stale cleanup**: removes leftover `.tmp` files before loading
- **Serialization**: `serde_json` with camelCase field names (`scoopBuckets`, `scoop_buckets` map key)
- **Path resolution**: a `None` or empty path resolves to `get_state_path()`
