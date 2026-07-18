return {
    options = {
        winget = { enable = true, acceptAgreements = true },
        chocolatey = { enable = true, yes = true }
    },
    pkgs = {
        winget = { "Microsoft.VisualStudioCode", "Git.Git" },
        chocolatey = { "git" }
    }
}