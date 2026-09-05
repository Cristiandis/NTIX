# Execution

## ExecutionEngine

Applies a computed diff by invoking package manager commands.

```rust
use ntix_rs::execution::execution_engine;
```

### apply_diff

```rust
#[allow(clippy::too_many_arguments)]
pub async fn apply_diff(
    diff: &DiffResult,
    options: &NTIXOptions,
    state: &mut State,
    state_path: &Path,
    stop_on_failure: bool,
    validation: &ValidationResult,
    apply_config: bool,
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
| `validation` | `&ValidationResult` | required | Manager capability check from the diff pass |
| `apply_config` | `bool` | `false` | Apply `configFiles` actions |
| `on_output` | `Option<LineCallback>` | `None` | Stdout callback |
| `on_error` | `Option<LineCallback>` | `None` | Stderr callback |
| `runner` | `Option<&dyn CommandRunner>` | `None` | Command runner; uses `ProcessCommandRunner` if `None` |

**Returns:** `bool` - true if all operations succeeded.

### Execution Order

1. Reports manager-validation warnings that are not already in `diff.warnings` through `on_error`
2. Ensures scoop buckets are added and orphaned buckets removed (if scoop is enabled)
3. Installs packages (`to_install`)
4. Upgrades packages (`to_upgrade`)
5. Removes orphans (`to_remove`)
6. Adopts packages (`to_adopt`) - state only, no install
7. Applies `configFiles` (`to_create` and `to_update` are copied from src to dest; orphans are dropped from tracking without touching disk) - only when `apply_config` is true
8. Saves state after each successful operation

Operations for a disabled manager are skipped. If `stop_on_failure` is true, execution halts on the first failed operation.

### Failure recovery

Some managers exit nonzero even for successful outcomes: winget reports an install of an already-installed package as errored, and choco/scoop uninstall of an absent package behaves similarly. After any failed install or upgrade, `apply_diff` re-queries the installed-package list and counts the operation as successful if the package is present; after a failed remove, it counts as successful if the package is confirmed absent. A failed list query is treated as a failure, never as a false positive.

### Locking

`apply_diff` does not acquire the lock itself. Locking is performed by the caller (the binary acquires a `LockFile` before invoking `apply_diff`).
