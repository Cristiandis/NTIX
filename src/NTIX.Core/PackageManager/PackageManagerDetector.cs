using System.Diagnostics;
using System.Text.RegularExpressions;
using System.Text.Json;
using NTIX.Core.Models;

namespace NTIX.Core.PackageManager;

public static class PackageManagerDetector
{
    public static PMStatus Detect() => new(
        IsWingetInstalled(),
        IsChocolateyInstalled(),
        IsScoopInstalled()
    );

    public static bool IsWingetInstalled() => RunCommandHidden("winget --version") != null;
    public static bool IsChocolateyInstalled() => RunCommandHidden("choco --version") != null;
    public static bool IsScoopInstalled() => RunCommandHidden("scoop --version") != null;

    public static bool IsRunningAsAdmin()
    {
        try
        {
            var identity = System.Security.Principal.WindowsIdentity.GetCurrent();
            var principal = new System.Security.Principal.WindowsPrincipal(identity);
            return principal.IsInRole(System.Security.Principal.WindowsBuiltInRole.Administrator);
        }
        catch { return false; }
    }

    public static InstalledPackages GetInstalledPackages()
    {
        var result = new InstalledPackages();

        // Winget
        var wingetOut = RunCommand("winget export -o - --accept-source-agreements --accept-package-agreements 2>nul");
        if (!string.IsNullOrEmpty(wingetOut))
        {
            try
            {
                using var doc = JsonDocument.Parse(wingetOut);
                if (doc.RootElement.ValueKind == JsonValueKind.Array)
                {
                    foreach (var item in doc.RootElement.EnumerateArray())
                    {
                        var id = item.GetProperty("PackageIdentifier").GetString();
                        var ver = item.GetProperty("Version").GetString();
                        if (!string.IsNullOrEmpty(id) && !string.IsNullOrEmpty(ver))
                            result.Winget[id] = ver;
                    }
                }
            }
            catch { }
        }

        // Chocolatey
        var chocoOut = RunCommand("choco list -r --local-only --limit-output 2>nul");
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

        // Scoop
        var scoopOut = RunCommand("scoop list --local-only --limit-output 2>nul");
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

    public static Dictionary<string, UpgradeInfo> GetWingetUpgradablePackages()
    {
        var result = new Dictionary<string, UpgradeInfo>();
        var output = RunCommand("winget list --upgrade-available --accept-source-agreements 2>nul");
        if (string.IsNullOrEmpty(output)) return result;

        var lines = output.Split('\n');
        var colRegex = new Regex(@"\s{2,}");

        for (int i = 2; i < lines.Length; i++) // Skip header and separator
        {
            var line = lines[i].Trim();
            if (string.IsNullOrEmpty(line)) continue;

            var cols = colRegex.Split(line);
            if (cols.Length >= 5)
            {
                var id = cols[1].Trim();
                var cur = cols[2].Trim();
                var avail = cols[3].Trim();
                if (!string.IsNullOrEmpty(id) && !string.IsNullOrEmpty(cur) && !string.IsNullOrEmpty(avail))
                    result[id] = new UpgradeInfo(cur, avail);
            }
        }
        return result;
    }

    public static Dictionary<string, UpgradeInfo> GetChocoUpgradablePackages()
    {
        var result = new Dictionary<string, UpgradeInfo>();
        var output = RunCommand("choco outdated --limit-output 2>nul");
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
        var output = RunCommand("scoop status --json 2>nul");
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

    private static string? RunCommandHidden(string cmd)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "cmd.exe",
                Arguments = $"/c {cmd} 2>nul",
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

    private static string RunCommand(string cmd)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "cmd.exe",
                Arguments = $"/c {cmd} 2>&1",
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true,
                StandardOutputEncoding = System.Text.Encoding.UTF8,
                StandardErrorEncoding = System.Text.Encoding.UTF8
            };
            using var p = Process.Start(psi);
            if (p == null) return string.Empty;
            var output = p.StandardOutput.ReadToEnd();
            p.WaitForExit();
            return output.Trim();
        }
        catch { return string.Empty; }
    }
}

public record PMStatus(bool Winget = false, bool Chocolatey = false, bool Scoop = false);