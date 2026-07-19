using System.Diagnostics;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using NTIX.Core.Models;
using NTIX.Core.PackageManager;
using NTIX.Core.StateManagement;

namespace NTIX.Core.Execution;

public static class ExecutionEngine
{
    public static async Task<bool> ApplyDiffAsync(
        DiffResult diff,
        NTIXOptions options,
        State state,
        string statePath,
        bool stopOnFailure = true,
        IWingetManager? wingetManager = null,
        NTIXConfig? config = null,
        Action<string>? onOutput = null,
        Action<string>? onError = null,
        ICommandRunner? runner = null)
    {
        var cmd = runner ?? new ProcessCommandRunner();

        if (!string.IsNullOrEmpty(diff.Error))
        {
            onError?.Invoke(diff.Error);
            foreach (var w in diff.Warnings)
                onError?.Invoke(w);
            return false;
        }

        if (config != null)
        {
            var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(options, config);
            if (!valid)
            {
                onError?.Invoke(error ?? "Unknown error");
                foreach (var w in warnings)
                    onError?.Invoke(w);
                return false;
            }
            foreach (var w in warnings)
                onError?.Invoke(w);
        }

        var allOk = true;
        var manager = wingetManager ?? new WingetManager();

        if (options.Scoop.Enable)
        {
            foreach (var bucket in diff.BucketsToAdd)
            {
                onOutput?.Invoke($"Adding scoop bucket: {bucket.Name}...");
                var exitCode = await cmd.RunAsync(CommandBuilder.BuildScoopBucketAdd(bucket.Name, bucket.Url));
                if (exitCode != 0)
                {
                    onError?.Invoke($"Failed to add scoop bucket: {bucket.Name}");
                    allOk = false;
                }
                else
                {
                    state.ScoopBuckets[bucket.Name] = bucket.Url;
                    StateService.SaveState(state, statePath);
                }
            }

            foreach (var bucket in diff.BucketsToRemove)
            {
                onOutput?.Invoke($"Removing scoop bucket: {bucket.Name}...");
                var exitCode = await cmd.RunAsync(CommandBuilder.BuildScoopBucketRemove(bucket.Name));
                if (exitCode != 0)
                {
                    onError?.Invoke($"Failed to remove scoop bucket: {bucket.Name}");
                    allOk = false;
                }
                else
                {
                    state.ScoopBuckets.Remove(bucket.Name);
                    StateService.SaveState(state, statePath);
                }
            }
        }

        foreach (var pkg in diff.ToInstall)
        {
            if (!IsEnabled(pkg.Source, options)) continue;

            onOutput?.Invoke($"Installing {pkg.Source}:{pkg.Id}...");

            bool success = pkg.Source switch
            {
                "winget" => await manager.InstallAsync(pkg.Id, pkg.Version, options.Winget.AcceptAgreements, !options.Winget.Interactive),
                "chocolatey" => await cmd.RunAsync(CommandBuilder.BuildChocoInstall(pkg.Id, pkg.Version, options.Chocolatey), onOutput, onError) == 0,
                "scoop" => await cmd.RunAsync(CommandBuilder.BuildScoopInstall(pkg.Id, pkg.Version, options.Scoop), onOutput, onError) == 0,
                _ => false
            };

            if (success)
            {
                UpdateState(state, pkg, true);
                StateService.SaveState(state, statePath);
            }
            else
            {
                onError?.Invoke($"Failed to install {pkg.Source}:{pkg.Id}");
                allOk = false;
                if (stopOnFailure) return false;
            }
        }

        foreach (var pkg in diff.ToUpgrade)
        {
            if (!IsEnabled(pkg.Source, options)) continue;

            onOutput?.Invoke($"Upgrading {pkg.Source}:{pkg.Id}...");

            bool success = pkg.Source switch
            {
                "winget" => await manager.UpgradeAsync(pkg.Id, options.Winget.AcceptAgreements, !options.Winget.Interactive),
                "chocolatey" => await cmd.RunAsync(CommandBuilder.BuildChocoUpgrade(pkg.Id, options.Chocolatey), onOutput, onError) == 0,
                "scoop" => await cmd.RunAsync(CommandBuilder.BuildScoopUpgrade(pkg.Id, options.Scoop), onOutput, onError) == 0,
                _ => false
            };

            if (success)
            {
                UpdateState(state, pkg, true);
                StateService.SaveState(state, statePath);
            }
            else
            {
                onError?.Invoke($"Failed to upgrade {pkg.Source}:{pkg.Id}");
                allOk = false;
                if (stopOnFailure) return false;
            }
        }

        foreach (var pkg in diff.ToRemove)
        {
            onOutput?.Invoke($"Removing {pkg.Source}:{pkg.Id}...");

            bool success = pkg.Source switch
            {
                "winget" => await cmd.RunAsync(CommandBuilder.BuildWingetUninstall(pkg.Id, options.Winget), onOutput, onError) == 0,
                "chocolatey" => await cmd.RunAsync(CommandBuilder.BuildChocoUninstall(pkg.Id, options.Chocolatey), onOutput, onError) == 0,
                "scoop" => await cmd.RunAsync(CommandBuilder.BuildScoopUninstall(pkg.Id, options.Scoop), onOutput, onError) == 0,
                _ => false
            };

            if (success)
            {
                UpdateState(state, pkg, false);
                StateService.SaveState(state, statePath);
            }
            else
            {
                onError?.Invoke($"Failed to remove {pkg.Source}:{pkg.Id}");
                allOk = false;
                if (stopOnFailure) return false;
            }
        }

        foreach (var pkg in diff.ToAdopt)
        {
            onOutput?.Invoke($"Adopting {pkg.Source}:{pkg.Id}...");
            UpdateState(state, pkg, true);
            StateService.SaveState(state, statePath);
        }

        return allOk;
    }

    internal static bool IsEnabled(string source, NTIXOptions options) => source switch
    {
        "winget" => options.Winget.Enable,
        "chocolatey" => options.Chocolatey.Enable,
        "scoop" => options.Scoop.Enable,
        _ => false
    };

    private static void UpdateState(State state, PackageSpec pkg, bool installed)
    {
        var dict = pkg.Source switch
        {
            "winget" => state.Winget,
            "chocolatey" => state.Chocolatey,
            "scoop" => state.Scoop,
            _ => throw new InvalidOperationException($"Unknown source: {pkg.Source}")
        };

        if (installed)
            dict[pkg.Id] = pkg.Version ?? "latest";
        else
            dict.Remove(pkg.Id);
    }
}
