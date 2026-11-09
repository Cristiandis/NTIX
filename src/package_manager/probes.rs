use crate::models::installed_packages::InstalledPackages;

use super::command_runner::CommandRunner;

/// Presence of managers on the system, either detected live or injected.
#[derive(Debug, Clone, Copy, Default)]
pub struct Presence {
    pub winget: bool,
    pub choco: bool,
    pub scoop: bool,
}

/// Injection knobs for probe-dependent operations. Production code passes
/// `Probes::default()` (real process runner, live detection); tests inject a
/// fake runner, a prebuilt installed-package snapshot, or fixed manager
/// presence to skip live detection.
#[derive(Default)]
pub struct Probes<'a> {
    pub runner: Option<&'a dyn CommandRunner>,
    pub installed: Option<&'a InstalledPackages>,
    pub presence: Option<Presence>,
    /// Skip per-package existence validation, which runs search commands.
    pub skip_package_validation: bool,
}
