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

public record ScoopBucket(string Name, string? Url = null);

public record ScoopOptions(
    bool Enable = false,
    List<ScoopBucket> Buckets = default!
)
{
    public List<ScoopBucket> Buckets { get; init; } = Buckets ?? new()
    {
        new("main"), new("extras"), new("versions")
    };
};

public record NTIXOptions(
    WingetOptions Winget = default!,
    ChocoOptions Chocolatey = default!,
    ScoopOptions Scoop = default!
);