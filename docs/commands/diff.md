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
| `--adopt`     |       | Show packages that would be adopted into state   |

By default, `diff` does **not** check for upgrades — it only shows installs, removals, and packages already at the desired version. Pass `-u` to include upgrade detection.

### Output

| Section               | Color    | Meaning                              |
| --------------------- | -------- | ------------------------------------ |
| **To install**        | Green    | Packages not yet installed           |
| **To upgrade**        | Yellow   | Unpinned packages with newer version (only with `-u`) |
| **To adopt**          | Cyan     | Installed packages not yet in state  |
| **Already installed** | Default  | Packages at the correct version      |
| **To remove**         | Dark red | Orphaned packages to clean up        |

Manager names: winget (purple), chocolatey (blue), scoop (pink).

### Example

```
To install:
  winget: Google.Chrome (latest)
  winget: 7zip.7zip (23.01)
  chocolatey: ripgrep (latest)
  scoop: fd (latest)
```

Warnings appear in yellow for packages not found in their manager.
