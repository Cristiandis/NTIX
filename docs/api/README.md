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
| `ntix_rs::package_manager` | Package manager abstraction and detection |
| `ntix_rs::lock` | Concurrent execution locking |
| `ntix_rs::paths` | Default paths (`%LOCALAPPDATA%/ntix`) |
| `ntix_rs::process_helper` | Admin / token membership checks |

### Key Types

| Type | Description |
|------|-------------|
| `NTIXConfig` | Parsed config (options + package lists + import tree) |
| `NTIXOptions` | Per-manager behavior options |
| `State` | Current tracked packages and scoop buckets |
| `DiffResult` | Computed actions (install/upgrade/remove/adopt/skip) |
| `InstalledPackages` | Packages found on the system |
| `WingetManagerTrait` | Abstraction over the winget CLI (mockable) |
| `CommandRunner` | Abstraction over shell commands (mockable) |
| `ManagerPresence` | Abstraction over choco/scoop availability (mockable) |
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

let config = config_loader::load("config.lua".into())?;
let mut state = state_service::load_state(None).unwrap_or_default();

let progress = indicatif::ProgressBar::new_spinner();
let diff = diff_engine::compute_diff(
    &config, &state, None, None, None, false, false, true, None, &progress,
).await?;

let state_path = state_service::get_state_path()?;
let success = execution_engine::apply_diff(
    &diff, &config.options, &mut state, &state_path, false,
    None, None, Some(&config),
    Some(&|line: &str| println!("{line}")),
    Some(&|err: &str| eprintln!("{err}")),
    None,
).await;
```
