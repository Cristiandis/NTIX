local options = {
	winget = {
		enable = true,
		acceptAgreements = true
	},
	chocolatey = {
		enable = true,
		yes = true
	},
	scoop = {
		enable = true,
		buckets = { "main", "extras", "versions" }
	}
}


local pkgs = {}


pkgs.winget = {
	"Google.Chrome",
	{
		id = "7zip.7zip",
		version = "23.01"
	}
}

pkgs.chocolatey = {
	"ripgrep"
}

pkgs.scoop = {
	"ripgrep",
	"fd"
}

return {
	options = options,
	pkgs = pkgs
}
