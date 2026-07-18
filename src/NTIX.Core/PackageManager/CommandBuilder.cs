using NTIX.Core.Models;

namespace NTIX.Core.PackageManager;

public static class CommandBuilder
{
    public static string BuildChocoInstall(string id, string? version, bool yes)
    {
        var cmd = $"choco install {id}";
        if (!string.IsNullOrEmpty(version))
            cmd += $" --version {version}";
        if (yes)
            cmd += " -y";
        return cmd;
    }

    public static string BuildScoopInstall(string id, string? version)
    {
        return string.IsNullOrEmpty(version)
            ? $"scoop install {id}"
            : $"scoop install {id}@{version}";
    }

    public static string BuildChocoUpgrade(string id, bool yes)
    {
        return $"choco upgrade {id}" + (yes ? " -y" : "");
    }

    public static string BuildScoopUpgrade(string id)
    {
        return $"scoop update {id}";
    }

    public static string BuildChocoUninstall(string id, bool yes)
    {
        return $"choco uninstall {id}" + (yes ? " -y" : "");
    }

    public static string BuildScoopUninstall(string id)
    {
        return $"scoop uninstall {id}";
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