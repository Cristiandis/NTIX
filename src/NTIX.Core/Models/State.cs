namespace NTIX.Core.Models;

public record State(
    int Version = 1,
    Dictionary<string, string>? Winget = null,
    Dictionary<string, string>? Chocolatey = null,
    Dictionary<string, string>? Scoop = null,
    Dictionary<string, string?>? ScoopBuckets = null
)
{
    public Dictionary<string, string> Winget { get; init; } = Winget ?? new();
    public Dictionary<string, string> Chocolatey { get; init; } = Chocolatey ?? new();
    public Dictionary<string, string> Scoop { get; init; } = Scoop ?? new();
    public Dictionary<string, string?> ScoopBuckets { get; init; } = ScoopBuckets ?? new();
}