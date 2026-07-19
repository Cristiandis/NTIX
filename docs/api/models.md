# Models

## Models

All types in `NTIX.Core.Models`.

### PackageEntry

A package declaration from the config file.

```csharp
public record PackageEntry(string Id, string? Version = null);
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Id` | `string` | required | Package identifier |
| `Version` | `string?` | `null` | Pinned version; `null` = latest |

### PackageSpec

A resolved package with its source manager.

```csharp
public record PackageSpec(string Id, string? Version, string Source);
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `Id` | `string` | Package identifier |
| `Version` | `string?` | Resolved version (null if unpinned) |
| `Source` | `string` | `"winget"`, `"chocolatey"`, or `"scoop"` |

### UpgradeInfo

Version information for an available upgrade.

```csharp
public record UpgradeInfo(string CurrentVersion, string AvailableVersion);
```

### InstalledPackages

Packages currently installed on the system.

```csharp
public record InstalledPackages(
    Dictionary<string, string>? Winget = null,
    Dictionary<string, string>? Chocolatey = null,
    Dictionary<string, string>? Scoop = null
);
```

Each dictionary maps package ID to installed version string.

### State

NTIX's tracked package state.

```csharp
public record State(
    int Version = 1,
    Dictionary<string, string>? Winget = null,
    Dictionary<string, string>? Chocolatey = null,
    Dictionary<string, string>? Scoop = null,
    Dictionary<string, string?>? ScoopBuckets = null
);
```

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `Version` | `int` | `1` | State file format version |
| `Winget` | `Dictionary<string, string>` | `new()` | Tracked winget packages |
| `Chocolatey` | `Dictionary<string, string>` | `new()` | Tracked chocolatey packages |
| `Scoop` | `Dictionary<string, string>` | `new()` | Tracked scoop packages |
| `ScoopBuckets` | `Dictionary<string, string?>` | `new()` | Scoop buckets added by NTIX (name → URL) |

### DiffResult

The computed set of actions to apply.

```csharp
public record DiffResult(
    List<PackageSpec> ToInstall = default!,
    List<PackageSpec> ToUpgrade = default!,
    List<PackageSpec> ToSkip = default!,
    List<PackageSpec> ToRemove = default!,
    List<PackageSpec> ToAdopt = default!,
    List<ScoopBucket> BucketsToAdd = default!,
    List<ScoopBucket> BucketsToRemove = default!,
    string? Error = null,
    List<string>? Warnings = null
);
```

| Property | Type | Description |
|----------|------|-------------|
| `ToInstall` | `List<PackageSpec>` | Packages to install |
| `ToUpgrade` | `List<PackageSpec>` | Packages to upgrade (requires `--upgrade`) |
| `ToSkip` | `List<PackageSpec>` | Packages already at desired state |
| `ToRemove` | `List<PackageSpec>` | Orphaned packages to remove |
| `ToAdopt` | `List<PackageSpec>` | External installs to adopt into state |
| `BucketsToAdd` | `List<ScoopBucket>` | Scoop buckets to add |
| `BucketsToRemove` | `List<ScoopBucket>` | NTIX-tracked scoop buckets to remove |
| `Error` | `string?` | Fatal error, if any |
| `Warnings` | `List<string>` | Non-fatal warnings |
| **`IsEmpty`** | `bool` | True if all action lists are empty |
| **`HasError`** | `bool` | True if `Error` is non-empty |

### ImportNode

Tracks the import tree from config parsing.

```csharp
public record ImportNode(string Path, List<ImportNode> Children);
```

### NTIXConfig

The fully parsed configuration.

```csharp
public record NTIXConfig(
    NTIXOptions Options,
    List<PackageEntry> WingetPackages = default!,
    List<PackageEntry> ChocoPackages = default!,
    List<PackageEntry> ScoopPackages = default!,
    List<ImportNode> Imports = default!
);
```
