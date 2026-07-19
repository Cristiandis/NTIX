using System.Text.RegularExpressions;
using NTIX.Core.Models;

namespace NTIX.Core.PackageManager;

public static class CommandBuilder
{
    private static readonly Regex SafeIdPattern = new(@"^[a-zA-Z0-9._\-/]+$", RegexOptions.Compiled);

    public static string SanitizeId(string id)
    {
        if (string.IsNullOrWhiteSpace(id))
            throw new ArgumentException("Package ID cannot be empty");
        if (!SafeIdPattern.IsMatch(id))
            throw new ArgumentException($"Package ID contains invalid characters: {id}");
        return id;
    }

    public static string BuildChocoInstall(string id, string? version, ChocoOptions opts)
    {
        var cmd = $"choco install {SanitizeId(id)}";
        if (!string.IsNullOrEmpty(version))
            cmd += $" --version {SanitizeId(version)}";
        if (opts.Yes) cmd += " -y";
        if (opts.Force) cmd += " --force";
        if (opts.IgnoreDependencies) cmd += " --ignore-dependencies";
        if (opts.AllowDowngrade) cmd += " --allow-downgrade";
        if (opts.SkipPowerShell) cmd += " --skip-scripts";
        if (opts.Pre) cmd += " --pre";
        if (!string.IsNullOrEmpty(opts.Params))
            cmd += $" --params=\"'{opts.Params}'\"";
        return cmd;
    }

    public static string BuildScoopInstall(string id, string? version, ScoopOptions opts)
    {
        var safeId = SanitizeId(id);
        var cmd = string.IsNullOrEmpty(version)
            ? $"scoop install {safeId}"
            : $"scoop install {safeId}@{SanitizeId(version)}";
        if (opts.Global) cmd += " -g";
        if (opts.Independent) cmd += " -i";
        if (opts.NoCache) cmd += " -k";
        if (opts.SkipHashCheck) cmd += " -s";
        if (!string.IsNullOrEmpty(opts.Arch))
            cmd += $" --arch {opts.Arch}";
        return cmd;
    }

    public static string BuildChocoUpgrade(string id, ChocoOptions opts)
    {
        var cmd = $"choco upgrade {SanitizeId(id)}";
        if (opts.Yes) cmd += " -y";
        if (opts.Force) cmd += " --force";
        if (opts.Pre) cmd += " --pre";
        return cmd;
    }

    public static string BuildScoopUpgrade(string id, ScoopOptions opts)
    {
        var cmd = $"scoop update {SanitizeId(id)}";
        if (opts.Global) cmd += " -g";
        return cmd;
    }

    public static string BuildChocoUninstall(string id, ChocoOptions opts)
    {
        var cmd = $"choco uninstall {SanitizeId(id)}";
        if (opts.Yes) cmd += " -y";
        if (opts.Force) cmd += " --force";
        if (opts.IgnoreDependencies) cmd += " --ignore-dependencies";
        return cmd;
    }

    public static string BuildScoopUninstall(string id, ScoopOptions opts)
    {
        var cmd = $"scoop uninstall {SanitizeId(id)}";
        if (opts.Global) cmd += " -g";
        return cmd;
    }

    public static string BuildChocoSearch(string id)
    {
        return $"choco search {SanitizeId(id)} --limit-output";
    }

    public static string BuildScoopInfo(string id)
    {
        return $"scoop info {SanitizeId(id)}";
    }

    public static string BuildScoopBucketAdd(string name, string? url)
    {
        return string.IsNullOrEmpty(url)
            ? $"scoop bucket add {SanitizeId(name)}"
            : $"scoop bucket add {SanitizeId(name)} {url}";
    }

    public static string BuildScoopBucketList()
    {
        return "scoop bucket list";
    }
}