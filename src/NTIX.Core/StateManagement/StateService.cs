using System.Text.Json;
using System.Text.Json.Serialization;
using NTIX.Core.Models;

namespace NTIX.Core.StateManagement;

[JsonSerializable(typeof(State))]
[JsonSerializable(typeof(Dictionary<string, string>))]
[JsonSerializable(typeof(List<PackageEntry>))]
[JsonSerializable(typeof(NTIXConfig))]
[JsonSerializable(typeof(NTIXOptions))]
[JsonSerializable(typeof(WingetOptions))]
[JsonSerializable(typeof(ChocoOptions))]
[JsonSerializable(typeof(ScoopOptions))]
[JsonSerializable(typeof(PackageEntry))]
internal partial class StateJsonContext : JsonSerializerContext { }

public static class StateService
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        PropertyNameCaseInsensitive = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        TypeInfoResolver = StateJsonContext.Default
    };

    public static string GetStatePath()
    {
        var localAppData = Environment.GetEnvironmentVariable("LOCALAPPDATA");
        if (string.IsNullOrEmpty(localAppData))
            throw new InvalidOperationException("LOCALAPPDATA environment variable not set");

        return Path.Combine(localAppData, "ntix", "state.json");
    }

    public static State? LoadState(string? path = null)
    {
        var statePath = string.IsNullOrEmpty(path) ? GetStatePath() : path;
        
        if (!File.Exists(statePath))
            return null;

        try
        {
            var json = File.ReadAllText(statePath);
            return JsonSerializer.Deserialize<State>(json, JsonOptions);
        }
        catch (JsonException)
        {
            return null;
        }
    }

    public static bool SaveState(State state, string? path = null)
    {
        var statePath = string.IsNullOrEmpty(path) ? GetStatePath() : path;
        
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(statePath)!);
            var json = JsonSerializer.Serialize(state, JsonOptions);
            File.WriteAllText(statePath, json);
            return true;
        }
        catch
        {
            return false;
        }
    }
}