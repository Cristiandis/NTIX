using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using FluentAssertions;
using NTIX.Core.Models;
using NTIX.Core.PackageManager;

namespace NTIX.Tests;

[Trait("Category", "Integration")]
public class WingetManagerTests
{
    [Fact]
    public void WingetManager_ImplementsIWingetManager()
    {
        var manager = new WingetManager();
        manager.Should().BeAssignableTo<IWingetManager>();
    }

    [Fact]
    public void IsInstalled_ReturnsBool()
    {
        var manager = new WingetManager();
        Action act = () => { var _ = manager.IsInstalled; };
        act.Should().NotThrow();
    }

    [Fact]
    public async Task IsInstalledAsync_ReturnsBool()
    {
        var manager = new WingetManager();
        var result = await manager.IsInstalledAsync();
        Func<bool> accessing = () => result;
        accessing.Should().NotThrow();
    }

    [Fact]
    public async Task GetInstalledPackagesAsync_ReturnsDictionary()
    {
        var manager = new WingetManager();
        var result = await manager.GetInstalledPackagesAsync();
        result.Should().NotBeNull();
        result.Should().BeAssignableTo<Dictionary<string, string>>();
    }

    [Fact]
    public async Task GetUpgradablePackagesAsync_ReturnsDictionary()
    {
        var manager = new WingetManager();
        var result = await manager.GetUpgradablePackagesAsync();
        result.Should().NotBeNull();
        result.Should().BeAssignableTo<Dictionary<string, UpgradeInfo>>();
    }

    [Fact]
    public async Task InstallAsync_WithInvalidPackage_ReturnsFalse()
    {
        var manager = new WingetManager();
        var result = await manager.InstallAsync("nonexistent-package-xyz-123", silent: true);
        result.Should().BeFalse();
    }

    [Fact]
    public async Task UninstallAsync_WithInvalidPackage_ReturnsFalse()
    {
        var manager = new WingetManager();
        var result = await manager.UninstallAsync("nonexistent-package-xyz-123", silent: true);
        result.Should().BeFalse();
    }

    [Fact]
    public async Task UpgradeAsync_WithInvalidPackage_ReturnsFalse()
    {
        var manager = new WingetManager();
        var result = await manager.UpgradeAsync("nonexistent-package-xyz-123", silent: true);
        result.Should().BeFalse();
    }

    [Fact]
    public async Task GetVersionAsync_ReturnsStringOrNull()
    {
        var manager = new WingetManager();
        var result = await manager.GetVersionAsync();
        if (manager.IsInstalled)
            result.Should().NotBeNull();
        else
            result.Should().BeNull();
    }
}
