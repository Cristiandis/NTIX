namespace NTIX.Core.Models;

public record PackageSpec(
    string Id,
    string? Version,
    string Source
);