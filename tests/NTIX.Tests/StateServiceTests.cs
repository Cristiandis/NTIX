using System.IO;
using FluentAssertions;
using NTIX.Core.Models;
using NTIX.Core.StateManagement;

namespace NTIX.Tests;

public class StateServiceTests
{
    [Fact]
    public void LoadState_NonExistent_ReturnsNull()
    {
        var state = StateService.LoadState("/nonexistent/path.json");
        state.Should().BeNull();
    }

    [Fact]
    public void SaveAndLoadState_RoundTrip()
    {
        var tempPath = Path.Combine(Path.GetTempPath(), $"ntix_test_{Guid.NewGuid()}.json");
        
        try
        {
            var state = new State
            {
                Winget = new Dictionary<string, string> { { "pkg1", "1.0" } },
                Chocolatey = new Dictionary<string, string> { { "pkg2", "2.0" } },
                Scoop = new Dictionary<string, string> { { "pkg3", "3.0" } }
            };

            StateService.SaveState(state, tempPath).Should().BeTrue();
            var loaded = StateService.LoadState(tempPath);
            
            loaded.Should().NotBeNull();
            loaded!.Winget.Should().ContainKey("pkg1").WhoseValue.Should().Be("1.0");
            loaded.Chocolatey.Should().ContainKey("pkg2").WhoseValue.Should().Be("2.0");
            loaded.Scoop.Should().ContainKey("pkg3").WhoseValue.Should().Be("3.0");
        }
        finally
        {
            if (File.Exists(tempPath))
                File.Delete(tempPath);
        }
    }
}