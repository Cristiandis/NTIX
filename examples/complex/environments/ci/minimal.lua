return {
    options = {
        winget = { enable = true, acceptAgreements = true, interactive = false },
        chocolatey = { enable = true, yes = true },
        scoop = { enable = true, buckets = { "main", "extras" } }
    },
    pkgs = {
        winget = { "Microsoft.VisualStudioCode", "Git.Git" },
    }
}
