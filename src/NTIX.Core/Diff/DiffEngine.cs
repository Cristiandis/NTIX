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
        bool adoptMode = false,
        bool upgradeMode = false,
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

        var hasWingetUnpinned = config.WingetPackages.Any(p => p.Version == null);
        var hasChocoUnpinned = config.ChocoPackages.Any(p => p.Version == null);
        var hasScoopUnpinned = config.ScoopPackages.Any(p => p.Version == null);

        var wingetEnabled = config.Options?.Winget?.Enable ?? false;
        var chocoEnabled = config.Options?.Chocolatey?.Enable ?? false;
        var scoopEnabled = config.Options?.Scoop?.Enable ?? false;

        progress?.Report("Checking for updates...");
        var wingetUpgradable = (upgradeMode && hasWingetUnpinned && wingetEnabled)
            ? await PackageManagerDetector.GetWingetUpgradablePackagesAsync(() => wingetManager ?? new WingetManager())
            : new Dictionary<string, UpgradeInfo>();
        var chocoUpgradable = (upgradeMode && hasChocoUnpinned && chocoEnabled)
            ? PackageManagerDetector.GetChocoUpgradablePackages()
            : new Dictionary<string, UpgradeInfo>();
        var scoopUpgradable = (upgradeMode && hasScoopUnpinned && scoopEnabled)
            ? PackageManagerDetector.GetScoopUpgradablePackages()
            : new Dictionary<string, UpgradeInfo>();

        ClassifyPackages(result, config.WingetPackages, "winget", wingetEnabled, installedPkgs.Winget, state.Winget, wingetUpgradable, adoptMode);
        ClassifyPackages(result, config.ChocoPackages, "chocolatey", chocoEnabled, installedPkgs.Chocolatey, state.Chocolatey, chocoUpgradable, adoptMode);
        ClassifyPackages(result, config.ScoopPackages, "scoop", scoopEnabled, installedPkgs.Scoop, state.Scoop, scoopUpgradable, adoptMode);

        if (validatePackages)
        {
            progress?.Report("Validating packages...");
            await ValidatePackageAvailabilityAsync(result, state, wingetManager, wingetEnabled, chocoEnabled, scoopEnabled);
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
        Dictionary<string, string> installedDict,
        Dictionary<string, string> stateDict,
        Dictionary<string, UpgradeInfo> upgradable,
        bool adoptMode)
    {
        if (!enabled) return;

        foreach (var pkg in packages)
        {
            var spec = new PackageSpec(pkg.Id, pkg.Version, sourceName);
            var isInstalled = installedDict.ContainsKey(pkg.Id);
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
                else if (isInstalled && inState)
                {
                    result.ToSkip.Add(spec);
                }
                else if (isInstalled && adoptMode)
                {
                    result.ToAdopt.Add(spec);
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
                else if (isInstalled && adoptMode)
                {
                    var installedVersion = installedDict[pkg.Id];
                    if (string.Equals(installedVersion, pkg.Version, StringComparison.OrdinalIgnoreCase))
                        result.ToAdopt.Add(spec with { Version = installedVersion });
                    else
                        result.ToInstall.Add(spec);
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
        State state,
        IWingetManager? wingetManager,
        bool wingetEnabled,
        bool chocoEnabled,
        bool scoopEnabled)
    {
        var invalid = new List<PackageSpec>();

        var wingetPkgs = result.ToInstall.Where(p => p.Source == "winget").ToList();
        var chocoPkgs = result.ToInstall.Where(p => p.Source == "chocolatey").ToList();
        var scoopPkgs = result.ToInstall.Where(p => p.Source == "scoop").ToList();

        var newWingetPkgs = wingetPkgs.Where(p => !state.Winget.ContainsKey(p.Id)).ToList();
        var newChocoPkgs = chocoPkgs.Where(p => !state.Chocolatey.ContainsKey(p.Id)).ToList();
        var newScoopPkgs = scoopPkgs.Where(p => !state.Scoop.ContainsKey(p.Id)).ToList();

        var validationTasks = new List<Task>();

        if (wingetEnabled && newWingetPkgs.Count > 0)
        {
            validationTasks.Add(Task.Run(async () =>
            {
                var results = await PackageManagerDetector.ValidateWingetPackagesExistsAsync(
                    newWingetPkgs.Select(p => p.Id), wingetManager);
                foreach (var pkg in newWingetPkgs)
                {
                    if (results.TryGetValue(pkg.Id, out var exists))
                    {
                        if (exists == false)
                        {
                            result.Warnings.Add($"Package not found in winget: {pkg.Id}");
                            lock (invalid) invalid.Add(pkg);
                        }
                        else if (exists == null)
                        {
                            result.Warnings.Add($"Could not verify package in winget: {pkg.Id}");
                        }
                    }
                    else
                    {
                        result.Warnings.Add($"Could not verify package in winget: {pkg.Id}");
                    }
                }
            }));
        }

        if (chocoEnabled && newChocoPkgs.Count > 0)
        {
            validationTasks.Add(Task.Run(async () =>
            {
                var results = await PackageManagerDetector.ValidateChocoPackagesExistsAsync(
                    newChocoPkgs.Select(p => p.Id));
                foreach (var pkg in newChocoPkgs)
                {
                    if (results.TryGetValue(pkg.Id, out var exists))
                    {
                        if (!exists)
                        {
                            result.Warnings.Add($"Package not found in chocolatey: {pkg.Id}");
                            lock (invalid) invalid.Add(pkg);
                        }
                    }
                    else
                    {
                        result.Warnings.Add($"Could not verify package in chocolatey: {pkg.Id}");
                    }
                }
            }));
        }

        if (scoopEnabled && newScoopPkgs.Count > 0)
        {
            validationTasks.Add(Task.Run(async () =>
            {
                var results = await PackageManagerDetector.ValidateScoopPackagesExistsAsync(
                    newScoopPkgs.Select(p => p.Id));
                foreach (var pkg in newScoopPkgs)
                {
                    if (results.TryGetValue(pkg.Id, out var exists))
                    {
                        if (!exists)
                        {
                            result.Warnings.Add($"Package not found in scoop: {pkg.Id}");
                            lock (invalid) invalid.Add(pkg);
                        }
                    }
                    else
                    {
                        result.Warnings.Add($"Could not verify package in scoop: {pkg.Id}");
                    }
                }
            }));
        }

        if (validationTasks.Count > 0)
            await Task.WhenAll(validationTasks);

        foreach (var pkg in invalid)
            result.ToInstall.Remove(pkg);
    }
}