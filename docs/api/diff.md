# Diff

## DiffEngine

Computes the difference between the desired state (config) and the current state.

```rust
use ntix_rs::diff::diff_engine;
```

### compute_diff

```rust
pub async fn compute_diff(
    config: &NTIXConfig,
    state: &State,
    probes: &Probes<'_>,
    adopt_mode: bool,
    upgrade_mode: bool,
    progress: &ProgressBar,
) -> Result<DiffResult, Box<dyn Error>>
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `config` | `&NTIXConfig` | required | Desired package state |
| `state` | `&State` | required | Current tracked state |
| `probes` | `&Probes<'_>` | required | Injection knobs for runner, manager presence, installed snapshot, and validation |
| `adopt_mode` | `bool` | `false` | Adopt externally installed packages into state |
| `upgrade_mode` | `bool` | `false` | Check for available upgrades |
| `progress` | `&ProgressBar` | required | Status message reporter (an indicatif spinner) |

**Returns:** `Result<DiffResult, Box<dyn Error>>`

`Probes` keeps test seams in one place. Production code passes `Probes::default()`, which uses the real process runner and live manager detection. Fields:

| Field | Type | Description |
|-------|------|-------------|
| `runner` | `Option<&dyn CommandRunner>` | Injected command runner; `ProcessCommandRunner` when `None` |
| `installed` | `Option<&InstalledPackages>` | Pre-fetched installed packages; discovered when `None` |
| `presence` | `Option<Presence>` | Fixed manager availability; probed live when `None` |
| `skip_package_validation` | `bool` | Skip per-package existence checks (which run search commands); defaults to `false` |

### Execution Flow

1. Runs a single manager capability-detection pass (`package_manager_detector::validate_managers(config, probes.runner, probes.presence)`); the resulting `ValidationResult` is stored on the diff and its warnings copied over
2. Discovers installed packages (unless `probes.installed` is set)
3. Builds one `ManagerView` per `PackageManager::ALL`, each caching the configured, installed, and tracked maps for that manager plus two flags:
   - `detected` - the manager is present on the system (gates orphan removal)
   - `enabled` - the manager is enabled in options **and** detected (gates install/upgrade classification)
4. Fetches upgradable packages for each enabled manager that has unpinned entries (only in `upgrade_mode`)
5. Classifies each declared package into `to_install`, `to_upgrade`, `to_skip`, `to_adopt`, or `to_untracked`
6. Validates new package IDs (unless `probes.skip_package_validation`): each manager's not-yet-tracked IDs are checked concurrently; packages confirmed not to exist are dropped from `to_install` with a `Package not found` warning, packages that cannot be verified are kept with a `Could not verify` warning, and already-tracked IDs are not re-checked
7. Finds orphans (tracked in state but not in config) for every **detected** manager and moves them to `to_remove`
8. Computes scoop bucket additions and removals (only when scoop is enabled, detected, and the config declares buckets)

Config-file classification (`config_files_to_create` / `config_files_to_update` / `config_files_no_longer_managed`) is **not** part of `compute_diff`. Use the separate helper when the caller opted in with `-c`:

```rust
pub fn compute_config_files_diff(
    result: &mut DiffResult,
    config: &NTIXConfig,
    state: &State,
);
```

### Gating rules

- Install/upgrade classification requires the manager to be **enabled and detected**. A manager that is enabled but missing is left untouched (and warned about by the detection pass).
- Orphan removal requires the manager to be **detected**, so packages tracked for an uninstalled manager are kept rather than removed.
- Package existence validation only covers IDs not already present in state; invalid packages are removed per manager, so the same ID under another manager is unaffected.
- Bucket handling requires scoop to be enabled, installed, and to have configured buckets.
