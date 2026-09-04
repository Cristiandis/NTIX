local options = {}

local pkgs = {}

local configFiles = {}

-- dest (absolute) -> source file, resolved relative to this config file.
configFiles["C:/Users/Example/AppData/Roaming/kitty/kitty.conf"] = "kitty.conf"

return {
	options = options,
	pkgs = pkgs,
	configFiles = configFiles
}
