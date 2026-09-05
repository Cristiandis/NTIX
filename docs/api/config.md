# Config

## ConfigLoader

Parses Lua configuration files into `NTIXConfig` structs.

```rust
use ntix_rs::config::config_loader;
```

### Items

| Item | Signature | Description |
|------|-----------|-------------|
| `DEFAULT_CONFIG_PATH` | `LazyLock<PathBuf>` | `~/ntix/config.lua` |
| `ensure_default_config` | `fn ensure_default_config(config_path: Option<PathBuf>) -> PathBuf` | Creates the default config if missing and returns the resolved path |
| `load` | `fn load(config_path: PathBuf) -> Result<NTIXConfig, Box<dyn Error>>` | Loads and parses a Lua config file from disk |
| `load_from_string` | `fn load_from_string(lua_script: &str, config_path: PathBuf) -> Result<NTIXConfig, Box<dyn Error>>` | Parses a Lua script string, resolving imports relative to `config_path` |

### Lua Config Contract

The script must return a table with `options` and `pkgs` keys:

```lua
return {
    options = {
        winget = { enable = true, acceptAgreements = true },
        chocolatey = { enable = true, yes = true },
        scoop = { enable = true }
    },
    pkgs = {
        winget = { "Google.Chrome", { id = "7zip.7zip", version = "23.01" } },
        chocolatey = { "ripgrep" },
        scoop = { "fd", "bat" }
    }
}
```

### Config files contract

An optional `configFiles` table declares arbitrary files to manage. Keys are absolute destination paths; values are source paths resolved relative to the config file's directory.

```lua
return {
    options = {},
    pkgs = {},
    configFiles = {
        ["C:/Users/you/AppData/Roaming/kitty/kitty.conf"] = "configs/kitty.conf"
    }
}
```

| Rule | Behavior |
|------|----------|
| Destination | Must be an absolute path; use `/` (Lua treats `\` as an escape sequence). A relative destination is a load error |
| Source | Resolved relative to the config file's directory; must exist at load time (otherwise a load error) |

### Built-in Lua Function: import()

```lua
import("shared/packages.lua")
import({ "./base.lua", "../../packages/scoop.lua" })
```

`import()` accepts either a single path string or an array of path strings. Paths are resolved relative to the importing file. Nested `import()` calls work while an imported script runs. Package arrays are deduplicated by ID (last wins); options tables are deep-merged; `configFiles` maps are merged the same way.

### Errors

- Config file not found
- Lua syntax or runtime error
- `return` value that is not a table
- Missing top-level `options` or `pkgs` table
- Import file not found (referenced from config)
- `configFiles` destination not absolute, or source file missing
