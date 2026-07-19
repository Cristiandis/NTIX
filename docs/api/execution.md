# ExecutionEngine

## ExecutionEngine

Applies a computed diff by invoking package manager commands.

```csharp
namespace NTIX.Core.Execution;
```

### ApplyDiffAsync

```csharp
public static async Task<bool> ApplyDiffAsync(
    DiffResult diff,
    NTIXOptions options,
    State state,
    string statePath,
    bool stopOnFailure = true,
    IWingetManager? wingetManager = null,
    NTIXConfig? config = null,
    Action<string>? onOutput = null,
    Action<string>? onError = null);
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `diff` | `DiffResult` | required | Computed diff to execute |
| `options` | `NTIXOptions` | required | Package manager options |
| `state` | `State` | required | Mutable state; updated in-place |
| `statePath` | `string` | required | Path to persist state after each operation |
| `stopOnFailure` | `bool` | `true` | Stop on first failure |
| `wingetManager` | `IWingetManager?` | `null` | Injected winget manager |
| `config` | `NTIXConfig?` | `null` | Re-validates managers before execution |
| `onOutput` | `Action<string>?` | `null` | Stdout callback |
| `onError` | `Action<string>?` | `null` | Stderr callback |

**Returns:** `Task<bool>` - true if all operations succeeded.

### Execution Order

1. Returns `false` early if `diff.Error` is set
2. Ensures scoop buckets exist (if scoop packages need installing)
3. Installs packages (`ToInstall`)
4. Upgrades packages (`ToUpgrade`)
5. Removes orphans (`ToRemove`)
6. Adopts packages (`ToAdopt`) - state-only, no install
7. Saves state after each successful operation
