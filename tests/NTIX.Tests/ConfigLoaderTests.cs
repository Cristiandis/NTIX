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
}