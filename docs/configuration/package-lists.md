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

NTIX validates packages exist in their managers before diffing. Invalid packages are flagged with a warning.
