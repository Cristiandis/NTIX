using NTIX.Core.Models;
using NTIX.Core.PackageManager;

namespace NTIX.Core.Diff;

public static class DiffEngine
{
    public static async Task<DiffResult> ComputeDiffAsync(
        NTIXConfig config,
        State state,
        InstalledPackages? installed = null,
        IWingetManager? wingetManager = null,
        bool validatePackages = true,
        IProgress<string>? progress = null)
    {
        progress?.Report("Checking package managers...");
        var (valid, error, warnings) = await PackageManagerDetector.ValidateManagersAsync(config.Options, config, wingetManager);
        if (!valid)
        {
            return new DiffResult(
                ToInstall: new(),
                ToUpgrade: new(),
                ToRemove: new(),
                ToSkip: new(),
                Error: error,
                Warnings: warnings
            );
        }

        var result = new DiffResult();
        result.Warnings.AddRange(warnings);

        progress?.Report("Discovering installed packages...");
        var installedPkgs = installed ?? await PackageManagerDetector.GetInstalledPackagesAsync();

        var wingetInstalled = new HashSet<string>(installedPkgs.Winget.Keys, StringComparer.OrdinalIgnoreCase);
        var chocoInstalled = new HashSet<string>(installedPkgs.Chocolatey.Keys, StringComparer.OrdinalIgnoreCase);
        var scoopInstalled = new HashSet<string>(installedPkgs.Scoop.Keys, StringComparer.OrdinalIgnoreCase);

        var hasWingetUnpinned = config.WingetPackages.Any(p => p.Version == null);
        var hasChocoUnpinned = config.ChocoPackages.Any(p => p.Version == null);
        var hasScoopUnpinned = config.ScoopPackages.Any(p => p.Version == null);

        var wingetEnabled = config.Options?.Winget?.Enable ?? false;
        var chocoEnabled = config.Options?.Chocolatey?.Enable ?? false;
        var scoopEnabled = config.Options?.Scoop?.Enable ?? false;

        progress?.Report("Checking for updates...");
        var wingetUpgradable = (hasWingetUnpinned && wingetEnabled)
            ? await PackageManagerDetector.GetWingetUpgradablePackagesAsync(() => wingetManager ?? new WingetManager())
            : new Dictionary<string, UpgradeInfo>();
        var chocoUpgradable = (hasChocoUnpinned && chocoEnabled)
            ? PackageManagerDetector.GetChocoUpgradablePackages()
            : new Dictionary<string, UpgradeInfo>();
        var scoopUpgradable = (hasScoopUnpinned && scoopEnabled)
            ? PackageManagerDetector.GetScoopUpgradablePackages()
            : new Dictionary<string, UpgradeInfo>();

        ClassifyPackages(result, config.WingetPackages, "winget", wingetEnabled, wingetInstalled, state.Winget, wingetUpgradable);
        ClassifyPackages(result, config.ChocoPackages, "chocolatey", chocoEnabled, chocoInstalled, state.Chocolatey, chocoUpgradable);
        ClassifyPackages(result, config.ScoopPackages, "scoop", scoopEnabled, scoopInstalled, state.Scoop, scoopUpgradable);

        if (validatePackages)
        {
            progress?.Report("Validating packages...");
            await ValidatePackageAvailabilityAsync(result, wingetManager, wingetEnabled, chocoEnabled, scoopEnabled);
        }

        progress?.Report("Finding orphans...");
        FindOrphans(result, state.Winget, config.WingetPackages, "winget");
        FindOrphans(result, state.Chocolatey, config.ChocoPackages, "chocolatey");
        FindOrphans(result, state.Scoop, config.ScoopPackages, "scoop");

        return result;
    }

    private static void ClassifyPackages(
        DiffResult result,
        List<PackageEntry> packages,
        string sourceName,
        bool enabled,
        HashSet<string> installed,
        Dictionary<string, string> stateDict,
        Dictionary<string, UpgradeInfo> upgradable)
    {
        if (!enabled) return;

        foreach (var pkg in packages)
        {
            var spec = new PackageSpec(pkg.Id, pkg.Version, sourceName);
            var isInstalled = installed.Contains(pkg.Id);
            var inState = stateDict.ContainsKey(pkg.Id);

            if (pkg.Version == null)
            {
                if (upgradable.TryGetValue(pkg.Id, out var upgrade))
                {
                    spec = spec with { Version = upgrade.AvailableVersion };
                    result.ToUpgrade.Add(spec);
                }
                else if (!isInstalled && !inState)
                {
                    result.ToInstall.Add(spec);
                }
                else if (isInstalled)
                {
                    result.ToSkip.Add(spec);
                }
                else if (inState)
                {
                    result.ToInstall.Add(spec);
                }
            }
            else
            {
                if (inState)
                {
                    var stateVersion = stateDict[pkg.Id];
                    if (!string.Equals(stateVersion, pkg.Version, StringComparison.OrdinalIgnoreCase))
                        result.ToInstall.Add(spec);
                    else
                        result.ToSkip.Add(spec);
                }
                else
                {
                    result.ToInstall.Add(spec);
                }
            }
        }
    }

    private static void FindOrphans(
        DiffResult result,
        Dictionary<string, string> stateDict,
        List<PackageEntry> configPackages,
        string sourceName)
    {
        foreach (var (id, ver) in stateDict)
        {
            if (!configPackages.Any(p => string.Equals(p.Id, id, StringComparison.OrdinalIgnoreCase)))
                result.ToRemove.Add(new PackageSpec(id, ver, sourceName));
        }
    }

    private static async Task ValidatePackageAvailabilityAsync(
        DiffResult result,
        IWingetManager? wingetManager,
        bool wingetEnabled,
        bool chocoEnabled,
        bool scoopEnabled)
    {
        var invalid = new List<PackageSpec>();

        var wingetPkgs = result.ToInstall.Where(p => p.Source == "winget").ToList();
        var chocoPkgs = result.ToInstall.Where(p => p.Source == "chocolatey").ToList();
        var scoopPkgs = result.ToInstall.Where(p => p.Source == "scoop").ToList();

        if (wingetEnabled && wingetPkgs.Count > 0)
        {
            var mgr = wingetManager ?? new WingetManager();
            foreach (var pkg in wingetPkgs)
            {
                try
                {
                    if (!await mgr.PackageExistsAsync(pkg.Id))
                    {
                        result.Warnings.Add($"Package not found in winget: {pkg.Id}");
                        invalid.Add(pkg);
                    }
                }
                catch
                {
                    result.Warnings.Add($"Could not verify package in winget: {pkg.Id}");
                }
            }
        }

        if (chocoEnabled && chocoPkgs.Count > 0)
        {
            foreach (var pkg in chocoPkgs)
            {
                try
                {
                    if (!PackageManagerDetector.ValidateChocoPackageExists(pkg.Id))
                    {
                        result.Warnings.Add($"Package not found in chocolatey: {pkg.Id}");
                        invalid.Add(pkg);
                    }
                }
                catch
                {
                    result.Warnings.Add($"Could not verify package in chocolatey: {pkg.Id}");
                }
            }
        }

        if (scoopEnabled && scoopPkgs.Count > 0)
        {
            foreach (var pkg in scoopPkgs)
            {
                try
                {
                    if (!PackageManagerDetector.ValidateScoopPackageExists(pkg.Id))
                    {
                        result.Warnings.Add($"Package not found in scoop: {pkg.Id}");
                        invalid.Add(pkg);
                    }
                }
                catch
                {
                    result.Warnings.Add($"Could not verify package in scoop: {pkg.Id}");
                }
            }
        }

        foreach (var pkg in invalid)
            result.ToInstall.Remove(pkg);
    }
}