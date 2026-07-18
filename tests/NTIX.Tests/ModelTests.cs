using FluentAssertions;
using NTIX.Core.Models;

namespace NTIX.Tests;

public class ModelTests
{
    [Fact]
    public void PackageEntry_DefaultVersion_IsNull()
    {
        var entry = new PackageEntry("test-id");
        entry.Id.Should().Be("test-id");
        entry.Version.Should().BeNull();
    }

    [Fact]
    public void PackageEntry_WithVersion_HasVersion()
    {
        var entry = new PackageEntry("test-id", "1.0.0");
        entry.Id.Should().Be("test-id");
        entry.Version.Should().Be("1.0.0");
    }

    [Fact]
    public void State_Default_EmptyDictionaries()
    {
        var state = new State();
        state.Winget.Should().BeEmpty();
        state.Chocolatey.Should().BeEmpty();
        state.Scoop.Should().BeEmpty();
        state.Version.Should().Be(1);
    }

    [Fact]
    public void State_AddPackage_TracksPackage()
    {
        var state = new State();
        state.Winget["test"] = "1.0";
        state.Winget.Should().ContainKey("test").WhoseValue.Should().Be("1.0");
    }

    [Fact]
    public void DiffResult_IsEmpty_WhenAllEmpty()
    {
        var diff = new DiffResult();
        diff.IsEmpty.Should().BeTrue();
    }

    [Fact]
    public void DiffResult_IsEmpty_False_WhenHasItems()
    {
        var diff = new DiffResult(ToInstall: new List<PackageSpec> { new("test", null, "winget") });
        diff.IsEmpty.Should().BeFalse();
    }

    [Fact]
    public void NTIXOptions_DefaultValues()
    {
        var options = new NTIXOptions(new WingetOptions(), new ChocoOptions(), new ScoopOptions());
        options.Winget.Should().NotBeNull();
        options.Winget.Enable.Should().BeFalse();
        options.Winget.AcceptAgreements.Should().BeFalse();
        options.Winget.Interactive.Should().BeFalse();
        options.Chocolatey.Should().NotBeNull();
        options.Chocolatey.Enable.Should().BeFalse();
        options.Chocolatey.Yes.Should().BeFalse();
        options.Scoop.Should().NotBeNull();
        options.Scoop.Enable.Should().BeFalse();
    }

    [Fact]
    public void ScoopOptions_DefaultBuckets()
    {
        var scoop = new ScoopOptions();
        scoop.Buckets.Should().HaveCount(3);
        scoop.Buckets[0].Name.Should().Be("main");
        scoop.Buckets[1].Name.Should().Be("extras");
        scoop.Buckets[2].Name.Should().Be("versions");
    }
}