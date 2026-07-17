using System.Diagnostics;
using NTIX.Core.Models;
using NTIX.Core.PackageManager;

namespace NTIX.Core.Execution;

public static class ExecutionEngine
{
    public static bool ApplyDiff(DiffResult diff, NTIXOptions options, State state)
    {
        var allOk = true;

        foreach (var pkg in diff.ToInstall)
        {
            if (!IsEnabled(pkg.Source, options)) continue;

            var cmd = BuildInstallCommand(pkg, options);
            Console.WriteLine($"Installing {pkg.Source}:{pkg.Id}...");
            
            if (RunCommand(cmd) == 0)
            {
                UpdateState(state, pkg, true);
            }
            else
            {
                Console.Error.WriteLine($"Failed to install {pkg.Source}:{pkg.Id}");
                allOk = false;
            }
        }

        foreach (var pkg in diff.ToUpgrade)
        {
            if (!IsEnabled(pkg.Source, options)) continue;

            var cmd = BuildUpgradeCommand(pkg, options);
            Console.WriteLine($"Upgrading {pkg.Source}:{pkg.Id}...");
            
            if (RunCommand(cmd) == 0)
            {
                UpdateState(state, pkg, true);
            }
            else
            {
                Console.Error.WriteLine($"Failed to upgrade {pkg.Source}:{pkg.Id}");
                allOk = false;
            }
        }

        foreach (var pkg in diff.ToRemove)
        {
            if (!IsEnabled(pkg.Source, options)) continue;

            var cmd = BuildUninstallCommand(pkg, options);
            Console.WriteLine($"Removing {pkg.Source}:{pkg.Id}...");
            
            if (RunCommand(cmd) == 0)
            {
                UpdateState(state, pkg, false);
            }
            else
            {
                Console.Error.WriteLine($"Failed to remove {pkg.Source}:{pkg.Id}");
                allOk = false;
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

    private static string BuildInstallCommand(PackageSpec pkg, NTIXOptions options) => pkg.Source switch
    {
        "winget" => CommandBuilder.BuildWingetInstall(pkg.Id, pkg.Version, options.Winget.AcceptAgreements, options.Winget.Interactive),
        "chocolatey" => CommandBuilder.BuildChocoInstall(pkg.Id, pkg.Version, options.Chocolatey.Yes),
        "scoop" => CommandBuilder.BuildScoopInstall(pkg.Id, pkg.Version, options.Scoop.Buckets),
        _ => throw new InvalidOperationException($"Unknown source: {pkg.Source}")
    };

    private static string BuildUpgradeCommand(PackageSpec pkg, NTIXOptions options) => pkg.Source switch
    {
        "winget" => CommandBuilder.BuildWingetUpgrade(pkg.Id, options.Winget.AcceptAgreements, options.Winget.Interactive),
        "chocolatey" => CommandBuilder.BuildChocoUpgrade(pkg.Id, options.Chocolatey.Yes),
        "scoop" => CommandBuilder.BuildScoopUpgrade(pkg.Id),
        _ => throw new InvalidOperationException($"Unknown source: {pkg.Source}")
    };

    private static string BuildUninstallCommand(PackageSpec pkg, NTIXOptions options) => pkg.Source switch
    {
        "winget" => CommandBuilder.BuildWingetUninstall(pkg.Id, options.Winget.AcceptAgreements, options.Winget.Interactive),
        "chocolatey" => CommandBuilder.BuildChocoUninstall(pkg.Id, options.Chocolatey.Yes),
        "scoop" => CommandBuilder.BuildScoopUninstall(pkg.Id),
        _ => throw new InvalidOperationException($"Unknown source: {pkg.Source}")
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

    private static int RunCommand(string command)
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
        process.WaitForExit();

        return process.ExitCode;
    }
}