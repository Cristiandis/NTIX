using System.Collections.Generic;
using System.Text.Json;
using FluentAssertions;
using NTIX.Core.Models;
using NTIX.Core.Config;
using NTIX.Core.StateManagement;
using NTIX.Core.Diff;
using NTIX.Core.PackageManager;
using NTIX.Core.Execution;
using NTIX.Core.Lock;

namespace NTIX.Tests;

public class ModelTests
{
    [Fact]
    public void PackageEntry_DefaultVersion_IsNull()
    {
        var entry = new PackageEntry("test-id");
        entry.Id.Should().Be("test-id");
        entry.Version.Should().BeNull();
    }

    [Fact]
    public void PackageEntry_WithVersion_HasVersion()
    {
        var entry = new PackageEntry("test-id", "1.0.0");
        entry.Id.Should().Be("test-id");
        entry.Version.Should().Be("1.0.0");
    }

    [Fact]
    public void State_Default_EmptyDictionaries()
    {
        var state = new State();
        state.Winget.Should().BeEmpty();
        state.Chocolatey.Should().BeEmpty();
        state.Scoop.Should().BeEmpty();
        state.Version.Should().Be(1);
    }

    [Fact]
    public void State_AddPackage_TracksPackage()
    {
        var state = new State();
        state.Winget["test"] = "1.0";
        state.Winget.Should().ContainKey("test").WhoseValue.Should().Be("1.0");
    }

    [Fact]
    public void DiffResult_IsEmpty_WhenAllEmpty()
    {
        var diff = new DiffResult();
        diff.IsEmpty.Should().BeTrue();
    }

    [Fact]
    public void DiffResult_IsEmpty_False_WhenHasItems()
    {
        var diff = new DiffResult(ToInstall: new List<PackageSpec> { new("test", null, "winget") });
        diff.IsEmpty.Should().BeFalse();
    }

    [Fact]
    public void NTIXOptions_DefaultValues()
    {
        var options = new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions());
        options.Winget.Should().NotBeNull();
        options.Chocolatey.Should().NotBeNull();
        options.Scoop.Should().NotBeNull();
    }

    [Fact]
    public void ScoopOptions_DefaultBuckets()
    {
        var scoop = new ScoopOptions();
        scoop.Buckets.Should().BeEquivalentTo(new[] { "main", "extras", "versions" });
    }
}

public class ConfigLoaderTests
{
    [Fact]
    public void LoadFromString_ValidConfig_ReturnsNTIXConfig()
    {
        var lua = """
            options = {
                winget = { enable = true, acceptAgreements = true, interactive = false },
                chocolatey = { enable = true, yes = true },
                scoop = { enable = true, buckets = { "main", "extras" } }
            }
            pkgs = {
                winget = { "Microsoft.VisualStudioCode" },
                chocolatey = { "git" },
                scoop = { "fd" }
            }
            return { options = options, pkgs = pkgs }
            """;

        var config = ConfigLoader.LoadFromString(lua, "test.lua");
        config.Should().NotBeNull();
        config.Options.Winget.Enable.Should().BeTrue();
        config.Options.Chocolatey.Enable.Should().BeTrue();
        config.Options.Scoop.Enable.Should().BeTrue();
        config.WingetPackages.Should().HaveCount(1);
        config.ChocoPackages.Should().HaveCount(1);
        config.ScoopPackages.Should().HaveCount(1);
    }
}

public class StateServiceTests
{
    [Fact]
    public void LoadState_NonExistent_ReturnsNull()
    {
        var state = StateService.LoadState("/nonexistent/path.json");
        state.Should().BeNull();
    }

    [Fact]
    public void SaveAndLoadState_RoundTrip()
    {
        var tempPath = Path.Combine(Path.GetTempPath(), $"ntix_test_{Guid.NewGuid()}.json");
        
        try
        {
            var state = new State
            {
                Winget = new Dictionary<string, string> { { "pkg1", "1.0" } },
                Chocolatey = new Dictionary<string, string> { { "pkg2", "2.0" } },
                Scoop = new Dictionary<string, string> { { "pkg3", "3.0" } }
            };

            StateService.SaveState(state, tempPath).Should().BeTrue();
            var loaded = StateService.LoadState(tempPath);
            
            loaded.Should().NotBeNull();
            loaded!.Winget.Should().ContainKey("pkg1").WhoseValue.Should().Be("1.0");
            loaded.Chocolatey.Should().ContainKey("pkg2").WhoseValue.Should().Be("2.0");
            loaded.Scoop.Should().ContainKey("pkg3").WhoseValue.Should().Be("3.0");
        }
        finally
        {
            if (File.Exists(tempPath))
                File.Delete(tempPath);
        }
    }
}

public class DiffEngineTests
{
    [Fact]
    public void ComputeDiff_EmptyConfigAndState_ReturnsEmpty()
    {
        var config = new NTIXConfig(new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions()));
        var state = new State();
        var diff = DiffEngine.ComputeDiff(config, state);
        diff.IsEmpty.Should().BeTrue();
    }

    [Fact]
    public void ComputeDiff_PackageInConfigNotInState_ToInstall()
    {
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("testpkg", null) });
        var state = new State();
        
        var diff = DiffEngine.ComputeDiff(config, state);
        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("testpkg");
    }

    [Fact]
    public void ComputeDiff_PackageInStateNotInConfig_ToRemove()
    {
        var config = new NTIXConfig(new NTIXOptions());
        var state = new State { Winget = new Dictionary<string, string> { { "oldpkg", "1.0" } } };
        
        var diff = DiffEngine.ComputeDiff(config, state);
        diff.ToRemove.Should().HaveCount(1);
        diff.ToRemove[0].Id.Should().Be("oldpkg");
    }

    [Fact]
    public void ComputeDiff_PackageInBothStateAndConfig_ToSkip()
    {
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("testpkg", "1.0") });
        var state = new State { Winget = new Dictionary<string, string> { { "testpkg", "1.0" } } };
        
        var diff = DiffEngine.ComputeDiff(config, state);
        diff.ToSkip.Should().HaveCount(1);
        diff.ToSkip[0].Id.Should().Be("testpkg");
    }
}

public class CommandBuilderTests
{
    [Fact]
    public void BuildWingetInstall_Basic()
    {
        var cmd = CommandBuilder.BuildWingetInstall("test", null, false, false);
        cmd.Should().Be("winget install --id test --silent");
    }

    [Fact]
    public void BuildWingetInstall_WithVersion()
    {
        var cmd = CommandBuilder.BuildWingetInstall("test", "1.0", false, false);
        cmd.Should().Be("winget install --id test --version 1.0 --silent");
    }

    [Fact]
    public void BuildWingetInstall_WithAcceptAgreements()
    {
        var cmd = CommandBuilder.BuildWingetInstall("test", null, true, false);
        cmd.Should().Contain("--accept-source-agreements");
        cmd.Should().Contain("--accept-package-agreements");
    }

    [Fact]
    public void BuildChocoInstall_Basic()
    {
        var cmd = CommandBuilder.BuildChocoInstall("test", null, false);
        cmd.Should().Be("choco install test");
    }

    [Fact]
    public void BuildChocoInstall_WithYes()
    {
        var cmd = CommandBuilder.BuildChocoInstall("test", null, true);
        cmd.Should().Contain("-y");
    }

    [Fact]
    public void BuildScoopInstall_WithBuckets()
    {
        var cmd = CommandBuilder.BuildScoopInstall("test", null, new List<string> { "main", "extras" });
        cmd.Should().Contain("--bucket main");
        cmd.Should().Contain("--bucket extras");
    }
}

public class PackageManagerDetectorTests
{
    [Fact]
    public void Detect_ReturnsPMStatus()
    {
        var status = PackageManagerDetector.Detect();
        status.Should().NotBeNull();
    }
}

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

public class ExecutionEngineTests
{
    [Fact]
    public void ApplyDiff_EmptyDiff_ReturnsTrue()
    {
        var diff = new DiffResult();
        var options = new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions());
        var state = new State();
        
        var result = ExecutionEngine.ApplyDiff(diff, options, state);
        result.Should().BeTrue();
    }
}