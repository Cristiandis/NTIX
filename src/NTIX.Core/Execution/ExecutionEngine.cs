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
        Action<string>? onError = null)
    {
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

        if (options.Scoop.Enable && diff.ToInstall.Any(p => p.Source == "scoop"))
        {
            var ensured = await EnsureScoopBucketsAsync(options.Scoop.Buckets, onOutput, onError);
            if (!ensured)
                allOk = false;
        }

        foreach (var pkg in diff.ToInstall)
        {
            if (!IsEnabled(pkg.Source, options)) continue;

            onOutput?.Invoke($"Installing {pkg.Source}:{pkg.Id}...");

            bool success = pkg.Source switch
            {
                "winget" => await manager.InstallAsync(pkg.Id, pkg.Version, options.Winget.AcceptAgreements, !options.Winget.Interactive),
                "chocolatey" => await RunCommandAsync(CommandBuilder.BuildChocoInstall(pkg.Id, pkg.Version, options.Chocolatey), onOutput, onError) == 0,
                "scoop" => await RunCommandAsync(CommandBuilder.BuildScoopInstall(pkg.Id, pkg.Version, options.Scoop), onOutput, onError) == 0,
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
                "chocolatey" => await RunCommandAsync(CommandBuilder.BuildChocoUpgrade(pkg.Id, options.Chocolatey), onOutput, onError) == 0,
                "scoop" => await RunCommandAsync(CommandBuilder.BuildScoopUpgrade(pkg.Id, options.Scoop), onOutput, onError) == 0,
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
            if (!IsEnabled(pkg.Source, options)) continue;

            onOutput?.Invoke($"Removing {pkg.Source}:{pkg.Id}...");

            bool success = pkg.Source switch
            {
                "winget" => await manager.UninstallAsync(pkg.Id, options.Winget.AcceptAgreements, !options.Winget.Interactive),
                "chocolatey" => await RunCommandAsync(CommandBuilder.BuildChocoUninstall(pkg.Id, options.Chocolatey), onOutput, onError) == 0,
                "scoop" => await RunCommandAsync(CommandBuilder.BuildScoopUninstall(pkg.Id, options.Scoop), onOutput, onError) == 0,
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

        return allOk;
    }

    internal static async Task<bool> EnsureScoopBucketsAsync(
        List<ScoopBucket> configuredBuckets,
        Action<string>? onOutput = null,
        Action<string>? onError = null)
    {
        var output = await RunCommandOutputAsync(CommandBuilder.BuildScoopBucketList());
        var addedBuckets = ParseScoopBucketList(output);

        var allOk = true;
        foreach (var bucket in configuredBuckets)
        {
            if (addedBuckets.Contains(bucket.Name, StringComparer.OrdinalIgnoreCase))
                continue;

            onOutput?.Invoke($"Adding scoop bucket: {bucket.Name}...");
            var exitCode = await RunCommandAsync(CommandBuilder.BuildScoopBucketAdd(bucket.Name, bucket.Url));
            if (exitCode != 0)
            {
                onError?.Invoke($"Failed to add scoop bucket: {bucket.Name}");
                allOk = false;
                continue;
            }

            var reCheck = await RunCommandOutputAsync(CommandBuilder.BuildScoopBucketList());
            var reAdded = ParseScoopBucketList(reCheck);
            if (!reAdded.Contains(bucket.Name, StringComparer.OrdinalIgnoreCase))
            {
                onError?.Invoke($"Scoop bucket was not added: {bucket.Name}");
                allOk = false;
            }
        }

        return allOk;
    }

    internal static HashSet<string> ParseScoopBucketList(string output)
    {
        var buckets = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        if (string.IsNullOrWhiteSpace(output)) return buckets;

        foreach (var line in output.Split('\n'))
        {
            var trimmed = line.Trim();
            if (string.IsNullOrEmpty(trimmed) || trimmed.StartsWith('-'))
                continue;

            var match = Regex.Match(trimmed, @"^(\S+)");
            if (match.Success)
                buckets.Add(match.Groups[1].Value);
        }

        return buckets;
    }

    private static bool IsEnabled(string source, NTIXOptions options) => source switch
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

    internal static async Task<int> RunCommandAsync(
        string command,
        Action<string>? onOutput = null,
        Action<string>? onError = null)
    {
        var psi = new ProcessStartInfo
        {
            FileName = "cmd.exe",
            Arguments = $"/c {command}",
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            StandardOutputEncoding = System.Text.Encoding.UTF8,
            StandardErrorEncoding = System.Text.Encoding.UTF8
        };

        using var process = Process.Start(psi);
        if (process == null) return -1;

        process.OutputDataReceived += (s, e) => { if (e.Data != null) onOutput?.Invoke(e.Data); };
        process.ErrorDataReceived += (s, e) => { if (e.Data != null) onError?.Invoke(e.Data); };

        process.BeginOutputReadLine();
        process.BeginErrorReadLine();
        await process.WaitForExitAsync();

        return process.ExitCode;
    }

    internal static async Task<string> RunCommandOutputAsync(string command)
    {
        var psi = new ProcessStartInfo
        {
            FileName = "cmd.exe",
            Arguments = $"/c {command}",
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            StandardOutputEncoding = System.Text.Encoding.UTF8,
            StandardErrorEncoding = System.Text.Encoding.UTF8
        };

        using var process = Process.Start(psi);
        if (process == null) return string.Empty;

        var stdout = await process.StandardOutput.ReadToEndAsync();
        await process.WaitForExitAsync();

        return stdout;
    }
}
