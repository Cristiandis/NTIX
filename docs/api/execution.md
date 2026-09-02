# Execution

## ExecutionEngine

Applies a computed diff by invoking package manager commands.

```rust
use ntix_rs::execution::execution_engine;
```

### apply_diff

```rust
pub async fn apply_diff(
    diff: &DiffResult,
    options: &NTIXOptions,
    state: &mut State,
    state_path: &Path,
    stop_on_failure: bool,
    winget_manager: Option<&dyn WingetManagerTrait>,
    presence: Option<&dyn ManagerPresence>,
    config: Option<&NTIXConfig>,
    on_output: Option<LineCallback<'_>>,
    on_error: Option<LineCallback<'_>>,
    runner: Option<&dyn CommandRunner>,
) -> bool
```

Where `LineCallback<'a> = &'a (dyn Fn(&str) + Sync)`.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `diff` | `&DiffResult` | required | Computed diff to execute |
| `options` | `&NTIXOptions` | required | Package manager options |
| `state` | `&mut State` | required | Mutable state; updated in place |
| `state_path` | `&Path` | required | Path to persist state after each operation |
| `stop_on_failure` | `bool` | `false` | Stop on first failure |
| `winget_manager` | `Option<&dyn WingetManagerTrait>` | `None` | Injected winget manager; uses `WingetManager` if `None` |
| `presence` | `Option<&dyn ManagerPresence>` | `None` | Injected choco/scoop presence; probes the system if `None` |
| `config` | `Option<&NTIXConfig>` | `None` | Re-validates managers before execution |
| `on_output` | `Option<LineCallback>` | `None` | Stdout callback |
| `on_error` | `Option<LineCallback>` | `None` | Stderr callback |
| `runner` | `Option<&dyn CommandRunner>` | `None` | Command runner; uses `ProcessCommandRunner` if `None` |

**Returns:** `bool` - true if all operations succeeded.

### Execution Order

1. If `config` is provided, re-validates enabled managers and reports new warnings on `on_error`
2. Ensures scoop buckets are added and orphaned buckets removed (if scoop is enabled)
3. Installs packages (`to_install`)
4. Upgrades packages (`to_upgrade`)
5. Removes orphans (`to_remove`)
6. Adopts packages (`to_adopt`) - state only, no install
7. Saves state after each successful operation

Operations for a disabled manager are skipped. If `stop_on_failure` is true, execution halts on the first failed operation.

### Locking

`apply_diff` does not acquire the lock itself. Locking is performed by the caller (the binary acquires a `LockFile` before invoking `apply_diff`).
