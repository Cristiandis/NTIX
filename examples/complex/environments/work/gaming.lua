import({ "./base.lua", "../../packages/scoop.lua" })

return {
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
