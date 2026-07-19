# DiffEngine

## DiffEngine

Computes the difference between desired state (config) and current state.

```csharp
namespace NTIX.Core.Diff;
```

### ComputeDiffAsync

```csharp
public static async Task<DiffResult> ComputeDiffAsync(
    NTIXConfig config,
    State state,
    InstalledPackages? installed = null,
    IWingetManager? wingetManager = null,
    bool validatePackages = true,
    bool adoptMode = false,
    bool upgradeMode = false,
    IProgress<string>? progress = null);
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `config` | `NTIXConfig` | required | Desired package state |
| `state` | `State` | required | Current tracked state |
| `installed` | `InstalledPackages?` | `null` | Pre-fetched packages; auto-discovers if null |
| `wingetManager` | `IWingetManager?` | `null` | Injected winget manager; creates new if null |
| `validatePackages` | `bool` | `true` | Validate package existence in remote repos |
| `adoptMode` | `bool` | `false` | Adopt externally-installed packages into state |
| `upgradeMode` | `bool` | `false` | Check for available upgrades |
| `progress` | `IProgress<string>?` | `null` | Status message reporter |

**Returns:** `Task<DiffResult>`

### Execution Flow

1. Validates enabled package managers are installed
2. Discovers installed packages
3. Fetches upgradable packages (only if `upgradeMode` and source has unpinned packages)
4. Classifies each declared package into: `ToInstall`, `ToUpgrade`, `ToSkip`, `ToAdopt`
5. Validates package existence (removes invalid from `ToInstall`)
6. Finds orphans (in state but not in config) -> `ToRemove`
