using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using FluentAssertions;
using Moq;
using NTIX.Core.Models;
using NTIX.Core.PackageManager;
using NTIX.Core.Execution;

namespace NTIX.Tests;

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

    [Fact]
    public async Task ApplyDiffAsync_EmptyDiff_ReturnsTrue()
    {
        var diff = new DiffResult();
        var options = new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions());
        var state = new State();
        
        var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state);
        result.Should().BeTrue();
    }

    [Fact]
    public async Task ApplyDiffAsync_WingetInstall_UsesMockManager()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.InstallAsync("test-pkg", "1.0", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("test-pkg", "1.0", "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State();

        var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, mockWinget.Object);
        
        result.Should().BeTrue();
        state.Winget.Should().ContainKey("test-pkg").WhoseValue.Should().Be("1.0");
        mockWinget.Verify(m => m.InstallAsync("test-pkg", "1.0", true, true, It.IsAny<CancellationToken>()), Times.Once);
    }

    [Fact]
    public async Task ApplyDiffAsync_WingetUpgrade_UsesMockManager()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.UpgradeAsync("test-pkg", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var diff = new DiffResult(
            ToUpgrade: new List<PackageSpec> { new("test-pkg", "2.0", "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State { Winget = new Dictionary<string, string> { { "test-pkg", "1.0" } } };

        var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, mockWinget.Object);
        
        result.Should().BeTrue();
        state.Winget["test-pkg"].Should().Be("2.0");
        mockWinget.Verify(m => m.UpgradeAsync("test-pkg", true, true, It.IsAny<CancellationToken>()), Times.Once);
    }

    [Fact]
    public async Task ApplyDiffAsync_WingetUninstall_UsesMockManager()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.UninstallAsync("test-pkg", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var diff = new DiffResult(
            ToRemove: new List<PackageSpec> { new("test-pkg", "1.0", "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State { Winget = new Dictionary<string, string> { { "test-pkg", "1.0" } } };

        var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, mockWinget.Object);
        
        result.Should().BeTrue();
        state.Winget.Should().NotContainKey("test-pkg");
        mockWinget.Verify(m => m.UninstallAsync("test-pkg", true, true, It.IsAny<CancellationToken>()), Times.Once);
    }

    [Fact]
    public async Task ApplyDiffAsync_MixedSources_WorksCorrectly()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.InstallAsync("winget-pkg", "1.0", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);
        mockWinget.Setup(m => m.UpgradeAsync("winget-upgrade", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);
        mockWinget.Setup(m => m.UninstallAsync("winget-remove", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("winget-pkg", "1.0", "winget") },
            ToUpgrade: new List<PackageSpec> { new("winget-upgrade", "2.0", "winget") },
            ToRemove: new List<PackageSpec> { new("winget-remove", "1.0", "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(Enable: true, Yes: true),
            new ScoopOptions(Enable: true, Buckets: new List<string> { "main" }));
        var state = new State 
        { 
            Winget = new Dictionary<string, string> { { "winget-upgrade", "1.0" }, { "winget-remove", "1.0" } },
            Chocolatey = new Dictionary<string, string> { { "choco-pkg", "1.0" } },
            Scoop = new Dictionary<string, string> { { "scoop-pkg", "1.0" } }
        };

        var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, mockWinget.Object);
        
        result.Should().BeTrue();
        state.Winget.Should().ContainKey("winget-pkg").WhoseValue.Should().Be("1.0");
        state.Winget["winget-upgrade"].Should().Be("2.0");
        state.Winget.Should().NotContainKey("winget-remove");
    }

    [Fact]
    public async Task ApplyDiffAsync_WingetInstallFailure_SetsAllOkFalse()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.InstallAsync("fail-pkg", "1.0", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(false);

        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("fail-pkg", "1.0", "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State();

        var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, mockWinget.Object);
        
        result.Should().BeFalse();
        state.Winget.Should().NotContainKey("fail-pkg");
    }
}