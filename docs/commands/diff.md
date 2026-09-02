# diff

## ntix diff

Show what would change without modifying anything.

### Usage

```bash
ntix diff [config-path] [options]
```

If no path is provided, NTIX uses `~/ntix/config.lua` (creating it on first run).

### Options

| Flag          | Short | Description                                      |
| ------------- | ----- | ------------------------------------------------ |
| `--upgrade`   | `-u`  | Check for and show available upgrades            |
| `--adopt`     | `-a`  | Show packages that would be adopted into state   |

By default, `diff` does **not** check for upgrades - it only shows installs, removals, and packages already at the desired version. Pass `-u` to include upgrade detection.

### Output

NTIX renders a tree with section headers, color-coded by action.

| Section | Symbol | Color | Meaning |
| ------- | ------ | ----- | ------- |
| imports | `imports` | Dim | Imported config files |
| **To install** | `↑` | Green | Packages not yet installed |
| **To upgrade** | `↑` | Yellow | Unpinned packages with newer version (only with `-u`) |
| **To adopt** | `✚` | Cyan | Installed packages not yet in state (only with `-a`) |
| **Already managed** | `✓` | Dim | Packages at the correct version |
| **Buckets to add** | `↑` | Green | Scoop buckets to add (only when scoop is enabled) |
| **Buckets to remove** | `↓` | Red | Scoop buckets to remove |
| **Orphans** | `✗` | Red | Orphaned packages to clean up |

Manager names are colored: winget (purple), chocolatey (blue), scoop (pink).

### Example

```
config.lua
├── ↑ To install (3)
│   ├── winget: Google.Chrome
│   ├── winget: 7zip.7zip (23.01)
│   └── scoop: fd
└── ✓ Already managed (1)
    └── chocolatey: ripgrep
```

Warnings appear in yellow for packages not found in their manager.
