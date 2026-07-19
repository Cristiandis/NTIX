# ConfigLoader

## ConfigLoader

Parses Lua configuration files into `NTIXConfig` objects.

```csharp
namespace NTIX.Core.Config;
```

### Members

| Member | Signature | Returns | Description |
|--------|-----------|---------|-------------|
| `DefaultConfigPath` | `static readonly string` | `string` | `~/ntix/config.lua` |
| `EnsureDefaultConfig` | `string EnsureDefaultConfig(string? configPath)` | `string` | Creates default config if missing; returns resolved path |
| `Load` | `NTIXConfig Load(string configPath)` | `NTIXConfig` | Loads and parses a Lua config file |
| `LoadFromString` | `NTIXConfig LoadFromString(string luaScript, string configPath)` | `NTIXConfig` | Parses a Lua script string |

### Lua Config Contract

The script must return a table with `options` and `pkgs`:

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

### Built-in Lua Function: import()

```lua
import("shared/packages.lua")
import({ path = "env/work.lua", merge = true })
```

Imports and merges additional Lua config files. Paths are resolved relative to the importing file. Package arrays are deduplicated by ID (last wins). Options tables are deep-merged.

### Exceptions

- `FileNotFoundException` - config file not found
- `InvalidOperationException` - Lua syntax or runtime error
