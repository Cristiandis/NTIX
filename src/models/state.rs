use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub version: i32,
    pub winget: HashMap<String, String>,
    pub chocolatey: HashMap<String, String>,
    pub scoop: HashMap<String, String>,
    pub scoop_buckets: HashMap<String, Option<String>>,
    /// Managed config files: dest (absolute path) -> sha256 of the last applied source content.
    pub config_files: HashMap<String, String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: 2,
            winget: HashMap::new(),
            chocolatey: HashMap::new(),
            scoop: HashMap::new(),
            scoop_buckets: HashMap::new(),
            config_files: HashMap::new(),
        }
    }
}
