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
}