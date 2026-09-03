#[derive(Debug, Clone, Copy, Default)]
pub struct WingetOptions {
    pub enable: bool,
    pub accept_agreement: bool,
    /// Pass `--silent` to winget (fully quiet installs; suppresses output).
    pub silent: bool,
    /// Pass `--disable-interactivity` to winget (streams output but never
    /// blocks on prompts). Ignored when `silent` is set.
    pub disable_interactivity: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ChocoOptions {
    pub enable: bool,
    pub yes: bool,
    pub force: bool,
    pub ignore_dependencies: bool,
    pub allow_downgrade: bool,
    pub skip_power_shell: bool,
    pub params: Option<String>,
    pub pre: bool,
}

#[derive(Debug, Clone)]
pub struct ScoopBucket {
    pub name: String,
    pub url: Option<String>,
}

impl ScoopBucket {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoopOptions {
    pub enable: bool,
    pub buckets: Vec<ScoopBucket>,
    pub global: bool,
    pub independent: bool,
    pub no_cache: bool,
    pub skip_hash_check: bool,
    pub arch: Option<String>,
}

impl Default for ScoopOptions {
    fn default() -> Self {
        Self {
            enable: false,
            buckets: vec![
                ScoopBucket::new("main"),
                ScoopBucket::new("extras"),
                ScoopBucket::new("versions"),
            ],
            global: false,
            independent: false,
            no_cache: false,
            skip_hash_check: false,
            arch: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NTIXOptions {
    pub winget: WingetOptions,
    pub chocolatey: ChocoOptions,
    pub scoop: ScoopOptions,
}
