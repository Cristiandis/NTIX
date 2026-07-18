using System.Diagnostics;
using System.Threading.Tasks;
using NTIX.Core.Models;
using NTIX.Core.PackageManager;
using NTIX.Core.StateManagement;

namespace NTIX.Core.Execution;

public static class ExecutionEngine
{
    public static async Task<bool> ApplyDiffAsync(DiffResult diff, NTIXOptions options, State state, string statePath, bool stopOnFailure = true, IWingetManager? wingetManager = null, NTIXConfig? config = null)
    {
        if (!string.IsNullOrEmpty(diff.Error))
        {
            Console.Error.WriteLine($"[error] {diff.Error}");
            foreach (var w in diff.Warnings)
                Console.Error.WriteLine($"[warn] {w}");
            return false;
        }

        if (config != null)
        {
            var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(options, config);
            if (!valid)
            {
                Console.Error.WriteLine($"[error] {error}");
                foreach (var w in warnings)
                    Console.Error.WriteLine($"[warn] {w}");
                return false;
            }
            foreach (var w in warnings)
                Console.Error.WriteLine($"[warn] {w}");
        }

        var allOk = true;
        var manager = wingetManager ?? new WingetManager();

        foreach (var pkg in diff.ToInstall)
        {
            if (!IsEnabled(pkg.Source, options)) continue;

            Console.WriteLine($"Installing {pkg.Source}:{pkg.Id}...");

            bool success = pkg.Source switch
            {
                "winget" => await manager.InstallAsync(pkg.Id, pkg.Version, options.Winget.AcceptAgreements, !options.Winget.Interactive),
                "chocolatey" => await RunCommandAsync(CommandBuilder.BuildChocoInstall(pkg.Id, pkg.Version, options.Chocolatey.Yes)) == 0,
                "scoop" => await RunCommandAsync(CommandBuilder.BuildScoopInstall(pkg.Id, pkg.Version, options.Scoop.Buckets)) == 0,
                _ => throw new InvalidOperationException($"Unknown source: {pkg.Source}")
            };

            if (success)
            {
                UpdateState(state, pkg, true);
                StateService.SaveState(state, statePath);
            }
            else
            {
                Console.Error.WriteLine($"Failed to install {pkg.Source}:{pkg.Id}");
                allOk = false;
                if (stopOnFailure) return false;
            }
        }

        foreach (var pkg in diff.ToUpgrade)
        {
            if (!IsEnabled(pkg.Source, options)) continue;

            Console.WriteLine($"Upgrading {pkg.Source}:{pkg.Id}...");

            bool success = pkg.Source switch
            {
                "winget" => await manager.UpgradeAsync(pkg.Id, options.Winget.AcceptAgreements, !options.Winget.Interactive),
                "chocolatey" => await RunCommandAsync(CommandBuilder.BuildChocoUpgrade(pkg.Id, options.Chocolatey.Yes)) == 0,
                "scoop" => await RunCommandAsync(CommandBuilder.BuildScoopUpgrade(pkg.Id)) == 0,
                _ => throw new InvalidOperationException($"Unknown source: {pkg.Source}")
            };

            if (success)
            {
                UpdateState(state, pkg, true);
                StateService.SaveState(state, statePath);
            }
            else
            {
                Console.Error.WriteLine($"Failed to upgrade {pkg.Source}:{pkg.Id}");
                allOk = false;
                if (stopOnFailure) return false;
            }
        }

        foreach (var pkg in diff.ToRemove)
        {
            if (!IsEnabled(pkg.Source, options)) continue;

            Console.WriteLine($"Removing {pkg.Source}:{pkg.Id}...");

            bool success = pkg.Source switch
            {
                "winget" => await manager.UninstallAsync(pkg.Id, options.Winget.AcceptAgreements, !options.Winget.Interactive),
                "chocolatey" => await RunCommandAsync(CommandBuilder.BuildChocoUninstall(pkg.Id, options.Chocolatey.Yes)) == 0,
                "scoop" => await RunCommandAsync(CommandBuilder.BuildScoopUninstall(pkg.Id)) == 0,
                _ => throw new InvalidOperationException($"Unknown source: {pkg.Source}")
            };

            if (success)
            {
                UpdateState(state, pkg, false);
                StateService.SaveState(state, statePath);
            }
            else
            {
                Console.Error.WriteLine($"Failed to remove {pkg.Source}:{pkg.Id}");
                allOk = false;
                if (stopOnFailure) return false;
            }
        }

        return allOk;
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

    private static async Task<int> RunCommandAsync(string command)
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

        process.OutputDataReceived += (s, e) => { if (e.Data != null) Console.WriteLine(e.Data); };
        process.ErrorDataReceived += (s, e) => { if (e.Data != null) Console.Error.WriteLine(e.Data); };

        process.BeginOutputReadLine();
        process.BeginErrorReadLine();
        await process.WaitForExitAsync();

        return process.ExitCode;
    }
}