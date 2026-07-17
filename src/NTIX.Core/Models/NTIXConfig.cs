namespace NTIX.Core.Models;

public record NTIXConfig(
    NTIXOptions Options,
    List<PackageEntry> WingetPackages = default!,
    List<PackageEntry> ChocoPackages = default!,
    List<PackageEntry> ScoopPackages = default!
)
{
    public List<PackageEntry> WingetPackages { get; init; } = WingetPackages ?? new();
    public List<PackageEntry> ChocoPackages { get; init; } = ChocoPackages ?? new();
    public List<PackageEntry> ScoopPackages { get; init; } = ScoopPackages ?? new();
}