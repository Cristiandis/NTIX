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
| `load_state` | `fn load_state() -> Option<State>` | Loads state from the default path; `None` if missing, corrupt, or unreadable |
| `load_state_from` | `fn load_state_from(path: &Path) -> Option<State>` | Loads state from an explicit path |
| `save_state_to` | `fn save_state_to(state: &State, path: &Path) -> Result<bool, Box<dyn Error>>` | Atomic save with retry; returns `true` on success |

The retry count (`MAX_RETRIES = 3`) is private; `save_state_to` always retries up to three times.

### Behavior

- **Atomic writes**: writes to a `.tmp` file, then renames it to the target
- **Retry**: up to 3 attempts with linear backoff (`50ms * attempt`)
- **Stale cleanup**: removes leftover `.tmp` files before loading
- **Serialization**: `serde_json` with camelCase field names (`scoopBuckets`, `configFiles`)
- **Format version**: a loaded state with `version < 2` is upgraded to `2` in memory; current version is `2` (added `configFiles` tracking, which deserializes as empty when absent)
