# Package Managers

## Package Managers

Traits, implementations, and utilities for package manager interaction, in `ntix_rs::package_manager`.

### WingetManagerTrait

Trait for winget operations. Used for dependency injection and testing.

```rust
use ntix_rs::package_manager::winget_manager_trait::WingetManagerTrait;

#[async_trait]
pub trait WingetManagerTrait: Send + Sync {
    fn is_installed(&self) -> bool;
    async fn is_installed_async(&self) -> bool;
    async fn get_installed_packages(&self)
        -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync>>;
    async fn get_upgradable_packages(&self)
        -> Result<HashMap<String, UpgradeInfo>, Box<dyn Error + Send + Sync>>;
    async fn install(&self, id: &str, version: Option<&str>, accept_agreements: bool, silent: bool) -> bool;
    async fn uninstall(&self, id: &str, accept_agreements: bool, silent: bool) -> bool;
    async fn upgrade(&self, id: &str, accept_agreements: bool, silent: bool) -> bool;
    async fn package_exists(&self, id: &str)
        -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn ensure_installed(&self);
}
```

| Method | Description |
|--------|-------------|
| `is_installed` / `is_installed_async` | Check whether winget is available |
| `get_installed_packages` | Map of installed package ID to version |
| `get_upgradable_packages` | Map of ID to `UpgradeInfo` for packages with a newer version |
| `install` | Install a package (optionally a pinned version) |
| `uninstall` | Uninstall a package |
| `upgrade` | Upgrade a package |
| `package_exists` | Check whether a package exists in the winget repo |
| `ensure_installed` | Attempt to auto-install winget (App Installer) if missing |

### CommandRunner

Trait for running shell commands. Injected into diff, execution, and detection for testability.

```rust
use ntix_rs::package_manager::command_runner::{CommandRunner, LineCallback};

#[async_trait]
pub trait CommandRunner: Send + Sync {
    async fn run(&self, command: &str, on_output: Option<LineCallback<'_>>, on_error: Option<LineCallback<'_>>) -> i32;
    async fn run_output(&self, command: &str, combine_stderr: bool) -> String;
}
```

| Method | Description |
|--------|-------------|
| `run` | Run a command; stream each output line to the callbacks; return the exit code |
| `run_output` | Run a command and capture stdout (optionally merged with stderr) as a string |

### ManagerPresence

Trait for probing whether Chocolatey and Scoop are installed. Injected for testability.

```rust
use ntix_rs::package_manager::manager_presence::ManagerPresence;

pub trait ManagerPresence {
    fn is_chocolatey_installed(&self) -> bool;
    fn is_scoop_installed(&self) -> bool;
}
```

### Implementations

| Type | Implements | Description |
|------|-----------|-------------|
| `WingetManager` | `WingetManagerTrait` | Real winget CLI adapter (builds `winget` arguments, parses `winget list` output) |
| `ProcessCommandRunner` | `CommandRunner` | Default runner that invokes `cmd.exe /c <command>` with `CREATE_NO_WINDOW` |

### Command Building

`command_builder` safely builds shell command strings and validates package IDs:

```rust
command_builder::validate_id(id)      // Result<(), Box<dyn Error>>
command_builder::build_choco_install(id, version, opts)
command_builder::build_scoop_install(id, version, opts)
command_builder::build_choco_upgrade(id, opts)
command_builder::build_scoop_upgrade(id, opts)
command_builder::build_choco_uninstall(id, opts)
command_builder::build_winget_uninstall(id, opts)
command_builder::build_scoop_uninstall(id, opts)
command_builder::build_choco_search(id)
command_builder::build_scoop_info(id)
command_builder::build_scoop_bucket_add(name, url)
command_builder::build_scoop_bucket_list()
command_builder::build_scoop_bucket_remove(name)
```

`validate_id` rejects empty IDs and any ID containing characters outside `[a-zA-Z0-9._/-]`, guarding against shell injection.

### Table Parsing

`package_manager::table_parser::parse_table` parses the column-aligned, `--`-separated table output produced by `winget list` and related commands.

```rust
pub fn parse_table(lines: impl Iterator<Item = &str>) -> Result<(usize, Vec<String>)>
```

Returns the header column count and a flat vector of trimmed cells (header cells followed by body cells).
