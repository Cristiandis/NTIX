# StateService

## StateService

Manages the JSON state file that tracks NTIX-managed packages.

```csharp
namespace NTIX.Core.StateManagement;
```

### Members

| Method | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `GetStatePath` | `string GetStatePath()` | `string` | Returns `%LOCALAPPDATA%/ntix/state.json` |
| `LoadState` | `State? LoadState(string? path = null)` | `State?` | Loads state from disk; null if missing/corrupt |
| `SaveState` | `bool SaveState(State state, string? path = null, int maxRetries = 3)` | `bool` | Atomic save with retry |
| `SaveStateAsync` | `Task<bool> SaveStateAsync(State state, string? path = null, int maxRetries = 3, CancellationToken ct = default)` | `Task<bool>` | Async atomic save |

### Behavior

- **Atomic writes**: writes to `.tmp` then moves to target
- **Retry**: up to 3 attempts with exponential backoff (50ms * attempt)
- **Stale cleanup**: removes leftover `.tmp` files before loading
- **Serialization**: `System.Text.Json` with source-generated context (AOT-safe)
