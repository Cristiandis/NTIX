# ntix-rs API Reference

## ntix-rs API Reference

`ntix-rs` is the Rust crate behind the NTIX binary. It provides config parsing, diff computation, package execution, state management, and file locking. The core library is pure data and logic with no console I/O; the binary wrapper (`src/main.rs`) owns all printing and the interactive spinner.

### Crate

```
ntix-rs (2024 edition)
```

Key dependencies: `mlua` (embedded Lua 5.4), `tokio`, `clap`, `serde_json`, `regex`, `anyhow`, `async-trait`, `indicatif`, `colored`, `windows`.

### Modules

| Module | Purpose |
|--------|---------|
| `ntix_rs::config` | Lua config file loading |
| `ntix_rs::models` | Data models, options, and config structs |
| `ntix_rs::diff` | Diff computation (desired vs current state) |
| `ntix_rs::execution` | Package install/upgrade/remove execution |
| `ntix_rs::state_management` | State file persistence |
| `ntix_rs::package_manager` | Per-manager operations (`winget_ops`, `choco_ops`, `scoop_ops`), command building, table parsing, detection |
| `ntix_rs::lock` | Concurrent execution locking |
| `ntix_rs::paths` | Default paths (`%LOCALAPPDATA%/ntix`) |
| `ntix_rs::process_helper` | Admin / token membership checks |
| `ntix_rs::hash` | File content hashing (config-file tracking) |

### Key Types

| Type | Description |
|------|-------------|
| `NTIXConfig` | Parsed config (options + package lists + config files + import tree) |
| `NTIXOptions` | Per-manager behavior options |
| `State` | Current tracked packages, scoop buckets, config files |
| `DiffResult` | Computed actions (install/upgrade/remove/adopt/untracked/config files/warnings) |
| `InstalledPackages` | Packages found on the system (per-manager maps) |
| `PackageManager` | Typed manager identifier (`Winget`, `Chocolatey`, `Scoop`) |
| `ValidationResult` | Manager capability check shared by diff and apply |
| `ConfigFileEntry` | A source/destination pair for a managed file |
| `CommandRunner` | Abstraction over shell commands (mockable) |
| `ProcessCommandRunner` | Default `CommandRunner` using `cmd.exe` |
| `parse_table` | Parses the column-aligned output of package managers |
| `DiffEngine` | `diff::diff_engine::compute_diff` |
| `ExecutionEngine` | `execution::execution_engine::apply_diff` |
| `StateService` | `state_management::state_service` |
| `LockFile` | `lock::lock_file::LockFile` |

### Quick Start

```rust
use ntix_rs::config::config_loader;
use ntix_rs::state_management::state_service;
use ntix_rs::diff::diff_engine;
use ntix_rs::execution::execution_engine;
use ntix_rs::package_manager::package_manager_detector;

let config = config_loader::load("config.lua".into())?;
let mut state = state_service::load_state(None).unwrap_or_default();

let progress = indicatif::ProgressBar::new_spinner();
let validation = package_manager_detector::validate_managers_async(
    &config.options, &config, None, None, None, None,
).await;

let diff = diff_engine::compute_diff(
    &config, &state,
    validation.winget_installed.then_some(true),
    validation.choco_installed.then_some(true),
    validation.scoop_installed.then_some(true),
    None, false, false, true, None, &progress,
).await?;

let state_path = state_service::get_state_path()?;
let success = execution_engine::apply_diff(
    &diff, &config.options, &mut state, &state_path, false,
    &validation, false,
    Some(&|line: &str| println!("{line}")),
    Some(&|err: &str| eprintln!("{err}")),
    None,
).await;
```
