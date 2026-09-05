# Options Reference

## Options Reference

The `options` table controls how NTIX interacts with each package manager.

### Winget

| Option               | Type    | Default | Description                        |
| -------------------- | ------- | ------- | ---------------------------------- |
| `enable`             | boolean | `false` | Enable winget                      |
| `acceptAgreements`   | boolean | `false` | Auto-accept source/license agreements |
| `silent`             | boolean | `false` | Run installs silently (`--silent`) |
| `disableInteractivity` | boolean | `false` | Disable interactive prompts (`--disable-interactivity`) |

### Chocolatey

| Option               | Type    | Default | Description                       |
| -------------------- | ------- | ------- | --------------------------------- |
| `enable`             | boolean | `false` | Enable Chocolatey                 |
| `yes`                | boolean | `false` | Skip all prompts (`-y`)           |
| `force`              | boolean | `false` | Force install (`--force`)         |
| `ignoreDependencies` | boolean | `false` | Skip dependencies (`--ignore-dependencies`) |
| `allowDowngrade`     | boolean | `false` | Allow downgrade (`--allow-downgrade`) |
| `skipPowerShell`     | boolean | `false` | Skip install scripts (`--skip-scripts`) |
| `pre`                | boolean | `false` | Include prereleases (`--pre`)     |
| `params`             | string  | `nil`   | Package parameters (`--params`)   |

### Scoop

| Option           | Type    | Default                          | Description                     |
| ---------------- | ------- | -------------------------------- | ------------------------------- |
| `enable`         | boolean | `false`                          | Enable Scoop                    |
| `buckets`        | list    | `{"main", "extras", "versions"}` | Buckets to search               |
| `global`         | boolean | `false`                          | Install globally (`-g`)         |
| `independent`    | boolean | `false`                          | Don't install dependencies (`-i`) |
| `noCache`        | boolean | `false`                          | Skip download cache (`-k`)      |
| `skipHashCheck`  | boolean | `false`                          | Skip hash validation (`-s`)     |
| `arch`           | string  | `nil`                            | Architecture (`--arch 64bit`)   |

### External buckets

Buckets can be specified as strings (built-in) or tables with a name and URL (external):

```lua
scoop = {
    enable = true,
    buckets = {
        "main", "extras",                              -- built-in
        { name = "ntix", url = "https://github.com/Cristiandis/scoop-ntix" }  -- external
    }
}
```

NTIX will automatically add any missing buckets before installing packages.
