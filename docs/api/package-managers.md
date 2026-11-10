# Package Managers

## Package Managers

Package manager interaction lives in `ntix_rs::package_manager`.

### PackageManager

`models::package_manager::PackageManager` is the typed identifier for the supported managers, used throughout the codebase in place of `"winget"` / `"chocolatey"` / `"scoop"` string literals.

```rust
pub enum PackageManager {
    Winget,
    Chocolatey,
    Scoop,
}

impl PackageManager {
    pub const ALL: [PackageManager; 3];                  // stable presentation order
    pub fn as_str(self) -> &'static str;                 // "winget", "chocolatey", "scoop"
    pub fn display_name(self) -> &'static str;           // "Winget", "Chocolatey", "Scoop"
}

impl std::fmt::Display for PackageManager { /* writes as_str() */ }
```

`ALL` is the canonical manager order (Winget, Chocolatey, Scoop) used by the per-manager passes; `as_str` is for configs and diagnostics, `display_name` for user-facing messages.

### Probes and Presence

`package_manager::probes` centralizes the injection points that `compute_diff` and detection need for testability.

```rust
pub struct Presence {
    pub winget: bool,
    pub choco: bool,
    pub scoop: bool,
}

#[derive(Default)]
pub struct Probes<'a> {
    pub runner: Option<&'a dyn CommandRunner>,
    pub installed: Option<&'a InstalledPackages>,
    pub presence: Option<Presence>,
    pub skip_package_validation: bool,
}
```

Production code passes `Probes::default()` (real runner, live detection). Tests inject a fake runner, a prebuilt installed-package snapshot, fixed manager presence, or `skip_package_validation` to avoid search commands.

### Per-manager operations

Each manager has its own module with free `async fn`s that take a `&dyn CommandRunner`.

#### `winget_ops`

```rust
pub async fn get_installed_packages(
    runner: &dyn CommandRunner,
) -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync>>;

pub async fn get_upgradable_packages(
    runner: &dyn CommandRunner,
) -> Result<HashMap<String, UpgradeInfo>, Box<dyn Error + Send + Sync>>;

pub async fn package_exists(
    runner: &dyn CommandRunner,
    id: &str,
) -> Result<bool, Box<dyn Error + Send + Sync>>;

pub async fn is_installed(runner: &dyn CommandRunner) -> bool;

pub async fn ensure_installed(runner: &dyn CommandRunner);
```

- `get_installed_packages` / `get_upgradable_packages` parse the column-aligned `winget list --accept-source-agreements` output.
- `package_exists` runs a `winget search --id <id> --exact` and reports whether the package is present.
- `is_installed` returns whether winget is on PATH.
- `ensure_installed` auto-installs winget (App Installer) when it is missing and enabled.

#### `choco_ops`

```rust
pub async fn is_installed(runner: &dyn CommandRunner) -> bool;

pub async fn get_installed_packages(
    runner: &dyn CommandRunner,
) -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync>>;

pub async fn get_upgradable_packages(runner: &dyn CommandRunner) -> HashMap<String, UpgradeInfo>;

pub async fn package_exists(
    runner: &dyn CommandRunner,
    id: &str,
) -> Result<Option<bool>, Box<dyn Error + Send + Sync>>;
```

`package_exists` returns a tri-state: `Some(true)` when the package is found, `Some(false)` when `choco search --limit-output` unambiguously reports it absent, and `None` when the query output is inconclusive (a failed or unexpected response).

#### `scoop_ops`

```rust
pub async fn is_installed(runner: &dyn CommandRunner) -> bool;

pub async fn get_installed_packages(
    runner: &dyn CommandRunner,
) -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync>>;

pub async fn get_upgradable_packages(runner: &dyn CommandRunner) -> HashMap<String, UpgradeInfo>;

pub async fn package_exists(
    runner: &dyn CommandRunner,
    id: &str,
) -> Result<Option<bool>, Box<dyn Error + Send + Sync>>;
```

As with choco, `package_exists` is tri-state: `Some(true)` found, `Some(false)` not found (`Couldn't find manifest`), `None` when the response cannot be trusted. Its upgradable map comes from `scoop status --json` and is empty on a failed query.

### CommandRunner

Trait for running shell commands. Injected into diff, execution, and detection for testability.

```rust
use ntix_rs::package_manager::command_runner::{CommandRunner, LineCallback};

pub type LineCallback<'a> = &'a (dyn Fn(&str) + Sync);

#[async_trait]
pub trait CommandRunner {
    async fn run(
        &self,
        command: &str,
        on_output: Option<LineCallback<'_>>,
        on_error: Option<LineCallback<'_>>,
    ) -> i32;
    async fn run_output(&self, command: &str, combine_stderr: bool) -> String;
}
```

| Method | Description |
|--------|-------------|
| `run` | Run a command; stream each output line to the callbacks; return the exit code |
| `run_output` | Run a command and capture stdout (optionally merged with stderr) as a string |

`ProcessCommandRunner` is the default runner: it invokes `cmd.exe /c <command>` with `CREATE_NO_WINDOW` and streams both pipes line by line.

### Detection and validation

`package_manager_detector` performs a single capability-detection pass shared by `diff` and `apply`, and validates package availability.

```rust
pub async fn validate_managers(
    config: &NTIXConfig,
    runner: Option<&dyn CommandRunner>,
    presence: Option<Presence>,
) -> ValidationResult;

pub fn collect_warnings(config: &NTIXConfig, validation: &ValidationResult) -> Vec<String>;

pub async fn get_installed_packages(runner: Option<&dyn CommandRunner>) -> InstalledPackages;

pub async fn get_upgradable_packages(
    manager: PackageManager,
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, UpgradeInfo>;

pub async fn validate_packages_exist(
    manager: PackageManager,
    ids: &[String],
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, Option<bool>>;
```

- `validate_managers` uses injected `presence` when given; otherwise it detects each manager. Only winget is auto-installed (when enabled but missing), after which it is re-detected.
- `collect_warnings` produces the enabled-but-missing and declared-but-disabled warnings.
- `get_installed_packages` queries all three managers concurrently and merges the results; a failed manager logs a diagnostic and contributes nothing.
- `get_upgradable_packages` dispatches on the manager; a failed winget query degrades to an empty map.
- `validate_packages_exist` checks each ID concurrently and maps it to `Some(true)` (found), `Some(false)` (definitively absent), or `None` (could not verify).

`ValidationResult` (in `models::manager_validation`) carries the outcome:

```rust
pub struct ValidationResult {
    pub warnings: Vec<String>,
    pub winget_installed: bool,
    pub choco_installed: bool,
    pub scoop_installed: bool,
}

impl ValidationResult {
    pub fn installed(&self, manager: PackageManager) -> bool;
}
```

`compute_diff` validates only packages that are not already tracked in state. It turns `Some(false)` into a `Package not found in <source>` warning and drops the package from `to_install` (per manager), while `None` produces a `Could not verify` warning and keeps the package.

### Command Building

`command_builder` safely builds shell command strings and validates package IDs and bucket URLs:

```rust
command_builder::validate_id(id)                          // Result<(), Box<dyn Error>>
command_builder::validate_url(url)                        // Result<(), Box<dyn Error>>
command_builder::build_winget_install(id, version, opts)
command_builder::build_winget_upgrade(id, opts)
command_builder::build_winget_uninstall(id, opts)
command_builder::build_winget_list()
command_builder::build_winget_search(id)
command_builder::build_winget_version()
command_builder::build_choco_install(id, version, opts)
command_builder::build_choco_upgrade(id, opts)
command_builder::build_choco_uninstall(id, opts)
command_builder::build_choco_search(id)
command_builder::build_choco_version()
command_builder::build_choco_list_installed()
command_builder::build_choco_outdated()
command_builder::build_scoop_install(id, version, opts)
command_builder::build_scoop_upgrade(id, opts)
command_builder::build_scoop_uninstall(id, opts)
command_builder::build_scoop_info(id)
command_builder::build_scoop_version()
command_builder::build_scoop_list_installed()
command_builder::build_scoop_status()
command_builder::build_scoop_bucket_add(name, url)
command_builder::build_scoop_bucket_remove(name)
command_builder::build_scoop_bucket_list()
command_builder::build_powershell_install_winget()
```

`validate_id` rejects empty IDs and any ID containing characters outside `[a-zA-Z0-9._\-/]`, guarding against shell injection. `validate_url` rejects empty URLs, URLs starting with `-` (option injection), and any URL containing characters outside `[a-zA-Z0-9._~/@:?\-]` (blocking spaces, `&`, `|`, quotes, `%`). Builders that interpolate a user-supplied ID, version, or bucket URL call them first.

### Table Parsing

`package_manager::table_parser::parse_table` parses the column-aligned, `--`-separated table output produced by `winget list` and related commands.

```rust
pub fn parse_table<'a>(
    lines: impl Iterator<Item = &'a str>,
) -> Result<(usize, Vec<String>), Box<dyn std::error::Error>>
```

Returns the header column count and a flat vector of trimmed cells (header cells followed by body cells). Column offsets are derived from the header row itself.
