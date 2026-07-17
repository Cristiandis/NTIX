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

        if (File.Exists(_lockPath))
        {
            // Check if file is locked by another process
            try
            {
                // Try to open with FileShare.None to see if we can get exclusive access
                using var checkStream = File.Open(_lockPath, FileMode.Open, FileAccess.ReadWrite, FileShare.None);
                // If we got here, file is not locked - read content to check if it's a stale lock
                var content = File.ReadAllText(_lockPath).Trim();
                if (!string.IsNullOrEmpty(content))
                {
                    throw new InvalidOperationException(
                        $"Another ntix apply is running (lock: {content}). " +
                        $"If this is stale, delete {_lockPath}");
                }
            }
            catch (IOException)
            {
                // File is locked by another process
                throw new InvalidOperationException(
                    $"Another ntix apply is running (lock file is locked). " +
                    $"If this is stale, delete {_lockPath}");
            }
        }

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
            throw new InvalidOperationException($"Failed to create lock file: {ex.Message}", ex);
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