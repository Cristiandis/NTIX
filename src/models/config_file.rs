use std::path::PathBuf;

/// An arbitrary file to be managed by NTIX, copied from `src` to `dest`.
///
/// Both paths are resolved to absolute paths at config-load time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFileEntry {
    pub dest: PathBuf,
    pub src: PathBuf,
}
