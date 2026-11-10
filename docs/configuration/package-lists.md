# Package Lists

## Package Lists

Packages are listed under `pkgs`, grouped by manager.

### String form

```lua
pkgs.winget = { "Google.Chrome", "7zip.7zip" }
```

Installs the latest version.

### Pinned versions

```lua
pkgs.winget = {
    "Google.Chrome",
    { id = "7zip.7zip", version = "23.01" }
}
```

Pinned packages are not upgraded and trigger re-install on version drift.

### Deduplication

When using `import()`, packages with the same ID are deduplicated. **Last import wins.**

### Validation

NTIX validates that newly declared packages exist in their managers before diffing; packages already tracked in state are not re-checked. Packages confirmed not to exist are dropped from the diff with a warning, while packages that cannot be verified are kept and flagged.
