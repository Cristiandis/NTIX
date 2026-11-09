use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::package_manager::PackageManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub version: i32,
    pub winget: HashMap<String, String>,
    pub chocolatey: HashMap<String, String>,
    pub scoop: HashMap<String, String>,
    pub scoop_buckets: HashMap<String, Option<String>>,
    #[serde(default)]
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

impl State {
    pub fn packages(&self, manager: PackageManager) -> &HashMap<String, String> {
        match manager {
            PackageManager::Winget => &self.winget,
            PackageManager::Chocolatey => &self.chocolatey,
            PackageManager::Scoop => &self.scoop,
        }
    }

    pub fn packages_mut(&mut self, manager: PackageManager) -> &mut HashMap<String, String> {
        match manager {
            PackageManager::Winget => &mut self.winget,
            PackageManager::Chocolatey => &mut self.chocolatey,
            PackageManager::Scoop => &mut self.scoop,
        }
    }
}
