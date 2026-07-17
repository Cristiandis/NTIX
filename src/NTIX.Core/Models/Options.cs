namespace NTIX.Core.Models;

public record WingetOptions(
    bool Enable = false,
    bool AcceptAgreements = false,
    bool Interactive = false
);

public record ChocoOptions(
    bool Enable = false,
    bool Yes = false
);

public record ScoopOptions(
    bool Enable = false,
    List<string> Buckets = default!
)
{
    public List<string> Buckets { get; init; } = Buckets ?? new() { "main", "extras", "versions" };
};

public record NTIXOptions(
    WingetOptions Winget = default!,
    ChocoOptions Chocolatey = default!,
    ScoopOptions Scoop = default!
);