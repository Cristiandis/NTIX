# Options

## Options

All option types in `NTIX.Core.Models`.

### NTIXOptions

Top-level options container.

```csharp
public record NTIXOptions(
    WingetOptions Winget = null!,
    ChocoOptions Chocolatey = null!,
    ScoopOptions Scoop = null!
);
```

Each property auto-initializes to a default (all disabled) if not provided.

### WingetOptions

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Enable` | `bool` | `false` | Enable winget support |
| `AcceptAgreements` | `bool` | `false` | Auto-accept license agreements |
| `Interactive` | `bool` | `false` | Show installer UI |

### ChocoOptions

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Enable` | `bool` | `false` | Enable Chocolatey support |
| `Yes` | `bool` | `false` | Pass `-y` to skip prompts |
| `Force` | `bool` | `false` | Pass `--force` |
| `IgnoreDependencies` | `bool` | `false` | Pass `--ignore-dependencies` |
| `AllowDowngrade` | `bool` | `false` | Pass `--allow-downgrade` |
| `SkipPowerShell` | `bool` | `false` | Pass `--skip-scripts` |
| `Params` | `string?` | `null` | Custom `--params` value |
| `Pre` | `bool` | `false` | Pass `--pre` for prereleases |

### ScoopBucket

```csharp
public record ScoopBucket(string Name, string? Url = null);
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `Name` | `string` | Bucket name (e.g. `"main"`, `"extras"`) |
| `Url` | `string?` | Optional custom bucket URL |

### ScoopOptions

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `Enable` | `bool` | `false` | Enable Scoop support |
| `Buckets` | `List<ScoopBucket>` | `[main, extras, versions]` | Buckets to ensure exist |
| `Global` | `bool` | `false` | Pass `-g` for global installs |
| `Independent` | `bool` | `false` | Pass `-i` |
| `NoCache` | `bool` | `false` | Pass `-k` |
| `SkipHashCheck` | `bool` | `false` | Pass `-s` |
| `Arch` | `string?` | `null` | Pass `--arch <value>` |
