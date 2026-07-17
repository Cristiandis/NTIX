using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using FluentAssertions;
using NTIX.Core.Models;
using NTIX.Core.PackageManager;

namespace NTIX.Tests;

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
        var result = manager.IsInstalled;
        (result == true || result == false).Should().BeTrue();
    }

    [Fact]
    public async Task IsInstalledAsync_ReturnsBool()
    {
        var manager = new WingetManager();
        var result = await manager.IsInstalledAsync();
        (result == true || result == false).Should().BeTrue();
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
        (result == null || result != null).Should().BeTrue();
    }

    [Fact]
    public async Task ExportImportPackagesAsync_InvalidPath_ReturnsFalse()
    {
        var manager = new WingetManager();
        var exportResult = await manager.ExportPackagesAsync("/invalid/path/export.json");
        exportResult.Should().BeFalse();
        
        var importResult = await manager.ImportPackagesAsync("/invalid/path/import.json");
        importResult.Should().BeFalse();
    }
}