namespace NTIX.Core.Models;

public record DiffResult(
    List<PackageSpec> ToInstall = default!,
    List<PackageSpec> ToUpgrade = default!,
    List<PackageSpec> ToSkip = default!,
    List<PackageSpec> ToRemove = default!
)
{
    public List<PackageSpec> ToInstall { get; init; } = ToInstall ?? new();
    public List<PackageSpec> ToUpgrade { get; init; } = ToUpgrade ?? new();
    public List<PackageSpec> ToSkip { get; init; } = ToSkip ?? new();
    public List<PackageSpec> ToRemove { get; init; } = ToRemove ?? new();

    public bool IsEmpty => ToInstall.Count == 0 && ToUpgrade.Count == 0 && ToSkip.Count == 0 && ToRemove.Count == 0;
}