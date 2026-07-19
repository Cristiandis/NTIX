using NTIX.Core.Models;

namespace NTIX.Core.PackageManager;

public static class CommandBuilder
{
    public static string BuildChocoInstall(string id, string? version, ChocoOptions opts)
    {
        var cmd = $"choco install {id}";
        if (!string.IsNullOrEmpty(version))
            cmd += $" --version {version}";
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
        var cmd = string.IsNullOrEmpty(version)
            ? $"scoop install {id}"
            : $"scoop install {id}@{version}";
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
        var cmd = $"choco upgrade {id}";
        if (opts.Yes) cmd += " -y";
        if (opts.Force) cmd += " --force";
        if (opts.Pre) cmd += " --pre";
        return cmd;
    }

    public static string BuildScoopUpgrade(string id, ScoopOptions opts)
    {
        var cmd = $"scoop update {id}";
        if (opts.Global) cmd += " -g";
        return cmd;
    }

    public static string BuildChocoUninstall(string id, ChocoOptions opts)
    {
        var cmd = $"choco uninstall {id}";
        if (opts.Yes) cmd += " -y";
        if (opts.Force) cmd += " --force";
        if (opts.IgnoreDependencies) cmd += " --ignore-dependencies";
        return cmd;
    }

    public static string BuildScoopUninstall(string id, ScoopOptions opts)
    {
        var cmd = $"scoop uninstall {id}";
        if (opts.Global) cmd += " -g";
        return cmd;
    }

    public static string BuildChocoSearch(string id)
    {
        return $"choco search {id} --limit-output";
    }

    public static string BuildScoopInfo(string id)
    {
        return $"scoop info {id}";
    }

    public static string BuildScoopBucketAdd(string name, string? url)
    {
        return string.IsNullOrEmpty(url)
            ? $"scoop bucket add {name}"
            : $"scoop bucket add {name} {url}";
    }

    public static string BuildScoopBucketList()
    {
        return "scoop bucket list";
    }
}