using System.Collections.Generic;
using FluentAssertions;
using NTIX.Core.Models;
using NTIX.Core.PackageManager;

namespace NTIX.Tests;

public class CommandBuilderTests
{
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

    [Fact]
    public void BuildScoopInstall_WithVersion_UsesAtSyntax()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", "16.14.2", new List<string>());
        cmd.Should().Be("scoop install nodejs@16.14.2");
    }

    [Fact]
    public void BuildScoopInstall_WithoutVersion_NoAt()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", null, new List<string>());
        cmd.Should().Be("scoop install nodejs");
    }

    [Fact]
    public void BuildChocoInstall_WithVersion_UsesVersionFlag()
    {
        var cmd = CommandBuilder.BuildChocoInstall("nodejs", "16.14.2", true);
        cmd.Should().Be("choco install nodejs --version 16.14.2 -y");
    }

    [Fact]
    public void BuildChocoInstall_WithoutVersion_NoVersionFlag()
    {
        var cmd = CommandBuilder.BuildChocoInstall("nodejs", null, false);
        cmd.Should().Be("choco install nodejs");
    }

    [Fact]
    public void BuildChocoSearch_IncludesLimitOutput()
    {
        var cmd = CommandBuilder.BuildChocoSearch("7zip");
        cmd.Should().Be("choco search 7zip --limit-output");
    }

    [Fact]
    public void BuildScoopInfo_ReturnsScoopInfoCommand()
    {
        var cmd = CommandBuilder.BuildScoopInfo("7zip");
        cmd.Should().Be("scoop info 7zip");
    }

    [Fact]
    public void BuildChocoUpgrade_WithYes()
    {
        var cmd = CommandBuilder.BuildChocoUpgrade("nodejs", true);
        cmd.Should().Be("choco upgrade nodejs -y");
    }

    [Fact]
    public void BuildChocoUpgrade_WithoutYes()
    {
        var cmd = CommandBuilder.BuildChocoUpgrade("nodejs", false);
        cmd.Should().Be("choco upgrade nodejs");
    }

    [Fact]
    public void BuildScoopUpgrade_ReturnsScoopUpdateCommand()
    {
        var cmd = CommandBuilder.BuildScoopUpgrade("nodejs");
        cmd.Should().Be("scoop update nodejs");
    }

    [Fact]
    public void BuildChocoUninstall_WithYes()
    {
        var cmd = CommandBuilder.BuildChocoUninstall("nodejs", true);
        cmd.Should().Be("choco uninstall nodejs -y");
    }

    [Fact]
    public void BuildChocoUninstall_WithoutYes()
    {
        var cmd = CommandBuilder.BuildChocoUninstall("nodejs", false);
        cmd.Should().Be("choco uninstall nodejs");
    }

    [Fact]
    public void BuildScoopUninstall_ReturnsScoopUninstallCommand()
    {
        var cmd = CommandBuilder.BuildScoopUninstall("nodejs");
        cmd.Should().Be("scoop uninstall nodejs");
    }
}