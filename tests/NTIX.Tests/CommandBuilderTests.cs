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
    public void BuildScoopInstall_WithVersion_UsesAtSyntax()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", "16.14.2");
        cmd.Should().Be("scoop install nodejs@16.14.2");
    }

    [Fact]
    public void BuildScoopInstall_WithoutVersion_NoAt()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", null);
        cmd.Should().Be("scoop install nodejs");
    }

    [Fact]
    public void BuildScoopBucketAdd_WithNameOnly()
    {
        var cmd = CommandBuilder.BuildScoopBucketAdd("main", null);
        cmd.Should().Be("scoop bucket add main");
    }

    [Fact]
    public void BuildScoopBucketAdd_WithUrl()
    {
        var cmd = CommandBuilder.BuildScoopBucketAdd("ntix", "https://github.com/Cristiandis/scoop-ntix");
        cmd.Should().Be("scoop bucket add ntix https://github.com/Cristiandis/scoop-ntix");
    }

    [Fact]
    public void BuildScoopBucketList_ReturnsCommand()
    {
        var cmd = CommandBuilder.BuildScoopBucketList();
        cmd.Should().Be("scoop bucket list");
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