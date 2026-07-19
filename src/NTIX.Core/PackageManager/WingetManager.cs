using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using WGetNET;
using NTIX.Core.Models;

namespace NTIX.Core.PackageManager;

public sealed class WingetManager : IWingetManager
{
    private readonly WinGetPackageManager _packageManager;
    private readonly WinGet _winGet;

    public WingetManager()
    {
        _packageManager = new WinGetPackageManager();
        _winGet = new WinGet();
    }

    public bool IsInstalled => _winGet.IsInstalled;

    public Task<bool> IsInstalledAsync(CancellationToken ct = default)
    {
        return Task.FromResult(_winGet.IsInstalled);
    }

    public async Task<Dictionary<string, string>> GetInstalledPackagesAsync(CancellationToken ct = default)
    {
        var result = new Dictionary<string, string>();
        var packages = await _packageManager.GetInstalledPackagesAsync(ct);
        
        foreach (var pkg in packages)
        {
            if (!string.IsNullOrEmpty(pkg.Id) && pkg.Version != null)
            {
                result[pkg.Id] = pkg.Version.ToString();
            }
        }
        return result;
    }

    public async Task<Dictionary<string, UpgradeInfo>> GetUpgradablePackagesAsync(CancellationToken ct = default)
    {
        var result = new Dictionary<string, UpgradeInfo>();
        var packages = await _packageManager.GetUpgradeablePackagesAsync(ct);
        
        foreach (var pkg in packages)
        {
            if (!string.IsNullOrEmpty(pkg.Id) && pkg.Version != null && pkg.AvailableVersion != null)
            {
                result[pkg.Id] = new UpgradeInfo(
                    pkg.Version.ToString(),
                    pkg.AvailableVersion.ToString()
                );
            }
        }
        return result;
    }

    public async Task<bool> InstallAsync(string id, string? version = null, bool acceptAgreements = true, bool silent = true, CancellationToken ct = default)
    {
        var args = WinGetArguments.Install()
            .Query(id)
            .Exact();
        
        if (!string.IsNullOrEmpty(version))
        {
            args.Version(version);
        }
        
        if (acceptAgreements)
        {
            args.AcceptSourceAgreements().AcceptPackageAgreements();
        }
        
        if (silent)
        {
            args.Silent();
        }

        var result = await _winGet.ExecuteCustomAsync(args, ct);
        return result.Success;
    }

    public async Task<bool> UninstallAsync(string id, bool acceptAgreements = true, bool silent = true, CancellationToken ct = default)
    {
        var args = WinGetArguments.Uninstall()
            .Query(id)
            .Exact();
        
        if (acceptAgreements)
        {
            args.AcceptSourceAgreements().AcceptPackageAgreements();
        }
        
        if (silent)
        {
            args.Silent();
        }

        var result = await _winGet.ExecuteCustomAsync(args, ct);
        return result.Success;
    }

    public async Task<bool> UpgradeAsync(string id, bool acceptAgreements = true, bool silent = true, CancellationToken ct = default)
    {
        var args = WinGetArguments.Upgrade()
            .Query(id)
            .Exact();
        
        if (acceptAgreements)
        {
            args.AcceptSourceAgreements().AcceptPackageAgreements();
        }
        
        if (silent)
        {
            args.Silent();
        }

        var result = await _winGet.ExecuteCustomAsync(args, ct);
        return result.Success;
    }

    public async Task<bool> ExportPackagesAsync(string filePath, CancellationToken ct = default)
    {
        return await _packageManager.ExportPackagesToFileAsync(filePath, ct);
    }

    public async Task<bool> ImportPackagesAsync(string filePath, CancellationToken ct = default)
    {
        return await _packageManager.ImportPackagesFromFileAsync(filePath, ct);
    }

    public async Task<string?> GetVersionAsync(CancellationToken ct = default)
    {
        var info = await _winGet.GetInfoAsync(ct);
        return info?.VersionString;
    }

    public async Task<bool> PackageExistsAsync(string id, CancellationToken ct = default)
    {
        try
        {
            var packages = await _packageManager.SearchPackageAsync(id, true, ct);
            return packages.Any(p =>
                string.Equals(p.Id, id, StringComparison.OrdinalIgnoreCase));
        }
        catch
        {
            return false;
        }
    }

    public async Task EnsureInstalledAsync(bool interactive = false, CancellationToken ct = default)
    {
        if (!await IsInstalledAsync(ct))
        {
            var psCommand = """
                if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
                    Write-Host "Installing winget via Microsoft Store..."
                    try {
                        Add-AppxPackage -Register "C:\Program Files\WindowsApps\Microsoft.DesktopAppInstaller_*_x64__8wekyb3d8bbwe\AppxManifest.xml" -DisableDevelopmentMode 2>$null
                    } catch {
                        $proc = Start-Process -FilePath "ms-windows-store:" -ArgumentList "pdp?ProductId=9NBLGGH4NNS1" -PassThru
                        $proc.WaitForExit(30000)
                    }
                }
                """;
            
            var startInfo = new System.Diagnostics.ProcessStartInfo
            {
                FileName = "powershell.exe",
                Arguments = $"-NoProfile -ExecutionPolicy Bypass -Command \"{psCommand.Replace("\"", "\\\"")}\"",
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true,
                StandardOutputEncoding = System.Text.Encoding.UTF8,
                StandardErrorEncoding = System.Text.Encoding.UTF8
            };

            using var process = System.Diagnostics.Process.Start(startInfo);
            if (process != null)
            {
                await process.WaitForExitAsync(ct);
            }
        }
    }
}