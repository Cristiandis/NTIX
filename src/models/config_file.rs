use std::path::PathBuf;

/// An arbitrary file to be managed by NTIX, copied from `src` to `dest`.
///
/// Both paths are resolved to absolute paths at config-load time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFileEntry {
    /// Absolute path of the destination file on the system.
    pub dest: PathBuf,
    /// Absolute path of the source file (resolved from the config file's directory).
    pub src: PathBuf,
}
