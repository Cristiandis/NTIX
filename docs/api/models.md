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
    pub source: String,
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Package identifier |
| `version` | `Option<String>` | Resolved version (None if unpinned) |
| `source` | `String` | `"winget"`, `"chocolatey"`, or `"scoop"` |

### UpgradeInfo

Version information for an available upgrade.

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

Packages currently installed on the system.

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
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `version` | `i32` | `1` | State file format version |
| `winget` | `HashMap<String, String>` | empty | Tracked winget packages |
| `chocolatey` | `HashMap<String, String>` | empty | Tracked chocolatey packages |
| `scoop` | `HashMap<String, String>` | empty | Tracked scoop packages |
| `scoop_buckets` | `HashMap<String, Option<String>>` | empty | Scoop buckets added by NTIX (name to URL) |

### DiffResult

The computed set of actions to apply.

```rust
pub struct DiffResult {
    pub to_install: Vec<PackageSpec>,
    pub to_upgrade: Vec<PackageSpec>,
    pub to_skip: Vec<PackageSpec>,
    pub to_remove: Vec<PackageSpec>,
    pub to_adopt: Vec<PackageSpec>,
    pub buckets_to_add: Vec<ScoopBucket>,
    pub buckets_to_remove: Vec<ScoopBucket>,
    pub warnings: Vec<String>,
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
| `buckets_to_add` | `Vec<ScoopBucket>` | Scoop buckets to add |
| `buckets_to_remove` | `Vec<ScoopBucket>` | Scoop buckets tracked by NTIX but no longer configured |
| `warnings` | `Vec<String>` | Non-fatal warnings |
| `is_empty()` | `bool` | True if all action lists are empty (warnings excluded) |

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
    pub imports: Vec<ImportNode>,
}
```
