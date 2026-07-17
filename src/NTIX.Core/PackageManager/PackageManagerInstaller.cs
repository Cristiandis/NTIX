using System.Diagnostics;
using NTIX.Core.Models;

namespace NTIX.Core.PackageManager;

public enum PMType
{
    Winget,
    Chocolatey,
    Scoop
}

public record InstallResult(bool Success, string? Error = null);

public static class PackageManagerInstaller
{
    public static async Task<bool> EnsureWingetInstalledAsync(bool interactive = false)
    {
        if (PackageManagerDetector.IsWingetInstalled())
            return true;

        var psCommand = """
            if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
                Write-Host "Installing winget via Microsoft Store..."
                try {
                    Add-AppxPackage -Register "C:\Program Files\WindowsApps\Microsoft.DesktopAppInstaller_*_x64__8wekyb3d8bbwe\AppxManifest.xml" -DisableDevelopmentMode 2>$null
                } catch {
                    $proc = Start-Process -FilePath "ms-windows-store:" -ArgumentList "pdp?ProductId=9NBLGGH4NNS1" -PassThru
                    $proc.WaitForExit(30000)
                }
            }
            """;

        return await RunPowerShellCommandAsync(psCommand);
    }

    public static async Task<bool> EnsureChocolateyInstalledAsync(bool interactive = false)
    {
        if (PackageManagerDetector.IsChocolateyInstalled())
            return true;

        if (!PackageManagerDetector.IsRunningAsAdmin())
        {
            Console.Error.WriteLine("[error] Chocolatey installation requires administrator privileges");
            return false;
        }

        var psCommand = """
            Set-ExecutionPolicy Bypass -Scope Process -Force
            [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
            iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
            """;

        return await RunPowerShellCommandAsync(psCommand);
    }

    public static async Task<bool> EnsureScoopInstalledAsync()
    {
        if (PackageManagerDetector.IsScoopInstalled())
            return true;

        var psCommand = """
            Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser -Force
            iex (irm get.scoop.sh)
            """;

        return await RunPowerShellCommandAsync(psCommand);
    }

    public static async Task<bool> EnsurePackageManagerInstalledAsync(PMType pm, bool interactive, bool admin)
    {
        return pm switch
        {
            PMType.Winget => await EnsureWingetInstalledAsync(interactive),
            PMType.Chocolatey => await EnsureChocolateyInstalledAsync(interactive),
            PMType.Scoop => await EnsureScoopInstalledAsync(),
            _ => false
        };
    }

    private static async Task<bool> RunPowerShellCommandAsync(string command)
    {
        try
        {
            var startInfo = new ProcessStartInfo
            {
                FileName = "powershell.exe",
                Arguments = $"-NoProfile -ExecutionPolicy Bypass -Command \"{command.Replace("\"", "\\\"")}\"",
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true,
                StandardOutputEncoding = System.Text.Encoding.UTF8,
                StandardErrorEncoding = System.Text.Encoding.UTF8
            };

            using var process = Process.Start(startInfo);
            if (process == null) return false;

            await process.WaitForExitAsync();
            return process.ExitCode == 0;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[error] Failed to run PowerShell command: {ex.Message}");
            return false;
        }
    }
}