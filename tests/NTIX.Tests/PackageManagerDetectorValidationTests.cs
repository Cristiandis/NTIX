using System.Collections.Generic;
using FluentAssertions;
using NTIX.Core.Models;
using NTIX.Core.PackageManager;

namespace NTIX.Tests;

public class PackageManagerDetectorValidationTests
{
    [Fact]
    public void ValidateManagers_ChocolateyEnabledNotInstalled_ReturnsError()
    {
        if (PackageManagerDetector.IsChocolateyInstalled())
            return; // Skip if choco is installed

        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(Enable: true),
            new ScoopOptions());
        var config = new NTIXConfig(options);
        
        var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(options, config);
        
        valid.Should().BeFalse();
        error.Should().Contain("Chocolatey is enabled but not installed");
        error.Should().Contain("chocolatey.org/install");
    }

    [Fact]
    public void ValidateManagers_ScoopEnabledNotInstalled_ReturnsError()
    {
        if (PackageManagerDetector.IsScoopInstalled())
            return; // Skip if scoop is installed

        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(),
            new ScoopOptions(Enable: true));
        var config = new NTIXConfig(options);
        
        var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(options, config);
        
        valid.Should().BeFalse();
        error.Should().Contain("Scoop is enabled but not installed");
        error.Should().Contain("scoop.sh");
    }

    [Fact]
    public void ValidateManagers_ChocolateyPackagesNotEnabled_ReturnsWarning()
    {
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(Enable: false),
            new ScoopOptions());
        var config = new NTIXConfig(
            options,
            ChocoPackages: new List<PackageEntry> { new("git", "1.0") });
        
        var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(options, config);
        
        valid.Should().BeTrue();
        error.Should().BeNull();
        warnings.Should().Contain("[warn] Chocolatey packages declared but chocolatey not enabled in options");
    }

    [Fact]
    public void ValidateManagers_ScoopPackagesNotEnabled_ReturnsWarning()
    {
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(),
            new ScoopOptions(Enable: false));
        var config = new NTIXConfig(
            options,
            ScoopPackages: new List<PackageEntry> { new("fd", "1.0") });
        
        var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(options, config);
        
        valid.Should().BeTrue();
        error.Should().BeNull();
        warnings.Should().Contain("[warn] Scoop packages declared but scoop not enabled in options");
    }

    [Fact]
    public void ValidateManagers_AllDisabledNoPackages_ReturnsSuccess()
    {
        var options = new NTIXOptions();
        var config = new NTIXConfig(options);
        
        var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(options, config);
        
        valid.Should().BeTrue();
        error.Should().BeNull();
        warnings.Should().BeEmpty();
    }

    [Fact]
    public void ValidateManagers_NullOptions_HandlesGracefully()
    {
        var options = new NTIXOptions();
        var config = new NTIXConfig(options);
        
        var (valid, error, warnings) = PackageManagerDetector.ValidateManagers(options, config);
        
        valid.Should().BeTrue();
        error.Should().BeNull();
    }
}