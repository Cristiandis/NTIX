using System.IO;
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
}