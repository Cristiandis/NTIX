# Package Managers

## Package Managers

Package manager interaction lives in `ntix_rs::package_manager`.

### PackageManager

`models::package_manager::PackageManager` is the typed identifier for the supported managers, used throughout the codebase in place of the old `"winget"` / `"chocolatey"` / `"scoop"` string literals.

```rust
pub enum PackageManager {
    Winget,      // default
    Chocolatey,
    Scoop,
}

impl PackageManager {
    pub fn as_str(self) -> &'static str;                 // "winget", "chocolatey", "scoop"
    pub fn from_name(s: &str) -> Option<Self>;           // case-insensitive lookup
    pub fn all() -> [PackageManager; 3];                 // stable order
}
```

### Per-manager operations

Each manager has its own module with free functions `async fn`s that take a `&dyn CommandRunner`.

#### `winget_ops`

```rust
pub struct WingetPackageEntry {
    pub id: String,
    pub version: String,
    pub available: Option<String>,
}

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

- `get_installed_packages` / `get_upgradable_packages` parse the column-aligned `winget list --accept-source-agreements --upgrade` output.
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

As with choco, `package_exists` is tri-state: `Some(true)` found, `Some(false)` not found (`Couldn't find manifest`), `None` when the response cannot be trusted.

Compared to winget, `get_upgradable_packages` here returns a plain `HashMap` and degrades to an empty map on a failed query instead of erroring.

### CommandRunner

Trait for running shell commands. Injected into diff, execution, and detection for testability.

```rust
use ntix_rs::package_manager::command_runner::{CommandRunner, LineCallback};

pub type LineCallback<'a> = &'a (dyn Fn(&str) + Sync);

#[async_trait]
pub trait CommandRunner: Send + Sync {
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
pub async fn validate_managers_async(
    options: &NTIXOptions,
    config: &NTIXConfig,
    winget_installed: Option<bool>,
    choco_installed: Option<bool>,
    scoop_installed: Option<bool>,
    runner: Option<&dyn CommandRunner>,
) -> ValidationResult;

pub fn collect_warnings(
    options: &NTIXOptions,
    winget_installed: bool,
    choco_installed: bool,
    scoop_installed: bool,
) -> Vec<String>;

pub async fn get_installed_packages_async(runner: Option<&dyn CommandRunner>) -> InstalledPackages;

pub async fn get_winget_upgradable_packages_async(
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, UpgradeInfo>;
pub async fn get_choco_upgradable_packages_async(
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, UpgradeInfo>;
pub async fn get_scoop_upgradable_packages_async(
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, UpgradeInfo>;

pub async fn validate_winget_packages_exist_async(
    package_ids: &[String],
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, Option<bool>>;
pub async fn validate_choco_packages_exist_async(
    package_ids: &[String],
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, Option<bool>>;
pub async fn validate_scoop_packages_exist_async(
    package_ids: &[String],
    runner: Option<&dyn CommandRunner>,
) -> HashMap<String, Option<bool>>;

pub async fn validate_choco_package_exists_async(
    id: &str,
    runner: Option<&dyn CommandRunner>,
) -> Option<bool>;
pub async fn validate_scoop_package_exists_async(
    id: &str,
    runner: Option<&dyn CommandRunner>,
) -> Option<bool>;
```

`ValidationResult` (in `models::manager_validation`) carries the outcome:

```rust
pub struct ValidationResult {
    pub warnings: Vec<String>,
    pub winget_installed: bool,
    pub choco_installed: bool,
    pub scoop_installed: bool,
}
```

The existence validators map each package ID to `Some(true)` (found), `Some(false)` (definitively absent), or `None` (could not verify). `compute_diff` turns `Some(false)` into a `Package not found in <source>` warning and drops the package, while `None` produces a `Could not verify` warning and keeps the package.

### Command Building

`command_builder` safely builds shell command strings and validates package IDs:

```rust
command_builder::validate_id(id)                          // Result<(), Box<dyn Error>>
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

`validate_id` rejects empty IDs and any ID containing characters outside `[a-zA-Z0-9._\-/]`, guarding against shell injection. Builders that interpolate a user-supplied ID or version call it first.

### Table Parsing

`package_manager::table_parser::parse_table` parses the column-aligned, `--`-separated table output produced by `winget list` and related commands.

```rust
pub fn parse_table<'a>(lines: impl Iterator<Item = &'a str>) -> Result<(usize, Vec<String>)>
```

Returns the header column count and a flat vector of trimmed cells (header cells followed by body cells). Column offsets are derived from the header row itself.