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
        silent = true,
        disableInteractivity = false
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

| Key           | Purpose                                          |
| ------------- | ------------------------------------------------ |
| `options`     | Configure behavior for each package manager      |
| `pkgs`        | List the packages you want installed             |
| `configFiles` | (Optional) Manage arbitrary files on disk        |

Each package manager section in `pkgs` is an array of packages. Packages can be strings (latest version) or tables with `id` and `version` fields.

### Config files

NTIX can also manage arbitrary files (dotfiles, settings, configs) declaratively. Declare them under `configFiles`, keyed by absolute destination path, with the source path as the value:

```lua
return {
    options = { scoop = { enable = true } },
    pkgs = { scoop = { "fd" } },
    configFiles = {
        ["C:/Users/you/AppData/Roaming/kitty/kitty.conf"] = "configs/kitty.conf",
        ["C:/Users/you/.gitconfig"] = "configs/gitconfig",
    }
}
```

* Destinations must be **absolute paths**. Use `/` as the separator - Lua treats `\` as an escape sequence.
* Sources are resolved **relative to the config file's directory** and must exist when the config is parsed.
* `ntix diff -c` previews the actions (`[new]`, `[update]`, `[orphan]`) and `ntix apply -c` applies them. Orphan removal only drops tracking - the file on disk is never touched.
* Managed config files are tracked in state by content hash, so NTIX only rewrites them when the source changes.

### Learn more

* Options Reference - all available options
* Package Lists - string vs pinned, deduplication
* Multi-Environment - imports and environment switching
