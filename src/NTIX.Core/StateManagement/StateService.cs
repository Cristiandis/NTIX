using System.Text.Json;
using System.Text.Json.Serialization;
using NTIX.Core.Models;

namespace NTIX.Core.StateManagement;

[JsonSerializable(typeof(State))]
[JsonSerializable(typeof(Dictionary<string, string>))]
[JsonSourceGenerationOptions(WriteIndented = true, PropertyNameCaseInsensitive = true, PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
internal partial class StateJsonContext : JsonSerializerContext { }

public static class StateService
{
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

        var tempPath = statePath + ".tmp";
        if (File.Exists(tempPath))
        {
            try { File.Delete(tempPath); } catch { }
        }

        if (!File.Exists(statePath))
            return null;

        try
        {
            var json = File.ReadAllText(statePath);
            return JsonSerializer.Deserialize(json, StateJsonContext.Default.State);
        }
        catch (JsonException)
        {
            return null;
        }
    }

    public static bool SaveState(State state, string? path = null, int maxRetries = 3)
    {
        var statePath = string.IsNullOrEmpty(path) ? GetStatePath() : path;
        var tempPath = statePath + ".tmp";

        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(statePath)!);
        }
        catch
        {
            return false;
        }

        var json = JsonSerializer.Serialize(state, StateJsonContext.Default.State);

        for (int attempt = 1; attempt <= maxRetries; attempt++)
        {
            try
            {
                File.WriteAllText(tempPath, json);
                File.Move(tempPath, statePath, overwrite: true);
                return true;
            }
            catch (Exception)
            {
                if (attempt >= maxRetries)
                    return false;
                Thread.Sleep(50 * attempt);
            }
            finally
            {
                if (File.Exists(tempPath))
                {
                    try { File.Delete(tempPath); } catch { }
                }
            }
        }

        return false;
    }

    public static async Task<bool> SaveStateAsync(State state, string? path = null, int maxRetries = 3, CancellationToken ct = default)
    {
        var statePath = string.IsNullOrEmpty(path) ? GetStatePath() : path;
        var tempPath = statePath + ".tmp";

        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(statePath)!);
        }
        catch
        {
            return false;
        }

        var json = JsonSerializer.Serialize(state, StateJsonContext.Default.State);

        for (int attempt = 1; attempt <= maxRetries; attempt++)
        {
            ct.ThrowIfCancellationRequested();
            try
            {
                await File.WriteAllTextAsync(tempPath, json, ct);
                File.Move(tempPath, statePath, overwrite: true);
                return true;
            }
            catch (Exception)
            {
                if (attempt >= maxRetries)
                    return false;
                await Task.Delay(50 * attempt, ct);
            }
            finally
            {
                if (File.Exists(tempPath))
                {
                    try { File.Delete(tempPath); } catch { }
                }
            }
        }

        return false;
    }
}