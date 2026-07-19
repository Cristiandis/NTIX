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
        var cmd = CommandBuilder.BuildChocoInstall("test", null, new ChocoOptions());
        cmd.Should().Be("choco install test");
    }

    [Fact]
    public void BuildChocoInstall_WithYes()
    {
        var cmd = CommandBuilder.BuildChocoInstall("test", null, new ChocoOptions(Yes: true));
        cmd.Should().Contain("-y");
    }

    [Fact]
    public void BuildChocoInstall_WithVersion_UsesVersionFlag()
    {
        var cmd = CommandBuilder.BuildChocoInstall("nodejs", "16.14.2", new ChocoOptions(Yes: true));
        cmd.Should().Be("choco install nodejs --version 16.14.2 -y");
    }

    [Fact]
    public void BuildChocoInstall_WithoutVersion_NoVersionFlag()
    {
        var cmd = CommandBuilder.BuildChocoInstall("nodejs", null, new ChocoOptions());
        cmd.Should().Be("choco install nodejs");
    }

    [Fact]
    public void BuildChocoInstall_WithForce()
    {
        var cmd = CommandBuilder.BuildChocoInstall("test", null, new ChocoOptions(Force: true));
        cmd.Should().Contain("--force");
    }

    [Fact]
    public void BuildChocoInstall_WithIgnoreDependencies()
    {
        var cmd = CommandBuilder.BuildChocoInstall("test", null, new ChocoOptions(IgnoreDependencies: true));
        cmd.Should().Contain("--ignore-dependencies");
    }

    [Fact]
    public void BuildChocoInstall_WithAllowDowngrade()
    {
        var cmd = CommandBuilder.BuildChocoInstall("test", null, new ChocoOptions(AllowDowngrade: true));
        cmd.Should().Contain("--allow-downgrade");
    }

    [Fact]
    public void BuildChocoInstall_WithSkipPowerShell()
    {
        var cmd = CommandBuilder.BuildChocoInstall("test", null, new ChocoOptions(SkipPowerShell: true));
        cmd.Should().Contain("--skip-scripts");
    }

    [Fact]
    public void BuildChocoInstall_WithPre()
    {
        var cmd = CommandBuilder.BuildChocoInstall("test", null, new ChocoOptions(Pre: true));
        cmd.Should().Contain("--pre");
    }

    [Fact]
    public void BuildChocoInstall_WithParams()
    {
        var cmd = CommandBuilder.BuildChocoInstall("git", null, new ChocoOptions(Params: "/GitAndUnixToolsOnPath"));
        cmd.Should().Contain("--params=\"'/GitAndUnixToolsOnPath'\"");
    }

    [Fact]
    public void BuildChocoInstall_AllFlags()
    {
        var cmd = CommandBuilder.BuildChocoInstall("git", "2.40.0", new ChocoOptions(
            Yes: true, Force: true, IgnoreDependencies: true,
            AllowDowngrade: true, SkipPowerShell: true, Pre: true, Params: "/GitAndUnixToolsOnPath"));
        cmd.Should().Be("choco install git --version 2.40.0 -y --force --ignore-dependencies --allow-downgrade --skip-scripts --pre --params=\"'/GitAndUnixToolsOnPath'\"");
    }

    [Fact]
    public void BuildScoopInstall_Basic()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", null, new ScoopOptions());
        cmd.Should().Be("scoop install nodejs");
    }

    [Fact]
    public void BuildScoopInstall_WithVersion_UsesAtSyntax()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", "16.14.2", new ScoopOptions());
        cmd.Should().Be("scoop install nodejs@16.14.2");
    }

    [Fact]
    public void BuildScoopInstall_Global()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", null, new ScoopOptions(Global: true));
        cmd.Should().Contain("-g");
    }

    [Fact]
    public void BuildScoopInstall_Independent()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", null, new ScoopOptions(Independent: true));
        cmd.Should().Contain("-i");
    }

    [Fact]
    public void BuildScoopInstall_NoCache()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", null, new ScoopOptions(NoCache: true));
        cmd.Should().Contain("-k");
    }

    [Fact]
    public void BuildScoopInstall_SkipHashCheck()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", null, new ScoopOptions(SkipHashCheck: true));
        cmd.Should().Contain("-s");
    }

    [Fact]
    public void BuildScoopInstall_Arch()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", null, new ScoopOptions(Arch: "64bit"));
        cmd.Should().Contain("--arch 64bit");
    }

    [Fact]
    public void BuildScoopInstall_AllFlags()
    {
        var cmd = CommandBuilder.BuildScoopInstall("nodejs", "20.0.0", new ScoopOptions(
            Global: true, Independent: true, NoCache: true, SkipHashCheck: true, Arch: "64bit"));
        cmd.Should().Be("scoop install nodejs@20.0.0 -g -i -k -s --arch 64bit");
    }

    [Fact]
    public void BuildChocoUpgrade_WithYes()
    {
        var cmd = CommandBuilder.BuildChocoUpgrade("nodejs", new ChocoOptions(Yes: true));
        cmd.Should().Be("choco upgrade nodejs -y");
    }

    [Fact]
    public void BuildChocoUpgrade_Basic()
    {
        var cmd = CommandBuilder.BuildChocoUpgrade("nodejs", new ChocoOptions());
        cmd.Should().Be("choco upgrade nodejs");
    }

    [Fact]
    public void BuildChocoUpgrade_WithForce()
    {
        var cmd = CommandBuilder.BuildChocoUpgrade("nodejs", new ChocoOptions(Force: true));
        cmd.Should().Contain("--force");
    }

    [Fact]
    public void BuildChocoUpgrade_WithPre()
    {
        var cmd = CommandBuilder.BuildChocoUpgrade("nodejs", new ChocoOptions(Pre: true));
        cmd.Should().Contain("--pre");
    }

    [Fact]
    public void BuildScoopUpgrade_Basic()
    {
        var cmd = CommandBuilder.BuildScoopUpgrade("nodejs", new ScoopOptions());
        cmd.Should().Be("scoop update nodejs");
    }

    [Fact]
    public void BuildScoopUpgrade_Global()
    {
        var cmd = CommandBuilder.BuildScoopUpgrade("nodejs", new ScoopOptions(Global: true));
        cmd.Should().Contain("-g");
    }

    [Fact]
    public void BuildChocoUninstall_WithYes()
    {
        var cmd = CommandBuilder.BuildChocoUninstall("nodejs", new ChocoOptions(Yes: true));
        cmd.Should().Be("choco uninstall nodejs -y");
    }

    [Fact]
    public void BuildChocoUninstall_Basic()
    {
        var cmd = CommandBuilder.BuildChocoUninstall("nodejs", new ChocoOptions());
        cmd.Should().Be("choco uninstall nodejs");
    }

    [Fact]
    public void BuildChocoUninstall_WithForce()
    {
        var cmd = CommandBuilder.BuildChocoUninstall("nodejs", new ChocoOptions(Force: true));
        cmd.Should().Contain("--force");
    }

    [Fact]
    public void BuildChocoUninstall_WithIgnoreDependencies()
    {
        var cmd = CommandBuilder.BuildChocoUninstall("nodejs", new ChocoOptions(IgnoreDependencies: true));
        cmd.Should().Contain("--ignore-dependencies");
    }

    [Fact]
    public void BuildScoopUninstall_Basic()
    {
        var cmd = CommandBuilder.BuildScoopUninstall("nodejs", new ScoopOptions());
        cmd.Should().Be("scoop uninstall nodejs");
    }

    [Fact]
    public void BuildScoopUninstall_Global()
    {
        var cmd = CommandBuilder.BuildScoopUninstall("nodejs", new ScoopOptions(Global: true));
        cmd.Should().Contain("-g");
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
    public void SanitizeId_ValidIds_PassThrough()
    {
        CommandBuilder.SanitizeId("test").Should().Be("test");
        CommandBuilder.SanitizeId("my-package").Should().Be("my-package");
        CommandBuilder.SanitizeId("nodejs").Should().Be("nodejs");
        CommandBuilder.SanitizeId("Package.Name").Should().Be("Package.Name");
        CommandBuilder.SanitizeId("some_pkg").Should().Be("some_pkg");
    }

    [Fact]
    public void SanitizeId_EmptyId_Throws()
    {
        Action act = () => CommandBuilder.SanitizeId("");
        act.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void SanitizeId_SpecialChars_Throws()
    {
        Action act = () => CommandBuilder.SanitizeId("pkg; rm -rf /");
        act.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void SanitizeId_PipeChars_Throws()
    {
        Action act = () => CommandBuilder.SanitizeId("pkg|cat /etc/passwd");
        act.Should().Throw<ArgumentException>();
    }
}
