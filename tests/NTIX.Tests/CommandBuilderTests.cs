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
}