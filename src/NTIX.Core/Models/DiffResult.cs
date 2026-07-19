namespace NTIX.Core.Models;

public record DiffResult(
    List<PackageSpec> ToInstall = default!,
    List<PackageSpec> ToUpgrade = default!,
    List<PackageSpec> ToSkip = default!,
    List<PackageSpec> ToRemove = default!,
    List<PackageSpec> ToAdopt = default!,
    string? Error = null,
    List<string>? Warnings = null
)
{
    public List<PackageSpec> ToInstall { get; init; } = ToInstall ?? new();
    public List<PackageSpec> ToUpgrade { get; init; } = ToUpgrade ?? new();
    public List<PackageSpec> ToSkip { get; init; } = ToSkip ?? new();
    public List<PackageSpec> ToRemove { get; init; } = ToRemove ?? new();
    public List<PackageSpec> ToAdopt { get; init; } = ToAdopt ?? new();
    public string? Error { get; init; } = Error;
    public List<string> Warnings { get; init; } = Warnings ?? new();

    public bool IsEmpty => ToInstall.Count == 0 && ToUpgrade.Count == 0 && ToRemove.Count == 0 && ToAdopt.Count == 0;
    public bool HasError => !string.IsNullOrEmpty(Error);
}