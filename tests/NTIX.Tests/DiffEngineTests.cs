using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using FluentAssertions;
using Moq;
using NTIX.Core.Models;
using NTIX.Core.Diff;
using NTIX.Core.PackageManager;

namespace NTIX.Tests;

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

    [Fact]
    public void ComputeDiff_WithMockWingetManager_UsesInjectedManager()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "mocked-pkg", "1.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo> { { "mocked-pkg", new UpgradeInfo("1.0", "2.0") } });

        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("mocked-pkg", null) });
        var state = new State();
        
        var diff = DiffEngine.ComputeDiff(config, state, wingetManager: mockWinget.Object);
        
        diff.ToUpgrade.Should().HaveCount(1);
        diff.ToUpgrade[0].Id.Should().Be("mocked-pkg");
        diff.ToUpgrade[0].Version.Should().Be("2.0");
    }

    [Fact]
    public void ComputeDiff_Chocolatey_PinnedVersion_InStateAndNotInState()
    {
        var installed = new InstalledPackages
        {
            Chocolatey = new Dictionary<string, string> { { "choco-in-state", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(), new ChocoOptions(Enable: true), new ScoopOptions()),
            ChocoPackages: new List<PackageEntry> { new("choco-in-state", "1.0"), new("choco-not-in-state", "1.0") });
        var state = new State { Chocolatey = new Dictionary<string, string> { { "choco-in-state", "1.0" } } };
        
        var diff = DiffEngine.ComputeDiff(config, state, installed);
        diff.ToSkip.Should().HaveCount(1);
        diff.ToSkip[0].Id.Should().Be("choco-in-state");
        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("choco-not-in-state");
    }

    [Fact]
    public void ComputeDiff_Scoop_PinnedVersion_InStateAndNotInState()
    {
        var installed = new InstalledPackages
        {
            Scoop = new Dictionary<string, string> { { "scoop-in-state", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions(Enable: true)),
            ScoopPackages: new List<PackageEntry> { new("scoop-in-state", "1.0"), new("scoop-not-in-state", "1.0") });
        var state = new State { Scoop = new Dictionary<string, string> { { "scoop-in-state", "1.0" } } };
        
        var diff = DiffEngine.ComputeDiff(config, state, installed);
        diff.ToSkip.Should().HaveCount(1);
        diff.ToSkip[0].Id.Should().Be("scoop-in-state");
        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("scoop-not-in-state");
    }
}