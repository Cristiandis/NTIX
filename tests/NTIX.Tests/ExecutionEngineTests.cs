using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using FluentAssertions;
using Moq;
using NTIX.Core.Models;
using NTIX.Core.PackageManager;
using NTIX.Core.Execution;
using NTIX.Core.StateManagement;

namespace NTIX.Tests;

public class ExecutionEngineTests
{
    [Fact]
    public async Task ApplyDiff_EmptyDiff_ReturnsTrue()
    {
        var diff = new DiffResult();
        var options = new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);
            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath);
            result.Should().BeTrue();
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
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
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, wingetManager: mockWinget.Object);

            result.Should().BeTrue();
            state.Winget.Should().ContainKey("test-pkg").WhoseValue.Should().Be("1.0");
            mockWinget.Verify(m => m.InstallAsync("test-pkg", "1.0", true, true, It.IsAny<CancellationToken>()), Times.Once);
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
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
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, wingetManager: mockWinget.Object);

            result.Should().BeTrue();
            state.Winget["test-pkg"].Should().Be("2.0");
            mockWinget.Verify(m => m.UpgradeAsync("test-pkg", true, true, It.IsAny<CancellationToken>()), Times.Once);
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
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
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, wingetManager: mockWinget.Object);

            result.Should().BeTrue();
            state.Winget.Should().NotContainKey("test-pkg");
            mockWinget.Verify(m => m.UninstallAsync("test-pkg", true, true, It.IsAny<CancellationToken>()), Times.Once);
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
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
            new ScoopOptions(Enable: true, Buckets: new List<ScoopBucket> { new("main") }));
        var state = new State
        {
            Winget = new Dictionary<string, string> { { "winget-upgrade", "1.0" }, { "winget-remove", "1.0" } },
            Chocolatey = new Dictionary<string, string> { { "choco-pkg", "1.0" } },
            Scoop = new Dictionary<string, string> { { "scoop-pkg", "1.0" } }
        };
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, wingetManager: mockWinget.Object);

            result.Should().BeTrue();
            state.Winget.Should().ContainKey("winget-pkg").WhoseValue.Should().Be("1.0");
            state.Winget["winget-upgrade"].Should().Be("2.0");
            state.Winget.Should().NotContainKey("winget-remove");
            state.Chocolatey.Should().ContainKey("choco-pkg");
            state.Scoop.Should().ContainKey("scoop-pkg");
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
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
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, wingetManager: mockWinget.Object);

            result.Should().BeFalse();
            state.Winget.Should().NotContainKey("fail-pkg");
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_StopOnFalse_ContinuesAfterFailure()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.InstallAsync("fail-pkg", "1.0", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(false);
        mockWinget.Setup(m => m.InstallAsync("ok-pkg", "2.0", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var diff = new DiffResult(
            ToInstall: new List<PackageSpec>
            {
                new("fail-pkg", "1.0", "winget"),
                new("ok-pkg", "2.0", "winget")
            });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, stopOnFailure: false, wingetManager: mockWinget.Object);

            result.Should().BeFalse();
            state.Winget.Should().ContainKey("ok-pkg").WhoseValue.Should().Be("2.0");
            mockWinget.Verify(m => m.InstallAsync("fail-pkg", "1.0", true, true, It.IsAny<CancellationToken>()), Times.Once);
            mockWinget.Verify(m => m.InstallAsync("ok-pkg", "2.0", true, true, It.IsAny<CancellationToken>()), Times.Once);
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_StopOnTrue_ReturnsEarlyOnFailure()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.InstallAsync("fail-pkg", "1.0", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(false);
        mockWinget.Setup(m => m.InstallAsync("ok-pkg", "2.0", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var diff = new DiffResult(
            ToInstall: new List<PackageSpec>
            {
                new("fail-pkg", "1.0", "winget"),
                new("ok-pkg", "2.0", "winget")
            });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, stopOnFailure: true, wingetManager: mockWinget.Object);

            result.Should().BeFalse();
            state.Winget.Should().NotContainKey("ok-pkg");
            mockWinget.Verify(m => m.InstallAsync("ok-pkg", "2.0", true, true, It.IsAny<CancellationToken>()), Times.Never);
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_DisabledSource_SkipsPackage()
    {
        var mockWinget = new Mock<IWingetManager>();

        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("test-pkg", "1.0", "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, wingetManager: mockWinget.Object);

            result.Should().BeTrue();
            state.Winget.Should().BeEmpty();
            mockWinget.Verify(m => m.InstallAsync(It.IsAny<string>(), It.IsAny<string?>(), It.IsAny<bool>(), It.IsAny<bool>(), It.IsAny<CancellationToken>()), Times.Never);
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_NullVersion_RecordsLatest()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.InstallAsync("test-pkg", null, true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("test-pkg", null, "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, wingetManager: mockWinget.Object);

            result.Should().BeTrue();
            state.Winget.Should().ContainKey("test-pkg").WhoseValue.Should().Be("latest");
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_DiffHasError_ReturnsFalse()
    {
        var diff = new DiffResult(Error: "something went wrong", Warnings: new List<string> { "warning1" });
        var options = new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath);

            result.Should().BeFalse();
            state.Winget.Should().BeEmpty();
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_DiffWithWarnings_StillProcesses()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.InstallAsync("test-pkg", "1.0", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("test-pkg", "1.0", "winget") },
            Warnings: new List<string> { "some warning" });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, wingetManager: mockWinget.Object);

            result.Should().BeTrue();
            state.Winget.Should().ContainKey("test-pkg");
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_ConfigValidationFails_ReturnsFalse()
    {
        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("test-pkg", "1.0", "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true),
            new ChocoOptions(Enable: true),
            new ScoopOptions());
        var config = new NTIXConfig(options);
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, config: config);

            result.Should().BeFalse();
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_UnknownSource_IsSkipped()
    {
        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("test-pkg", "1.0", "unknown") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath);

            result.Should().BeTrue();
            state.Winget.Should().BeEmpty();
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_ChocoDisabled_SkipsChocoPackage()
    {
        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("choco-pkg", "1.0", "chocolatey") });
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(Enable: false),
            new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath);

            result.Should().BeTrue();
            state.Chocolatey.Should().BeEmpty();
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_ScoopDisabled_SkipsScoopPackage()
    {
        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("scoop-pkg", "1.0", "scoop") });
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(),
            new ScoopOptions(Enable: false));
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath);

            result.Should().BeTrue();
            state.Scoop.Should().BeEmpty();
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_ConfigWithChocoValidationFails_ReturnsFalse()
    {
        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("test-pkg", "1.0", "winget") });
        var options = new NTIXOptions(
            new WingetOptions(),
            new ChocoOptions(Enable: true),
            new ScoopOptions());
        var config = new NTIXConfig(options);
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, config: config);

            result.Should().BeTrue();
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_ConfigValidationWithWarnings_StillProcesses()
    {
        var mockWinget = new Mock<IWingetManager>();
        mockWinget.Setup(m => m.InstallAsync("winget-pkg", "1.0", true, true, It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var diff = new DiffResult(
            ToInstall: new List<PackageSpec> { new("winget-pkg", "1.0", "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(Enable: false),
            new ScoopOptions(Enable: false));
        var config = new NTIXConfig(options,
            ChocoPackages: new List<PackageEntry> { new("choco-declared", "1.0") },
            ScoopPackages: new List<PackageEntry> { new("scoop-declared", "1.0") });
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(diff, options, state, tempPath, wingetManager: mockWinget.Object, config: config);

            result.Should().BeTrue();
            state.Winget.Should().ContainKey("winget-pkg");
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_OnOutputCalled_ForInstall()
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
        var tempPath = Path.GetTempFileName();
        var outputMessages = new List<string>();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(
                diff, options, state, tempPath,
                wingetManager: mockWinget.Object,
                onOutput: msg => outputMessages.Add(msg));

            result.Should().BeTrue();
            outputMessages.Should().Contain(m => m.Contains("Installing"));
            outputMessages.Should().Contain(m => m.Contains("test-pkg"));
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_OnErrorCalled_ForFailure()
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
        var tempPath = Path.GetTempFileName();
        var errorMessages = new List<string>();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(
                diff, options, state, tempPath,
                wingetManager: mockWinget.Object,
                onError: msg => errorMessages.Add(msg));

            result.Should().BeFalse();
            errorMessages.Should().Contain(m => m.Contains("Failed to install"));
            errorMessages.Should().Contain(m => m.Contains("fail-pkg"));
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_OnOutputCalled_ForUpgrade()
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
        var tempPath = Path.GetTempFileName();
        var outputMessages = new List<string>();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(
                diff, options, state, tempPath,
                wingetManager: mockWinget.Object,
                onOutput: msg => outputMessages.Add(msg));

            result.Should().BeTrue();
            outputMessages.Should().Contain(m => m.Contains("Upgrading"));
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_OnOutputCalled_ForRemove()
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
        var tempPath = Path.GetTempFileName();
        var outputMessages = new List<string>();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(
                diff, options, state, tempPath,
                wingetManager: mockWinget.Object,
                onOutput: msg => outputMessages.Add(msg));

            result.Should().BeTrue();
            outputMessages.Should().Contain(m => m.Contains("Removing"));
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_DiffHasError_OnErrorCalled()
    {
        var diff = new DiffResult(Error: "config error");
        var options = new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        var errorMessages = new List<string>();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(
                diff, options, state, tempPath,
                onError: msg => errorMessages.Add(msg));

            result.Should().BeFalse();
            errorMessages.Should().Contain(m => m.Contains("config error"));
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_ToAdopt_UpdatesStateWithoutInstall()
    {
        var mockWinget = new Mock<IWingetManager>();

        var diff = new DiffResult(
            ToAdopt: new List<PackageSpec> { new("manual-pkg", "3.0", "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        var outputMessages = new List<string>();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(
                diff, options, state, tempPath,
                wingetManager: mockWinget.Object,
                onOutput: msg => outputMessages.Add(msg));

            result.Should().BeTrue();
            state.Winget.Should().ContainKey("manual-pkg").WhoseValue.Should().Be("3.0");
            outputMessages.Should().Contain(m => m.Contains("Adopting"));
            mockWinget.Verify(m => m.InstallAsync(It.IsAny<string>(), It.IsAny<string?>(), It.IsAny<bool>(), It.IsAny<bool>(), It.IsAny<CancellationToken>()), Times.Never);
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }

    [Fact]
    public async Task ApplyDiffAsync_ToAdopt_NullVersion_RecordsLatest()
    {
        var mockWinget = new Mock<IWingetManager>();

        var diff = new DiffResult(
            ToAdopt: new List<PackageSpec> { new("manual-pkg", null, "winget") });
        var options = new NTIXOptions(
            new WingetOptions(Enable: true, AcceptAgreements: true, Interactive: false),
            new ChocoOptions(),
            new ScoopOptions());
        var state = new State();
        var tempPath = Path.GetTempFileName();
        try
        {
            File.Delete(tempPath);

            var result = await ExecutionEngine.ApplyDiffAsync(
                diff, options, state, tempPath,
                wingetManager: mockWinget.Object);

            result.Should().BeTrue();
            state.Winget.Should().ContainKey("manual-pkg").WhoseValue.Should().Be("latest");
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
        }
    }
}
