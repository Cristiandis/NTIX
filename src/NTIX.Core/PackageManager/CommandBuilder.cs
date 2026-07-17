namespace NTIX.Core.PackageManager;

public static class CommandBuilder
{
    public static string BuildWingetInstall(string id, string? version, bool acceptAgreements, bool interactive)
    {
        var cmd = $"winget install --id {id}";
        if (!string.IsNullOrEmpty(version))
            cmd += $" --version {version}";
        if (acceptAgreements)
            cmd += " --accept-source-agreements --accept-package-agreements";
        if (!interactive)
            cmd += " --silent";
        return cmd;
    }

    public static string BuildChocoInstall(string id, string? version, bool yes)
    {
        var cmd = $"choco install {id}";
        if (!string.IsNullOrEmpty(version))
            cmd += $" --version {version}";
        if (yes)
            cmd += " -y";
        return cmd;
    }

    public static string BuildScoopInstall(string id, string? version, List<string> buckets)
    {
        var cmd = $"scoop install {id}";
        if (!string.IsNullOrEmpty(version))
            cmd += $" --version {version}";
        foreach (var bucket in buckets)
        {
            cmd += $" --bucket {bucket}";
        }
        return cmd;
    }

    public static string BuildWingetUpgrade(string id, bool acceptAgreements, bool interactive)
    {
        var cmd = $"winget upgrade --id {id}";
        if (acceptAgreements)
            cmd += " --accept-source-agreements --accept-package-agreements";
        if (!interactive)
            cmd += " --silent";
        return cmd;
    }

    public static string BuildChocoUpgrade(string id, bool yes)
    {
        return $"choco upgrade {id}" + (yes ? " -y" : "");
    }

    public static string BuildScoopUpgrade(string id)
    {
        return $"scoop update {id}";
    }

    public static string BuildWingetUninstall(string id, bool acceptAgreements, bool interactive)
    {
        var cmd = $"winget uninstall --id {id}";
        if (acceptAgreements)
            cmd += " --accept-source-agreements --accept-package-agreements";
        if (!interactive)
            cmd += " --silent";
        return cmd;
    }

    public static string BuildChocoUninstall(string id, bool yes)
    {
        return $"choco uninstall {id}" + (yes ? " -y" : "");
    }

    public static string BuildScoopUninstall(string id)
    {
        return $"scoop uninstall {id}";
    }
}