using System.Diagnostics;
using System.Text.RegularExpressions;
using System.Text.Json;
using NTIX.Core.Models;

namespace NTIX.Core.PackageManager;

public static class PackageManagerDetector
{

    public static bool IsChocolateyInstalled() => RunProcess("choco --version") != null;
    public static bool IsScoopInstalled() => RunProcess("scoop --version") != null;

    public static async Task<(bool Valid, string? Error, List<string> Warnings)> ValidateManagersAsync(
        NTIXOptions options,
        NTIXConfig config,
        IWingetManager? wingetManager = null)
    {
        var warnings = new List<string>();

        if (options.Chocolatey.Enable && !IsChocolateyInstalled())
            return (false, "Chocolatey is enabled but not installed. Install from https://chocolatey.org/install", warnings);

        if (options.Scoop.Enable && !IsScoopInstalled())
            return (false, "Scoop is enabled but not installed. Install from https://scoop.sh", warnings);

        if (options.Winget.Enable)
        {
            var mgr = wingetManager ?? new WingetManager();
            if (!mgr.IsInstalled)
            {
                await mgr.EnsureInstalledAsync();
                if (!mgr.IsInstalled)
                    return (false, "Winget is enabled but not installed. Auto-install failed. Install from https://github.com/microsoft/winget-cli", warnings);
            }
        }

        if (config.ChocoPackages.Count > 0 && !options.Chocolatey.Enable)
            warnings.Add("[warn] Chocolatey packages declared but chocolatey not enabled in options");

        if (config.ScoopPackages.Count > 0 && !options.Scoop.Enable)
            warnings.Add("[warn] Scoop packages declared but scoop not enabled in options");

        return (true, null, warnings);
    }

    public static (bool Valid, string? Error, List<string> Warnings) ValidateManagers(
        NTIXOptions options,
        NTIXConfig config)
    {
        var warnings = new List<string>();
        options ??= new NTIXOptions();

        if ((options.Chocolatey?.Enable ?? false) && !IsChocolateyInstalled())
            return (false, "Chocolatey is enabled but not installed. Install from https://chocolatey.org/install", warnings);

        if ((options.Scoop?.Enable ?? false) && !IsScoopInstalled())
            return (false, "Scoop is enabled but not installed. Install from https://scoop.sh (make sure to install as admin)", warnings);

        if (config.ChocoPackages.Count > 0 && !(options.Chocolatey?.Enable ?? false))
            warnings.Add("[warn] Chocolatey packages declared but chocolatey not enabled in options");

        if (config.ScoopPackages.Count > 0 && !(options.Scoop?.Enable ?? false))
            warnings.Add("[warn] Scoop packages declared but scoop not enabled in options");

        return (true, null, warnings);
    }

    public static async Task<InstalledPackages> GetInstalledPackagesAsync(Func<IWingetManager>? wingetFactory = null)
    {
        var result = new InstalledPackages();
        var factory = wingetFactory ?? (() => new WingetManager());
        var wingetManager = factory();

        try
        {
            var wingetPackages = await wingetManager.GetInstalledPackagesAsync();
            foreach (var kvp in wingetPackages)
            {
                result.Winget[kvp.Key] = kvp.Value;
            }
        }
        catch { }

        var chocoOut = RunProcess("choco list -r --local-only --limit-output 2>nul", redirectStderr: true);
        if (!string.IsNullOrEmpty(chocoOut))
        {
            var regex = new Regex(@"^([^|]+)\|([^|]+)\|.*$", RegexOptions.Multiline);
            foreach (Match m in regex.Matches(chocoOut))
            {
                var id = m.Groups[1].Value.Trim();
                var ver = m.Groups[2].Value.Trim();
                if (!string.IsNullOrEmpty(id) && !string.IsNullOrEmpty(ver))
                    result.Chocolatey[id] = ver;
            }
        }

        var scoopOut = RunProcess("scoop list --local-only --limit-output 2>nul", redirectStderr: true);
        if (!string.IsNullOrEmpty(scoopOut))
        {
            var regex = new Regex(@"^([^\s]+)\s+([^\s]+)\s+.*$", RegexOptions.Multiline);
            foreach (Match m in regex.Matches(scoopOut))
            {
                var id = m.Groups[1].Value.Trim();
                var ver = m.Groups[2].Value.Trim();
                if (!string.IsNullOrEmpty(id) && !string.IsNullOrEmpty(ver))
                    result.Scoop[id] = ver;
            }
        }

        return result;
    }

    public static InstalledPackages GetInstalledPackages() => GetInstalledPackagesAsync().GetAwaiter().GetResult();

    public static async Task<Dictionary<string, UpgradeInfo>> GetWingetUpgradablePackagesAsync(Func<IWingetManager>? wingetFactory = null)
    {
        var factory = wingetFactory ?? (() => new WingetManager());
        var wingetManager = factory();
        return await wingetManager.GetUpgradablePackagesAsync();
    }

    public static Dictionary<string, UpgradeInfo> GetWingetUpgradablePackages(Func<IWingetManager>? wingetFactory = null) => GetWingetUpgradablePackagesAsync(wingetFactory).GetAwaiter().GetResult();

    public static Dictionary<string, UpgradeInfo> GetChocoUpgradablePackages()
    {
        var result = new Dictionary<string, UpgradeInfo>();
        var output = RunProcess("choco outdated --limit-output 2>nul", redirectStderr: true);
        if (string.IsNullOrEmpty(output)) return result;

        var regex = new Regex(@"^([^|]+)\|([^|]+)\|([^|]+)\|.*$", RegexOptions.Multiline);
        foreach (Match m in regex.Matches(output))
        {
            var id = m.Groups[1].Value.Trim();
            var cur = m.Groups[2].Value.Trim();
            var avail = m.Groups[3].Value.Trim();
            if (!string.IsNullOrEmpty(id) && !string.IsNullOrEmpty(cur) && !string.IsNullOrEmpty(avail))
                result[id] = new UpgradeInfo(cur, avail);
        }
        return result;
    }

    public static Dictionary<string, UpgradeInfo> GetScoopUpgradablePackages()
    {
        var result = new Dictionary<string, UpgradeInfo>();
        var output = RunProcess("scoop status --json 2>nul", redirectStderr: true);
        if (string.IsNullOrEmpty(output)) return result;

        try
        {
            using var doc = JsonDocument.Parse(output);
            if (doc.RootElement.ValueKind == JsonValueKind.Array)
            {
                foreach (var item in doc.RootElement.EnumerateArray())
                {
                    var id = item.GetProperty("name").GetString();
                    var cur = item.GetProperty("current_version").GetString();
                    var avail = item.GetProperty("latest_version").GetString();
                    if (!string.IsNullOrEmpty(id) && !string.IsNullOrEmpty(cur) && !string.IsNullOrEmpty(avail) && cur != avail)
                        result[id] = new UpgradeInfo(cur, avail);
                }
            }
        }
        catch { }
        return result;
    }

    public static async Task<Dictionary<string, UpgradeInfo>> GetAllUpgradablePackagesAsync(Func<IWingetManager>? wingetFactory = null)
    {
        var winget = await GetWingetUpgradablePackagesAsync(wingetFactory);
        var choco = GetChocoUpgradablePackages();
        var scoop = GetScoopUpgradablePackages();

        var result = new Dictionary<string, UpgradeInfo>();
        foreach (var kvp in winget) result[kvp.Key] = kvp.Value;
        foreach (var kvp in choco) result[kvp.Key] = kvp.Value;
        foreach (var kvp in scoop) result[kvp.Key] = kvp.Value;
        return result;
    }

    public static bool ValidateChocoPackageExists(string id)
    {
        var output = RunProcess(CommandBuilder.BuildChocoSearch(id), redirectStderr: true);
        if (string.IsNullOrEmpty(output)) return false;
        var regex = new Regex($@"^{Regex.Escape(id)}\|", RegexOptions.Multiline | RegexOptions.IgnoreCase);
        return regex.IsMatch(output);
    }

    public static bool ValidateScoopPackageExists(string id)
    {
        var output = RunProcess(CommandBuilder.BuildScoopInfo(id), redirectStderr: true);
        if (string.IsNullOrEmpty(output)) return false;
        var pattern = new Regex(@"^\s*Name\s*:", RegexOptions.Multiline | RegexOptions.IgnoreCase);
        return pattern.IsMatch(output);
    }

    internal static string? RunProcess(string cmd, bool redirectStderr = false)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "cmd.exe",
                Arguments = redirectStderr
                    ? $"/c {cmd} 2>&1"
                    : $"/c {cmd} 2>nul",
                RedirectStandardOutput = true,
                UseShellExecute = false,
                CreateNoWindow = true,
                StandardOutputEncoding = System.Text.Encoding.UTF8
            };

            using var p = Process.Start(psi);
            if (p == null) return null;
            var output = p.StandardOutput.ReadToEnd();
            p.WaitForExit();
            return output.Trim();
        }
        catch { return null; }
    }
}