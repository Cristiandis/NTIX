using System.IO;
using System.Text.RegularExpressions;
using FluentAssertions;
using NTIX.Core.Lock;

namespace NTIX.Tests;

public class LockFileTests
{
    [Fact]
    public void LockFile_CreateAndDispose()
    {
        var tempDir = Path.Combine(Path.GetTempPath(), $"ntix_lock_test_{Guid.NewGuid()}");
        Directory.CreateDirectory(tempDir);
        var lockPath = Path.Combine(tempDir, "test.lock");
        
        try
        {
            using (var lockFile = new LockFile(lockPath))
            {
                File.Exists(lockPath).Should().BeTrue();
            }
            
            File.Exists(lockPath).Should().BeFalse();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void LockFile_SecondLockThrows()
    {
        var tempDir = Path.Combine(Path.GetTempPath(), $"ntix_lock_test_{Guid.NewGuid()}");
        Directory.CreateDirectory(tempDir);
        var lockPath = Path.Combine(tempDir, "test.lock");
        
        try
        {
            using var lock1 = new LockFile(lockPath);
            var act = () => new LockFile(lockPath);
            act.Should().Throw<InvalidOperationException>()
                .WithMessage("*Another ntix apply is running*");
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void LockFile_StaleLock_Overwritten()
    {
        var tempDir = Path.Combine(Path.GetTempPath(), $"ntix_lock_test_{Guid.NewGuid()}");
        Directory.CreateDirectory(tempDir);
        var lockPath = Path.Combine(tempDir, "test.lock");

        try
        {
            File.WriteAllText(lockPath, "1234@9999999999");

            using var lockFile = new LockFile(lockPath);
            File.Exists(lockPath).Should().BeTrue();
        }
        finally
        {
            if (File.Exists(lockPath)) File.Delete(lockPath);
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void LockFile_EmptyExistingFile_CreatesLock()
    {
        var tempDir = Path.Combine(Path.GetTempPath(), $"ntix_lock_test_{Guid.NewGuid()}");
        Directory.CreateDirectory(tempDir);
        var lockPath = Path.Combine(tempDir, "test.lock");

        try
        {
            File.WriteAllText(lockPath, "");

            using (var lockFile = new LockFile(lockPath))
            {
                File.Exists(lockPath).Should().BeTrue();
            }

            File.Exists(lockPath).Should().BeFalse();
        }
        finally
        {
            if (File.Exists(lockPath)) File.Delete(lockPath);
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void LockFile_ShouldLockFalse_NoOp()
    {
        var tempDir = Path.Combine(Path.GetTempPath(), $"ntix_lock_test_{Guid.NewGuid()}");
        Directory.CreateDirectory(tempDir);
        var lockPath = Path.Combine(tempDir, "test.lock");

        try
        {
            using var lockFile = new LockFile(lockPath, shouldLock: false);
            File.Exists(lockPath).Should().BeFalse();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void LockFile_DisposeIdempotent()
    {
        var tempDir = Path.Combine(Path.GetTempPath(), $"ntix_lock_test_{Guid.NewGuid()}");
        Directory.CreateDirectory(tempDir);
        var lockPath = Path.Combine(tempDir, "test.lock");

        try
        {
            var lockFile = new LockFile(lockPath);
            File.Exists(lockPath).Should().BeTrue();

            var act = () =>
            {
                lockFile.Dispose();
                lockFile.Dispose();
            };
            act.Should().NotThrow();
            File.Exists(lockPath).Should().BeFalse();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void LockFile_StaleLock_CreatedByProcess_CanBeOverwritten()
    {
        var tempDir = Path.Combine(Path.GetTempPath(), $"ntix_lock_test_{Guid.NewGuid()}");
        Directory.CreateDirectory(tempDir);
        var lockPath = Path.Combine(tempDir, "test.lock");

        try
        {
            var lock1 = new LockFile(lockPath);
            lock1.Dispose();

            using var lock2 = new LockFile(lockPath);
            File.Exists(lockPath).Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void GetDefaultLockPath_ReturnsPathUnderLocalAppData()
    {
        var path = LockFile.GetDefaultLockPath();
        var localAppData = Environment.GetEnvironmentVariable("LOCALAPPDATA");
        path.Should().StartWith(localAppData);
        path.Should().EndWith(Path.Combine("ntix", "apply.lock"));
    }

    [Fact]
    public void GetDefaultLockPath_ContainsNtixFolder()
    {
        var path = LockFile.GetDefaultLockPath();
        path.Should().Contain("ntix");
    }
}