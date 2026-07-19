# Configuration

## Configuration

NTIX uses Lua configuration files. A config file must return a table with two keys: `options` and `pkgs`.

### Minimal config

```lua
local options = {
    winget = { enable = true }
}

local pkgs = {}
pkgs.winget = { "Google.Chrome" }

return { options = options, pkgs = pkgs }
```

### Full simple config

```lua
local options = {
    winget = {
        enable = true,
        acceptAgreements = true,
        interactive = false
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
    { id = "7zip.7zip", version = "23.01" }
}

pkgs.chocolatey = { "ripgrep" }
pkgs.scoop = { "fd", "bat", "neovim" }

return { options = options, pkgs = pkgs }
```

### Structure

| Key       | Purpose                                     |
| --------- | ------------------------------------------- |
| `options` | Configure behavior for each package manager |
| `pkgs`    | List the packages you want installed        |

Each package manager section in `pkgs` is an array of packages. Packages can be strings (latest version) or tables with `id` and `version` fields.

### Learn more

* Options Reference - all available options
* Package Lists - string vs pinned, deduplication
* Multi-Environment - imports and environment switching
