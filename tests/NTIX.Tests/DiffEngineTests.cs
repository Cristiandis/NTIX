using System.Collections.Generic;
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
    public async Task ComputeDiff_EmptyConfigAndState_ReturnsEmpty()
    {
        var config = new NTIXConfig(new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions()));
        var state = new State();
        var diff = await DiffEngine.ComputeDiffAsync(config, state);
        diff.IsEmpty.Should().BeTrue();
        diff.ToInstall.Should().BeEmpty();
        diff.ToUpgrade.Should().BeEmpty();
        diff.ToSkip.Should().BeEmpty();
        diff.ToRemove.Should().BeEmpty();
        diff.HasError.Should().BeFalse();
    }

    [Fact]
    public async Task ComputeDiff_PackageInConfigNotInState_ToInstall()
    {
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("testpkg", null) });
        var state = new State();
        
        var diff = await DiffEngine.ComputeDiffAsync(config, state, validatePackages: false);
        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("testpkg");
    }

    [Fact]
    public async Task ComputeDiff_PackageInStateNotInConfig_ToRemove()
    {
        var config = new NTIXConfig(new NTIXOptions());
        var state = new State { Winget = new Dictionary<string, string> { { "oldpkg", "1.0" } } };
        
        var diff = await DiffEngine.ComputeDiffAsync(config, state);
        diff.ToRemove.Should().HaveCount(1);
        diff.ToRemove[0].Id.Should().Be("oldpkg");
    }

    [Fact]
    public async Task ComputeDiff_PackageInBothStateAndConfig_ToSkip()
    {
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("testpkg", "1.0") });
        var state = new State { Winget = new Dictionary<string, string> { { "testpkg", "1.0" } } };
        
        var diff = await DiffEngine.ComputeDiffAsync(config, state);
        diff.ToSkip.Should().HaveCount(1);
        diff.ToSkip[0].Id.Should().Be("testpkg");
    }

    [Fact]
    public async Task ComputeDiff_WithMockWingetManager_UsesInjectedManager()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "mocked-pkg", "1.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo> { { "mocked-pkg", new UpgradeInfo("1.0", "2.0") } });

        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("mocked-pkg", null) });
        var state = new State();
        
        var diff = await DiffEngine.ComputeDiffAsync(config, state, wingetManager: mockWinget.Object, upgradeMode: true);
        
        diff.ToUpgrade.Should().HaveCount(1);
        diff.ToUpgrade[0].Id.Should().Be("mocked-pkg");
        diff.ToUpgrade[0].Version.Should().Be("2.0");
    }

    [Fact]
    public async Task ComputeDiff_Chocolatey_PinnedVersion_InStateAndNotInState()
    {
        var installed = new InstalledPackages
        {
            Chocolatey = new Dictionary<string, string> { { "choco-in-state", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(), new ChocoOptions(Enable: true), new ScoopOptions()),
            ChocoPackages: new List<PackageEntry> { new("choco-in-state", "1.0"), new("choco-not-in-state", "1.0") });
        var state = new State { Chocolatey = new Dictionary<string, string> { { "choco-in-state", "1.0" } } };
        
        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, validatePackages: false);
        diff.ToSkip.Should().HaveCount(1);
        diff.ToSkip[0].Id.Should().Be("choco-in-state");
        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("choco-not-in-state");
    }

    [Fact]
    public async Task ComputeDiff_Scoop_PinnedVersion_InStateAndNotInState()
    {
        var installed = new InstalledPackages
        {
            Scoop = new Dictionary<string, string> { { "scoop-in-state", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions(Enable: true)),
            ScoopPackages: new List<PackageEntry> { new("scoop-in-state", "1.0"), new("scoop-not-in-state", "1.0") });
        var state = new State { Scoop = new Dictionary<string, string> { { "scoop-in-state", "1.0" } } };
        
        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, validatePackages: false);
        diff.ToSkip.Should().HaveCount(1);
        diff.ToSkip[0].Id.Should().Be("scoop-in-state");
        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("scoop-not-in-state");
    }

    [Fact]
    public async Task ComputeDiff_UnpinnedPkgWithUpgrade_ToUpgrade()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "upgradable-pkg", "1.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>
            {
                { "upgradable-pkg", new UpgradeInfo("1.0", "2.0") }
            });

        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("upgradable-pkg", null) });
        var state = new State { Winget = new Dictionary<string, string> { { "upgradable-pkg", "1.0" } } };

        var diff = await DiffEngine.ComputeDiffAsync(config, state, wingetManager: mockWinget.Object, upgradeMode: true);

        diff.ToUpgrade.Should().HaveCount(1);
        diff.ToUpgrade[0].Id.Should().Be("upgradable-pkg");
        diff.ToUpgrade[0].Version.Should().Be("2.0");
        diff.ToSkip.Should().BeEmpty();
        diff.ToInstall.Should().BeEmpty();
    }

    [Fact]
    public async Task ComputeDiff_UnpinnedPkgInstalled_NoUpgrade_ToSkip()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());

        var installed = new InstalledPackages
        {
            Winget = new Dictionary<string, string> { { "current-pkg", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("current-pkg", null) });
        var state = new State { Winget = new Dictionary<string, string> { { "current-pkg", "1.0" } } };

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object);

        diff.ToSkip.Should().HaveCount(1);
        diff.ToSkip[0].Id.Should().Be("current-pkg");
        diff.ToUpgrade.Should().BeEmpty();
        diff.ToInstall.Should().BeEmpty();
    }

    [Fact]
    public async Task ComputeDiff_NoUpgradeFlag_UpgradablePkg_ToSkip()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "upgradable-pkg", "1.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>
            {
                { "upgradable-pkg", new UpgradeInfo("1.0", "2.0") }
            });

        var installed = new InstalledPackages
        {
            Winget = new Dictionary<string, string> { { "upgradable-pkg", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("upgradable-pkg", null) });
        var state = new State { Winget = new Dictionary<string, string> { { "upgradable-pkg", "1.0" } } };

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object, upgradeMode: false);

        diff.ToSkip.Should().HaveCount(1);
        diff.ToSkip[0].Id.Should().Be("upgradable-pkg");
        diff.ToUpgrade.Should().BeEmpty();
        diff.ToInstall.Should().BeEmpty();
    }

    [Fact]
    public async Task ComputeDiff_UnpinnedPkgNotInstalled_NotInState_ToInstall()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());
        mockWinget.Setup(m => m.PackageExistsAsync(It.IsAny<string>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var installed = new InstalledPackages();
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("new-pkg", null) });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object);

        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("new-pkg");
        diff.ToUpgrade.Should().BeEmpty();
        diff.ToSkip.Should().BeEmpty();
    }

    [Fact]
    public async Task ComputeDiff_UnpinnedPkgInState_NotInstalled_ToInstall()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());
        mockWinget.Setup(m => m.PackageExistsAsync(It.IsAny<string>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var installed = new InstalledPackages();
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            new List<PackageEntry> { new("drifted-pkg", null) });
        var state = new State { Winget = new Dictionary<string, string> { { "drifted-pkg", "1.0" } } };

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object);

        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("drifted-pkg");
    }

    [Fact]
    public async Task ComputeDiff_PinnedVersionMismatch_ToInstall()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "mismatch-pkg", "1.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());
        mockWinget.Setup(m => m.PackageExistsAsync(It.IsAny<string>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var installed = new InstalledPackages
        {
            Winget = new Dictionary<string, string> { { "mismatch-pkg", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("mismatch-pkg", "2.0") });
        var state = new State { Winget = new Dictionary<string, string> { { "mismatch-pkg", "1.0" } } };

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object);

        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("mismatch-pkg");
        diff.ToInstall[0].Version.Should().Be("2.0");
        diff.ToSkip.Should().BeEmpty();
    }

    [Fact]
    public async Task ComputeDiff_PinnedVersionMismatch_CaseInsensitive_ToInstall()
    {
        var installed = new InstalledPackages
        {
            Winget = new Dictionary<string, string> { { "case-pkg", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("case-pkg", "1.0") });
        var state = new State { Winget = new Dictionary<string, string> { { "case-pkg", "1.0" } } };

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed);

        diff.ToSkip.Should().HaveCount(1);
        diff.ToInstall.Should().BeEmpty();
    }

    [Fact]
    public async Task ComputeDiff_DisabledManager_SkipsPackages()
    {
        var installed = new InstalledPackages();
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(), new ChocoOptions(Enable: false), new ScoopOptions()),
            ChocoPackages: new List<PackageEntry> { new("choco-pkg", "1.0") });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed);

        diff.ToInstall.Should().BeEmpty();
        diff.ToUpgrade.Should().BeEmpty();
        diff.ToSkip.Should().BeEmpty();
        diff.ToRemove.Should().BeEmpty();
    }

    [Fact]
    public async Task ComputeDiff_MultipleManagers_AllEnabled()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "winget-current", "1.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());
        mockWinget.Setup(m => m.PackageExistsAsync(It.IsAny<string>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var installed = new InstalledPackages
        {
            Winget = new Dictionary<string, string> { { "winget-current", "1.0" } },
            Chocolatey = new Dictionary<string, string> { { "choco-installed", "1.0" } },
            Scoop = new Dictionary<string, string>()
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(Enable: true), new ScoopOptions(Enable: true)),
            WingetPackages: new List<PackageEntry>
            {
                new("winget-current", null),
                new("winget-new", null)
            },
            ChocoPackages: new List<PackageEntry>
            {
                new("choco-installed", "1.0"),
                new("choco-new", "1.0")
            },
            ScoopPackages: new List<PackageEntry>
            {
                new("scoop-new", null)
            });
        var state = new State
        {
            Winget = new Dictionary<string, string> { { "winget-current", "1.0" } },
            Chocolatey = new Dictionary<string, string> { { "choco-installed", "1.0" }, { "choco-orphan", "1.0" } },
            Scoop = new Dictionary<string, string>()
        };

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object, validatePackages: false);

        diff.ToSkip.Should().Contain(s => s.Id == "winget-current");
        diff.ToInstall.Should().Contain(s => s.Id == "winget-new" && s.Source == "winget");
        diff.ToSkip.Should().Contain(s => s.Id == "choco-installed");
        diff.ToInstall.Should().Contain(s => s.Id == "choco-new" && s.Source == "chocolatey");
        diff.ToInstall.Should().Contain(s => s.Id == "scoop-new" && s.Source == "scoop");
        diff.ToRemove.Should().Contain(s => s.Id == "choco-orphan");
    }

    [Fact]
    public async Task ComputeDiff_NonexistentWingetPackage_BecomesWarning()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string>());
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());
        mockWinget.Setup(m => m.PackageExistsAsync("real-pkg", It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);
        mockWinget.Setup(m => m.PackageExistsAsync("fake-pkg", It.IsAny<CancellationToken>()))
            .ReturnsAsync(false);

        var installed = new InstalledPackages();
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("real-pkg", null), new("fake-pkg", null) });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object);

        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("real-pkg");
        diff.Warnings.Should().Contain(w => w.Contains("fake-pkg"));
    }

    [Fact]
    public async Task ComputeDiff_NonexistentWingetPackage_RemovedFromToInstall()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string>());
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());
        mockWinget.Setup(m => m.PackageExistsAsync(It.IsAny<string>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync(false);

        var installed = new InstalledPackages();
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("only-fake", null) });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object);

        diff.ToInstall.Should().BeEmpty();
        diff.Warnings.Should().Contain(w => w.Contains("only-fake"));
    }

    [Fact]
    public async Task ComputeDiff_InstalledPackage_NotValidated()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "existing-pkg", "1.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());

        var installed = new InstalledPackages
        {
            Winget = new Dictionary<string, string> { { "existing-pkg", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("existing-pkg", null) });
        var state = new State { Winget = new Dictionary<string, string> { { "existing-pkg", "1.0" } } };

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object);

        diff.ToSkip.Should().HaveCount(1);
        mockWinget.Verify(m => m.PackageExistsAsync(It.IsAny<string>(), It.IsAny<CancellationToken>()), Times.Never);
    }

    [Fact]
    public async Task ComputeDiff_WingetValidationThrows_GracefulDegradation()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string>());
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());
        mockWinget.Setup(m => m.PackageExistsAsync(It.IsAny<string>(), It.IsAny<CancellationToken>()))
            .ThrowsAsync(new Exception("network error"));

        var installed = new InstalledPackages();
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("some-pkg", null) });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object);

        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("some-pkg");
        diff.Warnings.Should().Contain(w => w.Contains("Could not verify"));
    }

    [Fact]
    public async Task ComputeDiff_InvalidManagers_ReturnsWarningInResult()
    {
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(), new ChocoOptions(Enable: false), new ScoopOptions(Enable: false)),
            ChocoPackages: new List<PackageEntry> { new("pkg1", "1.0") });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state);

        diff.Error.Should().BeNull();
        diff.Warnings.Should().Contain(w => w.Contains("Chocolatey packages declared but chocolatey not enabled"));
    }

    [Fact]
    public async Task ComputeDiff_ScoopDisabled_WithPackages_GeneratesWarning()
    {
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions(Enable: false)),
            ScoopPackages: new List<PackageEntry> { new("pkg1", "1.0") });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state);

        diff.Warnings.Should().Contain(w => w.Contains("Scoop packages declared but scoop not enabled"));
    }

    [Fact]
    public async Task ComputeDiff_KnownPackage_SkipsValidation()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string>());
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());

        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("known-pkg", null) });
        var state = new State { Winget = new Dictionary<string, string> { { "known-pkg", "1.0" } } };

        var diff = await DiffEngine.ComputeDiffAsync(config, state, new InstalledPackages(), mockWinget.Object);

        diff.ToInstall.Should().HaveCount(1);
        mockWinget.Verify(m => m.PackageExistsAsync(It.IsAny<string>(), It.IsAny<CancellationToken>()), Times.Never);
    }

    [Fact]
    public async Task ComputeDiff_NewPackage_Validates()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string>());
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());
        mockWinget.Setup(m => m.PackageExistsAsync("new-pkg", It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("new-pkg", null) });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state, new InstalledPackages(), mockWinget.Object);

        diff.ToInstall.Should().HaveCount(1);
        mockWinget.Verify(m => m.PackageExistsAsync("new-pkg", It.IsAny<CancellationToken>()), Times.Once);
    }

    [Fact]
    public async Task ComputeDiff_AdoptMode_InstalledNotInState_ToAdopt()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "manual-pkg", "3.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());

        var installed = new InstalledPackages
        {
            Winget = new Dictionary<string, string> { { "manual-pkg", "3.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("manual-pkg", null) });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object, adoptMode: true);

        diff.ToAdopt.Should().HaveCount(1);
        diff.ToAdopt[0].Id.Should().Be("manual-pkg");
        diff.ToSkip.Should().BeEmpty();
        diff.ToInstall.Should().BeEmpty();
    }

    [Fact]
    public async Task ComputeDiff_NoAdoptMode_InstalledNotInState_ToSkip()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "manual-pkg", "3.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());

        var installed = new InstalledPackages
        {
            Winget = new Dictionary<string, string> { { "manual-pkg", "3.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("manual-pkg", null) });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object, adoptMode: false);

        diff.ToSkip.Should().HaveCount(1);
        diff.ToSkip[0].Id.Should().Be("manual-pkg");
        diff.ToAdopt.Should().BeEmpty();
    }

    [Fact]
    public async Task ComputeDiff_AdoptMode_PinnedVersionMatches_ToAdopt()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "pinned-pkg", "1.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());

        var installed = new InstalledPackages
        {
            Winget = new Dictionary<string, string> { { "pinned-pkg", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("pinned-pkg", "1.0") });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object, adoptMode: true);

        diff.ToAdopt.Should().HaveCount(1);
        diff.ToAdopt[0].Id.Should().Be("pinned-pkg");
        diff.ToAdopt[0].Version.Should().Be("1.0");
        diff.ToInstall.Should().BeEmpty();
    }

    [Fact]
    public async Task ComputeDiff_AdoptMode_PinnedVersionMismatch_ToInstall()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.IsInstalled).Returns(true);
        mockWinget.Setup(m => m.GetInstalledPackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, string> { { "pinned-pkg", "1.0" } });
        mockWinget.Setup(m => m.GetUpgradablePackagesAsync(It.IsAny<CancellationToken>()))
            .ReturnsAsync(new Dictionary<string, UpgradeInfo>());

        var installed = new InstalledPackages
        {
            Winget = new Dictionary<string, string> { { "pinned-pkg", "1.0" } }
        };
        var config = new NTIXConfig(
            new NTIXOptions(new WingetOptions(Enable: true), new ChocoOptions(), new ScoopOptions()),
            WingetPackages: new List<PackageEntry> { new("pinned-pkg", "2.0") });
        var state = new State();

        var diff = await DiffEngine.ComputeDiffAsync(config, state, installed, mockWinget.Object, validatePackages: false, adoptMode: true);

        diff.ToInstall.Should().HaveCount(1);
        diff.ToInstall[0].Id.Should().Be("pinned-pkg");
        diff.ToInstall[0].Version.Should().Be("2.0");
        diff.ToAdopt.Should().BeEmpty();
    }
}