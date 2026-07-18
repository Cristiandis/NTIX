using System;
using System.Runtime.Versioning;
using FluentAssertions;
using Moq;
using NTIX.Core;
using NTIX.Core.Models;
using NTIX.Core.PackageManager;

namespace NTIX.Tests;

public class PackageManagerDetectorTests
{
    [Fact]
    public async Task GetInstalledPackagesAsync_ReturnsAllSources()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "winget-pkg", "1.0" } });

        var result = await PackageManagerDetector.GetInstalledPackagesAsync(() => mockWinget.Object);

        result.Winget.Should().ContainKey("winget-pkg").WhoseValue.Should().Be("1.0");
        result.Chocolatey.Should().NotBeNull();
        result.Scoop.Should().NotBeNull();
    }

    [Fact]
    public async Task GetWingetUpgradablePackagesAsync_ReturnsUpgrades()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>
            {
                { "upgrade-pkg", new UpgradeInfo("1.0", "2.0") }
            });

        var result = await PackageManagerDetector.GetWingetUpgradablePackagesAsync(() => mockWinget.Object);

        result.Should().ContainKey("upgrade-pkg");
        result["upgrade-pkg"].CurrentVersion.Should().Be("1.0");
        result["upgrade-pkg"].AvailableVersion.Should().Be("2.0");
    }

    [Fact]
    public async Task GetAllUpgradablePackagesAsync_MergesAllSources()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>
            {
                { "winget-pkg", new UpgradeInfo("1.0", "2.0") }
            });

        var result = await PackageManagerDetector.GetAllUpgradablePackagesAsync(() => mockWinget.Object);

        result.Should().ContainKey("winget-pkg");
        result["winget-pkg"].CurrentVersion.Should().Be("1.0");
        result["winget-pkg"].AvailableVersion.Should().Be("2.0");
    }

    [Fact]
    public void GetChocoUpgradablePackages_ReturnsDictionary()
    {
        var result = PackageManagerDetector.GetChocoUpgradablePackages();
        result.Should().NotBeNull();
        result.Should().BeAssignableTo<Dictionary<string, UpgradeInfo>>();
    }

    [Fact]
    public void GetScoopUpgradablePackages_ReturnsDictionary()
    {
        var result = PackageManagerDetector.GetScoopUpgradablePackages();
        result.Should().NotBeNull();
        result.Should().BeAssignableTo<Dictionary<string, UpgradeInfo>>();
    }

    [Fact]
    [SupportedOSPlatform("windows")]
    public void IsRunningAsAdmin_ReturnsBool()
    {
        var result = ProcessHelper.IsRunningAsAdmin();
        Action act = () => { var _ = result; };
        act.Should().NotThrow();
    }

    [Fact]
    public async Task ValidateManagersAsync_ScoopDisabled_ReturnsValid()
    {
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(),
            new ScoopOptions(Enable: false));
        var config = new NTIXConfig(options);

        var (valid, error, warnings) = await PackageManagerDetector.ValidateManagersAsync(options, config);

        valid.Should().BeTrue();
        error.Should().BeNull();
    }

    [Fact]
    public async Task ValidateManagersAsync_ChocoDisabled_ReturnsValid()
    {
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(Enable: false),
            new ScoopOptions());
        var config = new NTIXConfig(options);

        var (valid, error, warnings) = await PackageManagerDetector.ValidateManagersAsync(options, config);

        valid.Should().BeTrue();
        error.Should().BeNull();
    }

    [Fact]
    public void ValidateManagers_ScoopDisabled_ReturnsValid()
    {
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(),
            new ScoopOptions(Enable: false));
        var config = new NTIXConfig(options);

        var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(options, config);

        valid.Should().BeTrue();
        error.Should().BeNull();
    }

    [Fact]
    public void ValidateManagers_ChocoDisabled_ReturnsValid()
    {
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(Enable: false),
            new ScoopOptions());
        var config = new NTIXConfig(options);

        var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(options, config);

        valid.Should().BeTrue();
        error.Should().BeNull();
    }

    [Fact]
    public void ValidateManagers_ScoopPackagesDeclared_NotEnabled_GeneratesWarning()
    {
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(),
            new ScoopOptions(Enable: false));
        var config = new NTIXConfig(options,
            ScoopPackages: new List<PackageEntry> { new("pkg1", "1.0") });

        var (valid, _, warnings) = PackageManagerDetector.ValidateManagers(options, config);

        valid.Should().BeTrue();
        warnings.Should().Contain(w => w.Contains("Scoop packages declared but scoop not enabled"));
    }

    [Fact]
    public void ValidateManagers_ChocoPackagesDeclared_NotEnabled_GeneratesWarning()
    {
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(Enable: false),
            new ScoopOptions());
        var config = new NTIXConfig(options,
            ChocoPackages: new List<PackageEntry> { new("pkg1", "1.0") });

        var (valid, _, warnings) = PackageManagerDetector.ValidateManagers(options, config);

        valid.Should().BeTrue();
        warnings.Should().Contain(w => w.Contains("Chocolatey packages declared but chocolatey not enabled"));
    }

    [Fact]
    public void ValidateManagers_NullOptions_DefaultsToNew()
    {
        var config = new NTIXConfig(new NTIXOptions());

        var (valid, _, warnings) = PackageManagerDetector.ValidateManagers(null!, config);

        valid.Should().BeTrue();
    }

    [Fact]
    public void ValidateManagers_SyncWithOptions_ReturnsValid()
    {
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(Enable: false),
            new ScoopOptions(Enable: false));
        var config = new NTIXConfig(options);

        var (valid, _, warnings) = PackageManagerDetector.ValidateManagers(options, config);

        valid.Should().BeTrue();
        warnings.Should().BeEmpty();
    }

    [Fact]
    public async Task GetInstalledPackagesAsync_WingetManagerThrows_ReturnsEmptyWinget()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ThrowsAsync(new Exception("winget error"));

        var result = await PackageManagerDetector.GetInstalledPackagesAsync(() => mockWinget.Object);

        result.Winget.Should().BeEmpty();
    }
}