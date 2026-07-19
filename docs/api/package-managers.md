# Package Managers

## Package Managers

Interfaces, implementations, and utilities for package manager interaction.

### IWingetManager

The sole interface in NTIX.Core. Used for dependency injection and testing.

```csharp
namespace NTIX.Core.PackageManager;
```

| Member | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `IsInstalled` | `bool IsInstalled { get; }` | `bool` | Sync check |
| `IsInstalledAsync` | `Task<bool> IsInstalledAsync(CancellationToken ct)` | `Task<bool>` | Async check |
| `GetInstalledPackagesAsync` | `Task<Dictionary<string, string>> GetInstalledPackagesAsync(CancellationToken ct)` | `ID -> version` | All installed packages |
| `GetUpgradablePackagesAsync` | `Task<Dictionary<string, UpgradeInfo>> GetUpgradablePackagesAsync(CancellationToken ct)` | `ID -> UpgradeInfo` | Upgradable packages |
| `InstallAsync` | `Task<bool> InstallAsync(string id, string? version, bool acceptAgreements, bool silent, CancellationToken ct)` | `bool` | Install a package |
| `UninstallAsync` | `Task<bool> UninstallAsync(string id, bool acceptAgreements, bool silent, CancellationToken ct)` | `bool` | Uninstall a package |
| `UpgradeAsync` | `Task<bool> UpgradeAsync(string id, bool acceptAgreements, bool silent, CancellationToken ct)` | `bool` | Upgrade a package |
| `PackageExistsAsync` | `Task<bool> PackageExistsAsync(string id, CancellationToken ct)` | `bool` | Check if package exists in repo |
| `EnsureInstalledAsync` | `Task EnsureInstalledAsync(bool interactive, CancellationToken ct)` | `void` | Auto-install winget if missing |
| `ExportPackagesAsync` | `Task<bool> ExportPackagesAsync(string filePath, CancellationToken ct)` | `bool` | Export package list |
| `ImportPackagesAsync` | `Task<bool> ImportPackagesAsync(string filePath, CancellationToken ct)` | `bool` | Import package list |
| `GetVersionAsync` | `Task<string?> GetVersionAsync(CancellationToken ct)` | `string?` | Winget version string |

### WingetManager

Default implementation using WGet.NET. Create via `new WingetManager()`.

### PackageManagerDetector

Static utility for discovering installed and upgradable packages.

| Method | Returns | Description |
|--------|---------|-------------|
| `IsChocolateyInstalled()` | `bool` | Runs `choco --version` |
| `IsScoopInstalled()` | `bool` | Runs `scoop --version` |
| `ValidateManagersAsync(options, config, wingetManager?)` | `(bool Valid, string? Error, List<string> Warnings)` | Validates enabled managers are installed |
| `GetInstalledPackagesAsync(wingetFactory?)` | `Task<InstalledPackages>` | Enumerates all installed packages |
| `GetWingetUpgradablePackagesAsync(wingetFactory?)` | `Task<Dictionary<string, UpgradeInfo>>` | Winget upgrades |
| `GetChocoUpgradablePackages()` | `Dictionary<string, UpgradeInfo>` | Choco upgrades |
| `GetScoopUpgradablePackages()` | `Dictionary<string, UpgradeInfo>` | Scoop upgrades |
| `ValidateWingetPackagesExistsAsync(ids, wingetManager?, ct)` | `Task<Dictionary<string, bool?>>` | Parallel existence check |
| `ValidateChocoPackagesExistsAsync(ids, ct)` | `Task<Dictionary<string, bool>>` | Parallel existence check |
| `ValidateScoopPackagesExistsAsync(ids, ct)` | `Task<Dictionary<string, bool>>` | Parallel existence check |

### CommandBuilder

Static CLI command builder. All methods return shell command strings.

| Method | Description |
|--------|-------------|
| `SanitizeId(id)` | Validates ID matches `[a-zA-Z0-9._\-/]` |
| `BuildChocoInstall(id, version, opts)` | `choco install` with flags |
| `BuildScoopInstall(id, version, opts)` | `scoop install` with flags |
| `BuildChocoUpgrade(id, opts)` | `choco upgrade` with flags |
| `BuildScoopUpgrade(id, opts)` | `scoop update` with flags |
| `BuildChocoUninstall(id, opts)` | `choco uninstall` with flags |
| `BuildScoopUninstall(id, opts)` | `scoop uninstall` with flags |
| `BuildChocoSearch(id)` | `choco search --limit-output` |
| `BuildScoopInfo(id)` | `scoop info` |
| `BuildScoopBucketAdd(name, url?)` | `scoop bucket add` |
| `BuildScoopBucketList()` | `scoop bucket list` |
