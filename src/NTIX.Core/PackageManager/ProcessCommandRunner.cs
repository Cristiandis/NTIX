using System.Diagnostics;
using System.Threading.Tasks;

namespace NTIX.Core.PackageManager;

public sealed class ProcessCommandRunner : ICommandRunner
{
    public async Task<int> RunAsync(string command, Action<string>? onOutput = null, Action<string>? onError = null)
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

    public async Task<string> RunOutputAsync(string command, bool combineStderr = false)
    {
        var psi = new ProcessStartInfo
        {
            FileName = "cmd.exe",
            Arguments = combineStderr ? $"/c {command} 2>&1" : $"/c {command}",
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

        return stdout.Trim();
    }
}
