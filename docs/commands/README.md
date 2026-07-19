# Commands

## Commands

NTIX has three commands: `diff`, `apply`, and `state`.

<table data-view="cards"><thead><tr><th></th><th></th><th></th><th data-hidden data-card-target data-type="content-ref"></th></tr></thead><tbody><tr><td><strong>🔍</strong></td><td><strong>diff</strong></td><td>Preview what would change without modifying anything.</td><td></td></tr><tr><td><strong>📦</strong></td><td><strong>apply</strong></td><td>Install, upgrade, and remove packages to match your config.</td><td></td></tr><tr><td><strong>📋</strong></td><td><strong>state</strong></td><td>Show what NTIX currently manages.</td><td></td></tr></tbody></table>

All commands take a config file path as their primary argument:

```bash
ntix <command> <config-path> [options]
```

### Global behavior

* **Validation**: NTIX validates that packages exist in their respective managers before listing them. Non-existent packages are removed from the diff with a warning.
* **Colors**: Output is color-coded - green for install, yellow for upgrade, cyan for adopt, dark red for remove. Package managers are colored: purple for winget, blue for chocolatey, pink for scoop.
* **Warnings** are yellow, **errors** are red.
