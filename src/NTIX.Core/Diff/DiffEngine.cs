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

        if (wingetEnabled)
        {
            foreach (var pkg in config.WingetPackages)
            {
                var spec = new PackageSpec(pkg.Id, pkg.Version, "winget");
                var isInstalled = wingetInstalled.Contains(pkg.Id);
                var inState = state.Winget.ContainsKey(pkg.Id);

                if (pkg.Version == null)
                {
                    if (wingetUpgradable.TryGetValue(pkg.Id, out var upgrade))
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
                        result.ToSkip.Add(spec);
                    else
                        result.ToInstall.Add(spec);
                }
            }
        }

        if (chocoEnabled)
        {
            foreach (var pkg in config.ChocoPackages)
            {
                var spec = new PackageSpec(pkg.Id, pkg.Version, "chocolatey");
                var isInstalled = chocoInstalled.Contains(pkg.Id);
                var inState = state.Chocolatey.ContainsKey(pkg.Id);

                if (pkg.Version == null)
                {
                    if (chocoUpgradable.TryGetValue(pkg.Id, out var upgrade))
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
                        result.ToSkip.Add(spec);
                    else
                        result.ToInstall.Add(spec);
                }
            }
        }

        if (scoopEnabled)
        {
            foreach (var pkg in config.ScoopPackages)
            {
                var spec = new PackageSpec(pkg.Id, pkg.Version, "scoop");
                var isInstalled = scoopInstalled.Contains(pkg.Id);
                var inState = state.Scoop.ContainsKey(pkg.Id);

                if (pkg.Version == null)
                {
                    if (scoopUpgradable.TryGetValue(pkg.Id, out var upgrade))
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
                        result.ToSkip.Add(spec);
                    else
                        result.ToInstall.Add(spec);
                }
            }
        }

        foreach (var (id, ver) in state.Winget)
        {
            if (!config.WingetPackages.Any(p => string.Equals(p.Id, id, StringComparison.OrdinalIgnoreCase)))
                result.ToRemove.Add(new PackageSpec(id, ver, "winget"));
        }

        foreach (var (id, ver) in state.Chocolatey)
        {
            if (!config.ChocoPackages.Any(p => string.Equals(p.Id, id, StringComparison.OrdinalIgnoreCase)))
                result.ToRemove.Add(new PackageSpec(id, ver, "chocolatey"));
        }

        foreach (var (id, ver) in state.Scoop)
        {
            if (!config.ScoopPackages.Any(p => string.Equals(p.Id, id, StringComparison.OrdinalIgnoreCase)))
                result.ToRemove.Add(new PackageSpec(id, ver, "scoop"));
        }

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
}