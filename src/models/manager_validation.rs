#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub warnings: Vec<String>,
    pub winget_installed: bool,
    pub choco_installed: bool,
    pub scoop_installed: bool,
}
