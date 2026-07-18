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

| Flag                | Short | Description              |
| ------------------- | ----- | ------------------------ |
| `--dry-run`         | `-d`  | Preview without applying |
| `--no-gc`           |       | Skip orphan removal      |
| `--stop-on-failure` |       | Halt on first failure    |

### What it does

1. Loads config
2. Computes diff
3. Acquires lock file
4. Installs, upgrades, and removes packages
5. Updates state file after each operation

Orphaned packages are removed automatically. Use `--no-gc` to skip.
