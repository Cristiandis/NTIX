using System.IO;
using System.Threading.Tasks;
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

    [Fact]
    public async Task SaveStateAsync_RoundTrip()
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

            (await StateService.SaveStateAsync(state, tempPath)).Should().BeTrue();
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

    [Fact]
    public void LoadState_CorruptJson_ReturnsNull()
    {
        var tempPath = Path.Combine(Path.GetTempPath(), $"ntix_test_{Guid.NewGuid()}.json");

        try
        {
            File.WriteAllText(tempPath, "not valid json {{{");
            var loaded = StateService.LoadState(tempPath);
            loaded.Should().BeNull();
        }
        finally
        {
            if (File.Exists(tempPath))
                File.Delete(tempPath);
        }
    }

    [Fact]
    public void LoadState_CleansOrphanTmp()
    {
        var tempPath = Path.Combine(Path.GetTempPath(), $"ntix_test_{Guid.NewGuid()}.json");
        var tmpPath = tempPath + ".tmp";

        try
        {
            var state = new State
            {
                Winget = new Dictionary<string, string> { { "pkg1", "1.0" } }
            };
            StateService.SaveState(state, tempPath).Should().BeTrue();

            File.WriteAllText(tmpPath, "orphan data");
            File.Exists(tmpPath).Should().BeTrue();

            var loaded = StateService.LoadState(tempPath);

            loaded.Should().NotBeNull();
            loaded!.Winget.Should().ContainKey("pkg1");
            File.Exists(tmpPath).Should().BeFalse();
        }
        finally
        {
            if (File.Exists(tempPath)) File.Delete(tempPath);
            if (File.Exists(tmpPath)) File.Delete(tmpPath);
        }
    }

    [Fact]
    public void SaveState_DirectoryCreationFails_ReturnsFalse()
    {
        var badPath = Path.Combine(Path.GetTempPath(), "ntix_test_impossible<dir", "state.json");
        var state = new State();

        var result = StateService.SaveState(state, badPath);
        result.Should().BeFalse();
    }

    [Fact]
    public void SaveState_ExhaustsRetries_ReturnsFalse()
    {
        var tempDir = Path.Combine(Path.GetTempPath(), $"ntix_test_{Guid.NewGuid()}");
        Directory.CreateDirectory(tempDir);
        var stateFilePath = Path.Combine(tempDir, "state.json");

        try
        {
            Directory.CreateDirectory(stateFilePath);

            var state = new State
            {
                Winget = new Dictionary<string, string> { { "pkg1", "1.0" } }
            };

            var result = StateService.SaveState(state, stateFilePath, maxRetries: 2);
            result.Should().BeFalse();
        }
        finally
        {
            if (Directory.Exists(stateFilePath)) Directory.Delete(stateFilePath);
            if (Directory.Exists(tempDir)) Directory.Delete(tempDir, true);
        }
    }
}