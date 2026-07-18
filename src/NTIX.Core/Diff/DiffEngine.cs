using NTIX.Core.Models;
using NTIX.Core.PackageManager;

namespace NTIX.Core.Diff;

public static class DiffEngine
{
    public static DiffResult ComputeDiff(
        NTIXConfig config, 
        State state, 
        InstalledPackages? installed = null,
        IWingetManager? wingetManager = null)
    {
        var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(config.Options, config);
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

        foreach (var w in warnings)
            Console.Error.WriteLine($"[warn] {w}");

        var result = new DiffResult();
        var installedPkgs = installed ?? PackageManagerDetector.GetInstalledPackages();

        var wingetInstalled = new HashSet<string>(installedPkgs.Winget.Keys, StringComparer.OrdinalIgnoreCase);
        var chocoInstalled = new HashSet<string>(installedPkgs.Chocolatey.Keys, StringComparer.OrdinalIgnoreCase);
        var scoopInstalled = new HashSet<string>(installedPkgs.Scoop.Keys, StringComparer.OrdinalIgnoreCase);

        var hasWingetUnpinned = config.WingetPackages.Any(p => p.Version == null);
        var hasChocoUnpinned = config.ChocoPackages.Any(p => p.Version == null);
        var hasScoopUnpinned = config.ScoopPackages.Any(p => p.Version == null);

        var wingetEnabled = config.Options?.Winget?.Enable ?? false;
        var chocoEnabled = config.Options?.Chocolatey?.Enable ?? false;
        var scoopEnabled = config.Options?.Scoop?.Enable ?? false;

        var wingetUpgradable = (hasWingetUnpinned && wingetEnabled) 
            ? PackageManagerDetector.GetWingetUpgradablePackages(() => wingetManager ?? new WingetManager()) 
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

        FindOrphans(result, state.Winget, config.WingetPackages, "winget");
        FindOrphans(result, state.Chocolatey, config.ChocoPackages, "chocolatey");
        FindOrphans(result, state.Scoop, config.ScoopPackages, "scoop");

        return result;
    }

    public static void PrintDiff(DiffResult diff)
    {
        if (!string.IsNullOrEmpty(diff.Error))
        {
            Console.Error.WriteLine($"[error] {diff.Error}");
            foreach (var w in diff.Warnings)
                Console.Error.WriteLine($"[warn] {w}");
            return;
        }

        if (diff.ToInstall.Count > 0)
        {
            Console.WriteLine("To install:");
            foreach (var p in diff.ToInstall)
                Console.WriteLine($"  {p.Source}: {p.Id} ({p.Version ?? "latest"})");
        }

        if (diff.ToUpgrade.Count > 0)
        {
            Console.WriteLine("To upgrade:");
            foreach (var p in diff.ToUpgrade)
                Console.WriteLine($"  {p.Source}: {p.Id} ({p.Version ?? "latest"})");
        }

        if (diff.ToSkip.Count > 0)
        {
            Console.WriteLine("Already installed (skip):");
            foreach (var p in diff.ToSkip)
                Console.WriteLine($"  {p.Source}: {p.Id} ({p.Version ?? "latest"})");
        }

        if (diff.ToRemove.Count > 0)
        {
            Console.WriteLine("To remove:");
            foreach (var p in diff.ToRemove)
                Console.WriteLine($"  {p.Source}: {p.Id} ({p.Version ?? "latest"})");
        }

        if (diff.IsEmpty)
        {
            Console.WriteLine("Nothing to do.");
        }

        foreach (var w in diff.Warnings)
            Console.Error.WriteLine($"[warn] {w}");
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
}