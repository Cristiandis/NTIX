return {
    options = {
        winget = {
            enable = true,
            acceptAgreements = true,
            interactive = true,
        },
        chocolatey = { enable = true, yes = true },
        scoop = {
            enable = true,
            buckets = { "extras", "versions", "nerd-fonts", "games" },
        },
    },
    pkgs = {
        winget = {
            "Steam",
            "EpicGamesLauncher",
            "GOGGalaxy",
            "Discord",
        },
        chocolatey = {
            "nvidia-geforce-experience",
            "msi-afterburner",
        },
        scoop = {
            "lutris",
            "heroic-games-launcher",
            "protonup-qt",
        }
    }
}
