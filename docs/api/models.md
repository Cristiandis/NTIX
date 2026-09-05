# Models

## Models

Data types in `ntix_rs::models`.

### PackageEntry

A package declaration from the config file.

```rust
pub struct PackageEntry {
    pub id: String,
    pub version: Option<String>,
}

impl PackageEntry {
    pub fn new(id: impl Into<String>) -> Self;
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | `String` | required | Package identifier |
| `version` | `Option<String>` | `None` | Pinned version; `None` = latest |

### PackageSpec

A resolved package with its source manager.

```rust
pub struct PackageSpec {
    pub id: String,
    pub version: Option<String>,
    pub source: PackageManager,
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Package identifier |
| `version` | `Option<String>` | Resolved version (None if unpinned) |
| `source` | `PackageManager` | `Winget`, `Chocolatey`, or `Scoop` |

### UpgradeInfo

Version information for an available upgrade. Defined in `models::installed_packages`.

```rust
pub struct UpgradeInfo {
    pub current_version: String,
    pub available_version: String,
}

impl UpgradeInfo {
    pub fn new(current_version: impl Into<String>, available_version: impl Into<String>) -> Self;
}
```

### InstalledPackages

Packages currently installed on the system. Defined in `models::installed_packages`.

```rust
pub struct InstalledPackages {
    pub winget: HashMap<String, String>,
    pub chocolatey: HashMap<String, String>,
    pub scoop: HashMap<String, String>,
}
```

Each map maps package ID to the installed version string.

### State

NTIX's tracked package state.

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub version: i32,
    pub winget: HashMap<String, String>,
    pub chocolatey: HashMap<String, String>,
    pub scoop: HashMap<String, String>,
    pub scoop_buckets: HashMap<String, Option<String>>,
    pub config_files: HashMap<String, String>,
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `version` | `i32` | `2` | State file format version |
| `winget` | `HashMap<String, String>` | empty | Tracked winget packages |
| `chocolatey` | `HashMap<String, String>` | empty | Tracked chocolatey packages |
| `scoop` | `HashMap<String, String>` | empty | Tracked scoop packages |
| `scoop_buckets` | `HashMap<String, Option<String>>` | empty | Scoop buckets added by NTIX (name to URL) |
| `config_files` | `HashMap<String, String>` | empty | Managed config files (absolute dest path to content hash) |

### DiffResult

The computed set of actions to apply.

```rust
pub struct DiffResult {
    pub to_install: Vec<PackageSpec>,
    pub to_upgrade: Vec<PackageSpec>,
    pub to_skip: Vec<PackageSpec>,
    pub to_remove: Vec<PackageSpec>,
    pub to_adopt: Vec<PackageSpec>,
    pub to_untracked: Vec<PackageSpec>,
    pub buckets_to_add: Vec<ScoopBucket>,
    pub buckets_to_remove: Vec<ScoopBucket>,
    pub config_files_to_create: Vec<ConfigFileEntry>,
    pub config_files_to_update: Vec<ConfigFileEntry>,
    pub config_files_no_longer_managed: Vec<String>,
    pub warnings: Vec<String>,
    pub manager_validation: ValidationResult,
}

impl DiffResult {
    pub fn is_empty(&self) -> bool;
}
```

| Field | Type | Description |
|-------|------|-------------|
| `to_install` | `Vec<PackageSpec>` | Packages to install |
| `to_upgrade` | `Vec<PackageSpec>` | Packages to upgrade (only with `--upgrade`) |
| `to_skip` | `Vec<PackageSpec>` | Packages already at desired state |
| `to_remove` | `Vec<PackageSpec>` | Orphaned packages to remove |
| `to_adopt` | `Vec<PackageSpec>` | Externally installed packages to adopt into state |
| `to_untracked` | `Vec<PackageSpec>` | Installed packages not tracked in state (informational only) |
| `buckets_to_add` | `Vec<ScoopBucket>` | Scoop buckets to add |
| `buckets_to_remove` | `Vec<ScoopBucket>` | Scoop buckets tracked by NTIX but no longer configured |
| `config_files_to_create` | `Vec<ConfigFileEntry>` | Files to create on disk |
| `config_files_to_update` | `Vec<ConfigFileEntry>` | Files whose source content changed |
| `config_files_no_longer_managed` | `Vec<String>` | Dest paths tracked in state but no longer in config |
| `warnings` | `Vec<String>` | Non-fatal warnings |
| `manager_validation` | `ValidationResult` | Result of the capability-detection pass |
| `is_empty()` | `bool` | True if all action lists are empty (warnings excluded) |

### ConfigFileEntry

An arbitrary file managed by NTIX, copied from `src` to `dest`. Both paths are resolved to absolute paths at config-load time.

```rust
pub struct ConfigFileEntry {
    pub dest: PathBuf,
    pub src: PathBuf,
}
```

| Field | Type | Description |
|-------|------|-------------|
| `dest` | `PathBuf` | Absolute destination path |
| `src` | `PathBuf` | Absolute source path (resolved from the config file's directory) |

### ValidationResult

Outcome of the single package-manager capability check shared by `diff` and `apply`.

```rust
pub struct ValidationResult {
    pub warnings: Vec<String>,
    pub winget_installed: bool,
    pub choco_installed: bool,
    pub scoop_installed: bool,
}
```

### ImportNode

Tracks the import tree produced while parsing configuration.

```rust
pub struct ImportNode {
    pub path: PathBuf,
    pub children: Vec<ImportNode>,
}
```

### NTIXConfig

The fully parsed configuration.

```rust
pub struct NTIXConfig {
    pub options: NTIXOptions,
    pub winget_packages: Vec<PackageEntry>,
    pub choco_packages: Vec<PackageEntry>,
    pub scoop_packages: Vec<PackageEntry>,
    pub config_files: Vec<ConfigFileEntry>,
    pub imports: Vec<ImportNode>,
}
```
