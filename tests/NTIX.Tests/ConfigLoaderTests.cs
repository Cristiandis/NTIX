using System.Collections.Generic;
using System.Text.Json;
using FluentAssertions;
using NTIX.Core.Config;
using NTIX.Core.Models;

namespace NTIX.Tests;

public class ConfigLoaderTests
{
    [Fact]
    public void LoadFromString_ValidConfig_ReturnsNTIXConfig()
    {
        var lua = """
            options = {
                winget = { enable = true, acceptAgreements = true, interactive = false },
                chocolatey = { enable = true, yes = true },
                scoop = { enable = true, buckets = { "main", "extras" } }
            }
            pkgs = {
                winget = { "Microsoft.VisualStudioCode" },
                chocolatey = { "git" },
                scoop = { "fd" }
            }
            return { options = options, pkgs = pkgs }
            """;

        var config = ConfigLoader.LoadFromString(lua, "test.lua");
        config.Should().NotBeNull();
        config.Options.Winget.Enable.Should().BeTrue();
        config.Options.Chocolatey.Enable.Should().BeTrue();
        config.Options.Scoop.Enable.Should().BeTrue();
        config.WingetPackages.Should().HaveCount(1);
        config.ChocoPackages.Should().HaveCount(1);
        config.ScoopPackages.Should().HaveCount(1);
    }

    [Fact]
    public void LoadFromString_MissingInteractiveField_DefaultsToFalse()
    {
        var lua = """
            options = {
                winget = { enable = true, acceptAgreements = true },
                chocolatey = { enable = true, yes = true },
                scoop = { enable = true, buckets = { "main", "extras" } }
            }
            pkgs = {
                winget = { "Microsoft.VisualStudioCode" },
                chocolatey = { "git" },
                scoop = { "fd" }
            }
            return { options = options, pkgs = pkgs }
            """;

        var config = ConfigLoader.LoadFromString(lua, "test.lua");
        config.Should().NotBeNull();
        config.Options.Winget.Enable.Should().BeTrue();
        config.Options.Winget.Interactive.Should().BeFalse();
    }

    [Fact]
    public void Load_MissingConfigFile_ThrowsFileNotFoundException()
    {
        var act = () => ConfigLoader.Load("nonexistent.lua");
        act.Should().Throw<FileNotFoundException>()
            .WithMessage("Config file not found: nonexistent.lua");
    }

    [Fact]
    public void LoadFromString_MissingOptionsTable_ThrowsInvalidOperationException()
    {
        var lua = """
            pkgs = {
                winget = { "test" }
            }
            return { pkgs = pkgs }
            """;

        var act = () => ConfigLoader.LoadFromString(lua, "test.lua");
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("Config error: missing top-level 'options' table");
    }

    [Fact]
    public void LoadFromString_MissingPkgsTable_ThrowsInvalidOperationException()
    {
        var lua = """
            options = {
                winget = { enable = true }
            }
            return { options = options }
            """;

        var act = () => ConfigLoader.LoadFromString(lua, "test.lua");
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("Config error: missing top-level 'pkgs' table");
    }

    [Fact]
    public void LoadFromString_MissingWingetInOptions_DefaultsToEmpty()
    {
        var lua = """
            options = {
                chocolatey = { enable = true }
            }
            pkgs = {
                winget = { "test" }
            }
            return { options = options, pkgs = pkgs }
            """;

        var config = ConfigLoader.LoadFromString(lua, "test.lua");
        config.Should().NotBeNull();
        config.Options.Winget.Enable.Should().BeFalse();
    }
}