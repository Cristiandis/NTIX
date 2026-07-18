using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using NTIX.Core.Models;

namespace NTIX.Core.PackageManager;

public interface IWingetManager
{
    bool IsInstalled { get; }
    Task<bool> IsInstalledAsync(CancellationToken ct = default);
    Task<Dictionary<string, string>> GetInstalledPackagesAsync(CancellationToken ct = default);
    Task<Dictionary<string, UpgradeInfo>> GetUpgradablePackagesAsync(CancellationToken ct = default);
    Task<bool> InstallAsync(string id, string? version = null, bool acceptAgreements = true, bool silent = true, CancellationToken ct = default);
    Task<bool> UninstallAsync(string id, bool acceptAgreements = true, bool silent = true, CancellationToken ct = default);
    Task<bool> UpgradeAsync(string id, bool acceptAgreements = true, bool silent = true, CancellationToken ct = default);
    Task<bool> ExportPackagesAsync(string filePath, CancellationToken ct = default);
    Task<bool> ImportPackagesAsync(string filePath, CancellationToken ct = default);
    Task<string?> GetVersionAsync(CancellationToken ct = default);
    Task<bool> PackageExistsAsync(string id, CancellationToken ct = default);
    Task EnsureInstalledAsync(bool interactive = false, CancellationToken ct = default);
}