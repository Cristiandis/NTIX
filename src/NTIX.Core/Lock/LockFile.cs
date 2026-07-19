using System.IO;

namespace NTIX.Core.Lock;

public class LockFile : IDisposable
{
    private readonly FileStream? _lockStream;
    private readonly string _lockPath;
    private bool _disposed;

    public LockFile(string? lockPath = null, bool shouldLock = true)
    {
        _lockPath = lockPath ?? GetDefaultLockPath();

        if (!shouldLock)
            return;

        var dir = Path.GetDirectoryName(_lockPath)!;
        Directory.CreateDirectory(dir);

        try
        {
            _lockStream = File.Open(_lockPath, FileMode.Create, FileAccess.Write, FileShare.None);
            var lockInfo = $"{Environment.ProcessId}@{DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()}";
            var bytes = System.Text.Encoding.UTF8.GetBytes(lockInfo);
            _lockStream.Write(bytes, 0, bytes.Length);
            _lockStream.Flush();
        }
        catch (IOException ex)
        {
            throw new InvalidOperationException(
                $"Another ntix apply is running (lock file is locked). " +
                $"If this is stale, delete {_lockPath}", ex);
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        if (_lockStream != null)
        {
            try
            {
                _lockStream.Dispose();
                File.Delete(_lockPath);
            }
            catch { }
        }
    }

    public static string GetDefaultLockPath()
    {
        var localAppData = Environment.GetEnvironmentVariable("LOCALAPPDATA");
        if (string.IsNullOrEmpty(localAppData))
            throw new InvalidOperationException("LOCALAPPDATA environment variable not set");

        return Path.Combine(localAppData, "ntix", "apply.lock");
    }
}