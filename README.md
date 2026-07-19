![NTIX Banner](branding/banner.png)

# NTIX

**nix-like package management for Windows.**

Declare your desired packages in Lua. NTIX figures out what to install, upgrade, and remove across winget, Chocolatey, and Scoop.

[Read the docs](https://cristiandis.gitbook.io/ntix) · [Quick Start](https://cristiandis.gitbook.io/ntix/quick-start) · [Report a Bug](https://github.com/Cristiandis/NTIX/issues)

## Quick Start

1. [Install NTIX](https://cristiandis.gitbook.io/ntix/quick-start) or [build from source](https://cristiandis.gitbook.io/ntix/contributing)
2. Create a `config.lua`:

```lua
local options = {
    winget = { enable = true, acceptAgreements = true },
    chocolatey = { enable = true, yes = true },
    scoop = { enable = true, buckets = { "main", "extras" } }
}

local pkgs = {}
pkgs.winget = { "Google.Chrome", "7zip.7zip" }
pkgs.chocolatey = { "ripgrep" }
pkgs.scoop = { "fd", "bat" }

return { options = options, pkgs = pkgs }
```

3. `ntix diff config.lua` - preview what would change
4. `ntix apply config.lua` - apply changes (requires admin)

## Roadmap

- **Arbitrary config files** - manage dotfiles, shell configs, and system settings declaratively (like NixOS Home Manager)
- **Windows optional features** - enable/disable Hyper-V, OpenSSH, WSL, and other Windows features
- **Nix-shells** - temporary environments with specific packages for your current session

## License

Copyright © 2026 Cristian Izzo. Licensed under GPLv3, see [LICENSE](LICENSE).
Branding assets are licensed under [CC BY 4.0](branding/LICENSE).
