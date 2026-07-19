# NTIX.Core API Reference

## NTIX.Core API Reference

`NTIX.Core` is the core library behind NTIX. It provides config parsing, diff computation, package execution, state management, and file locking - all as a pure data/logic library with no console I/O.

### Package

```
NTIX.Core (net10.0)
```

Dependencies: `LuaCSharp`, `WGet.NET`

### Namespaces

| Namespace | Purpose |
|-----------|---------|
| `NTIX.Core.Models` | Data models, options, and config records |
| `NTIX.Core.Config` | Lua config file loading |
| `NTIX.Core.Diff` | Diff computation (desired vs current state) |
| `NTIX.Core.Execution` | Package install/upgrade/remove execution |
| `NTIX.Core.StateManagement` | State file persistence |
| `NTIX.Core.PackageManager` | Package manager abstraction and detection |
| `NTIX.Core.Lock` | Concurrent execution locking |

### Quick Start

```csharp
using NTIX.Core.Config;
using NTIX.Core.Diff;
using NTIX.Core.Execution;
using NTIX.Core.StateManagement;
using NTIX.Core.Models;

// 1. Load config
var config = ConfigLoader.Load("config.lua");

// 2. Load state
var state = StateService.LoadState() ?? new State();

// 3. Compute diff
var diff = await DiffEngine.ComputeDiffAsync(config, state);

// 4. Apply
var statePath = StateService.GetStatePath();
var success = await ExecutionEngine.ApplyDiffAsync(
    diff, config.Options, state, statePath,
    onOutput: Console.WriteLine,
    onError: Console.Error.WriteLine);
```

### Key Types

| Type | Description |
|------|-------------|
| `NTIXConfig` | Parsed config file (options + package lists) |
| `State` | Current tracked packages |
| `DiffResult` | Computed actions (install/upgrade/remove/adopt/skip) |
| `InstalledPackages` | Packages found on the system |
| `IWingetManager` | Abstraction over winget CLI (mockable) |
| `ICommandRunner` | Abstraction over shell commands (mockable) |
| `ProcessCommandRunner` | Default `ICommandRunner` using `cmd.exe` |
| `ConfigLoader` | Parses Lua config files |
| `DiffEngine` | Computes what needs to change |
| `ExecutionEngine` | Applies changes via package managers |
| `StateService` | Reads/writes the state file |
| `LockFile` | Prevents concurrent `apply` runs |
