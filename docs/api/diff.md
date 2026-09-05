# Diff

## DiffEngine

Computes the difference between the desired state (config) and the current state.

```rust
use ntix_rs::diff::diff_engine;
```

### compute_diff

```rust
#[allow(clippy::too_many_arguments)]
pub async fn compute_diff(
    config: &NTIXConfig,
    state: &State,
    winget_installed: Option<bool>,
    choco_installed: Option<bool>,
    scoop_installed: Option<bool>,
    runner: Option<&dyn CommandRunner>,
    adopt_mode: bool,
    upgrade_mode: bool,
    validate_packages: bool,
    installed: Option<&InstalledPackages>,
    progress: &ProgressBar,
) -> Result<DiffResult, Box<dyn Error>>
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `config` | `&NTIXConfig` | required | Desired package state |
| `state` | `&State` | required | Current tracked state |
| `winget_installed` | `Option<bool>` | `None` | Override winget availability; auto-detected if `None` |
| `choco_installed` | `Option<bool>` | `None` | Override Chocolatey availability; auto-detected if `None` |
| `scoop_installed` | `Option<bool>` | `None` | Override Scoop availability; auto-detected if `None` |
| `runner` | `Option<&dyn CommandRunner>` | `None` | Injected command runner; uses `ProcessCommandRunner` if `None` |
| `adopt_mode` | `bool` | `false` | Adopt externally installed packages into state |
| `upgrade_mode` | `bool` | `false` | Check for available upgrades |
| `validate_packages` | `bool` | `true` | Validate package existence in remote repos |
| `installed` | `Option<&InstalledPackages>` | `None` | Pre-fetched installed packages; auto-discovers if `None` |
| `progress` | `&ProgressBar` | required | Status message reporter (an indicatif spinner) |

**Returns:** `Result<DiffResult, Box<dyn Error>>`

### Execution Flow

1. Runs a single manager capability-detection pass (`package_manager_detector::validate_managers_async`); the resulting `ValidationResult` is stored on the diff and its warnings copied over
2. Discovers installed packages (if `installed` is `None`)
3. Fetches upgradable packages for each enabled source that has unpinned entries (only in `upgrade_mode`)
4. Classifies each declared package into `to_install`, `to_upgrade`, `to_skip`, or `to_adopt`
5. Collects `to_untracked` (installed packages NTIX does not track)
6. Validates package existence: packages confirmed not to exist are dropped from `to_install` with a `Package not found` warning; packages that cannot be verified are kept with a `Could not verify` warning (skipped when `validate_packages` is `false`)
7. Finds orphans (in state but not in config) and moves them to `to_remove`
8. Computes scoop bucket additions and removals (only when scoop is enabled)

Config-file classification (`config_files_to_create` / `config_files_to_update` / `config_files_no_longer_managed`) is **not** part of `compute_diff`. Use the separate `compute_config_files_diff(result, config, state)` helper when the caller opted in with `-c`.

Note: orphan detection, upgrade detection, and bucket diff all run only for managers that are enabled and confirmed installed.
