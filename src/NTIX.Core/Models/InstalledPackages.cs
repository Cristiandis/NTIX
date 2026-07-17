namespace NTIX.Core.Models;

public record UpgradeInfo(
    string CurrentVersion,
    string AvailableVersion
);

public record InstalledPackages(
    Dictionary<string, string>? Winget = null,
    Dictionary<string, string>? Chocolatey = null,
    Dictionary<string, string>? Scoop = null
)
{
    public Dictionary<string, string> Winget { get; init; } = Winget ?? new();
    public Dictionary<string, string> Chocolatey { get; init; } = Chocolatey ?? new();
    public Dictionary<string, string> Scoop { get; init; } = Scoop ?? new();
}