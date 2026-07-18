import({ "../../packages/scoop.lua", "../../packages/chocolatey.lua", "../../packages/winget.lua" })

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
            buckets = { "main", "extras", "versions", "nerd-fonts", "games" },
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
