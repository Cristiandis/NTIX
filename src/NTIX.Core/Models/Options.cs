namespace NTIX.Core.Models;

public record WingetOptions(
    bool Enable = false,
    bool AcceptAgreements = false,
    bool Interactive = false
);

public record ChocoOptions(
    bool Enable = false,
    bool Yes = false,
    bool Force = false,
    bool IgnoreDependencies = false,
    bool AllowDowngrade = false,
    bool SkipPowerShell = false,
    string? Params = null,
    bool Pre = false
);

public record ScoopBucket(string Name, string? Url = null);

public record ScoopOptions(
    bool Enable = false,
    List<ScoopBucket> Buckets = default!,
    bool Global = false,
    bool Independent = false,
    bool NoCache = false,
    bool SkipHashCheck = false,
    string? Arch = null
)
{
    public List<ScoopBucket> Buckets { get; init; } = Buckets ?? new()
    {
        new("main"), new("extras"), new("versions")
    };
};

public record NTIXOptions(
    WingetOptions Winget = null!,
    ChocoOptions Chocolatey = null!,
    ScoopOptions Scoop = null!
)
{
    public WingetOptions Winget { get; init; } = Winget ?? new();
    public ChocoOptions Chocolatey { get; init; } = Chocolatey ?? new();
    public ScoopOptions Scoop { get; init; } = Scoop ?? new();
};