# Multi-Environment

## Multi-Environment

Split your config across files with `import()`.

### Root config

```lua
local env = os.getenv("NTIX_ENV") or "work.dev"

if env == "work.dev" then
    import("environments/work/dev.lua")
elseif env == "work.gaming" then
    import("environments/work/gaming.lua")
end

return { options = options, pkgs = pkgs }
```

### Import syntax

```lua
import("packages/winget.lua")
import({ "./base.lua", "../../packages/scoop.lua" })
```

Paths are relative to the importing file.

