# diff

## ntix diff

Show what would change without modifying anything.

### Usage

```bash
ntix diff [config-path]
```

If no path is provided, NTIX uses `~/ntix/config.lua` (creating it on first run).

### Output

| Section               | Color    | Meaning                              |
| --------------------- | -------- | ------------------------------------ |
| **To install**        | Green    | Packages not yet installed           |
| **To upgrade**        | Orange   | Unpinned packages with newer version |
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
