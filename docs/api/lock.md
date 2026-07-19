# LockFile

## LockFile

Prevents concurrent `ntix apply` execution via an exclusive file lock.

```csharp
namespace NTIX.Core.Lock;
```

### API

```csharp
public class LockFile : IDisposable
{
    public LockFile(string? lockPath = null, bool shouldLock = true);
    public void Dispose();
    public static string GetDefaultLockPath();
}
```

| Member | Description |
|--------|-------------|
| Constructor | Acquires exclusive lock at `lockPath` (or default). Throws `InvalidOperationException` if locked by another process. |
| `Dispose()` | Releases lock and deletes the lock file. Idempotent. |
| `GetDefaultLockPath()` | Returns `%LOCALAPPDATA%/ntix/apply.lock` |

### Usage

```csharp
using var lockFile = new LockFile();
// critical section - only one ntix apply can run
```

### Stale Lock Recovery

If a lock file exists but the owning process is no longer running, the lock is automatically recovered. Lock files contain `PID@UnixTimestamp` for detection.
