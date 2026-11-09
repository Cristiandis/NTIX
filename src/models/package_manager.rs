use std::fmt;

/// The package managers ntix can operate on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageManager {
    Winget,
    Chocolatey,
    Scoop,
}

impl PackageManager {
    /// Every supported manager, in the canonical presentation order.
    pub const ALL: [PackageManager; 3] = [
        PackageManager::Winget,
        PackageManager::Chocolatey,
        PackageManager::Scoop,
    ];

    /// The canonical lowercase name used in configs, messages and diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            PackageManager::Winget => "winget",
            PackageManager::Chocolatey => "chocolatey",
            PackageManager::Scoop => "scoop",
        }
    }

    /// The capitalized name used in user-facing messages.
    pub fn display_name(self) -> &'static str {
        match self {
            PackageManager::Winget => "Winget",
            PackageManager::Chocolatey => "Chocolatey",
            PackageManager::Scoop => "Scoop",
        }
    }
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
