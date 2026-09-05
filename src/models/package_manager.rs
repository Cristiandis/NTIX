use std::fmt;

/// The package managers ntix can operate on. Used as a typed replacement for the
/// string literals `"winget"` / `"chocolatey"` / `"scoop"` that were previously
/// matched by hand throughout the codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum PackageManager {
    #[default]
    Winget,
    Chocolatey,
    Scoop,
}

impl PackageManager {
    /// The canonical lowercase name used in configs, messages and diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            PackageManager::Winget => "winget",
            PackageManager::Chocolatey => "chocolatey",
            PackageManager::Scoop => "scoop",
        }
    }

    /// Case-insensitive lookup of a manager by name.
    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "winget" => Some(PackageManager::Winget),
            "chocolatey" => Some(PackageManager::Chocolatey),
            "scoop" => Some(PackageManager::Scoop),
            _ => None,
        }
    }

    /// All supported managers, in a stable order.
    pub fn all() -> [PackageManager; 3] {
        [
            PackageManager::Winget,
            PackageManager::Chocolatey,
            PackageManager::Scoop,
        ]
    }
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
