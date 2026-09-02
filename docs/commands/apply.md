# apply

## ntix apply

Install, upgrade, and remove packages to match your config.

### Usage

```bash
ntix apply [config-path] [options]
```

If no path is provided, NTIX uses `~/ntix/config.lua` (creating it on first run).

> Requires **administrator privileges**.

### Options

| Flag                | Short | Description                                      |
| ------------------- | ----- | ------------------------------------------------ |
| `--dry-run`         | `-d`  | Preview without applying                        |
| `--upgrade`         | `-u`  | Check for and apply available upgrades           |
| `--adopt`           | `-a`  | Adopt already-installed packages into NTIX state |
| `--no-gc`           |       | Skip orphan removal                             |
| `--stop-on-failure` |       | Halt on first failure                           |

By default, `apply` does **not** check for upgrades - it only installs missing packages, removes orphans, and enforces pinned versions. Pass `-u` to also upgrade unpinned packages.

### What it does

1. Loads config
2. Computes diff
3. Acquires the lock file (fails if another `apply` is running)
4. Adds missing scoop buckets and removes orphaned ones (if scoop is enabled)
5. Installs packages
6. Upgrades packages (only with `-u`)
7. Removes orphaned packages (unless `--no-gc`)
8. Adopts installed packages into state (only with `-a`)
9. Updates the state file after each operation

Orphaned packages are removed automatically. Use `--no-gc` to skip.
