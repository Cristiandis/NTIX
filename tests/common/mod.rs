#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use ntix_rs::models::installed_packages::UpgradeInfo;
use ntix_rs::package_manager::command_runner::{CommandRunner, LineCallback};
use ntix_rs::package_manager::manager_presence::ManagerPresence;
use ntix_rs::package_manager::winget_manager_trait::WingetManagerTrait;

/// Hand-rolled mock of `ManagerPresence`.
pub struct MockManagerPresence {
    pub chocolatey_installed: bool,
    pub scoop_installed: bool,
}

impl MockManagerPresence {
    pub fn new() -> Self {
        Self {
            chocolatey_installed: true,
            scoop_installed: true,
        }
    }

    pub fn with_choco(installed: bool) -> Self {
        Self {
            chocolatey_installed: installed,
            scoop_installed: true,
        }
    }
}

impl Default for MockManagerPresence {
    fn default() -> Self {
        Self::new()
    }
}

impl ManagerPresence for MockManagerPresence {
    fn is_chocolatey_installed(&self) -> bool {
        self.chocolatey_installed
    }

    fn is_scoop_installed(&self) -> bool {
        self.scoop_installed
    }
}

/// Hand-rolled mock of `CommandRunner`.
pub struct MockCommandRunner {
    pub captured_commands: Mutex<Vec<String>>,
    pub run_handler: Mutex<Option<Box<dyn Fn(&str) -> i32 + Send + Sync>>>,
    pub output_responses: HashMap<String, String>,
}

impl MockCommandRunner {
    pub fn new() -> Self {
        Self {
            captured_commands: Mutex::new(Vec::new()),
            run_handler: Mutex::new(None),
            output_responses: HashMap::new(),
        }
    }

    pub fn commands(&self) -> Vec<String> {
        self.captured_commands.lock().unwrap().clone()
    }
}

impl Default for MockCommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandRunner for MockCommandRunner {
    async fn run(
        &self,
        command: &str,
        _on_output: Option<LineCallback<'_>>,
        _on_error: Option<LineCallback<'_>>,
    ) -> i32 {
        self.captured_commands
            .lock()
            .unwrap()
            .push(command.to_string());

        if let Some(handler) = self.run_handler.lock().unwrap().as_ref() {
            return handler(command);
        }

        0
    }

    async fn run_output(&self, command: &str, _combine_stderr: bool) -> String {
        self.captured_commands
            .lock()
            .unwrap()
            .push(command.to_string());

        if let Some(output) = self.output_responses.get(command) {
            return output.clone();
        }

        for (key, output) in &self.output_responses {
            if command.contains(key) {
                return output.clone();
            }
        }

        String::new()
    }
}

/// Hand-rolled mock of `WingetManagerTrait`.
pub struct MockWingetManager {
    pub is_installed: bool,
    pub installed_packages: HashMap<String, String>,
    pub upgradable_packages: HashMap<String, UpgradeInfo>,
    pub package_exists_result: Option<bool>,
    pub install_result: bool,
    pub install_per_id: Option<Vec<(String, bool)>>,
    pub uninstall_result: bool,
    pub upgrade_result: bool,
    pub install_calls: Mutex<Vec<(String, Option<String>, bool, bool)>>,
    pub upgrade_calls: Mutex<usize>,
    pub package_exists_calls: Mutex<Vec<String>>,
    pub package_exists_error: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub package_exists_by_id: Option<HashMap<String, bool>>,
    pub package_exists_throw: bool,
}

impl MockWingetManager {
    pub fn new() -> Self {
        Self {
            is_installed: true,
            installed_packages: HashMap::new(),
            upgradable_packages: HashMap::new(),
            package_exists_result: None,
            install_result: true,
            install_per_id: None,
            uninstall_result: true,
            upgrade_result: true,
            install_calls: Mutex::new(Vec::new()),
            upgrade_calls: Mutex::new(0),
            package_exists_calls: Mutex::new(Vec::new()),
            package_exists_error: None,
            package_exists_by_id: None,
            package_exists_throw: false,
        }
    }

    pub fn install_call_count(&self) -> usize {
        self.install_calls.lock().unwrap().len()
    }

    pub fn upgrade_call_count(&self) -> usize {
        *self.upgrade_calls.lock().unwrap()
    }

    pub fn package_exists_call_count(&self) -> usize {
        self.package_exists_calls.lock().unwrap().len()
    }
}

impl Default for MockWingetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WingetManagerTrait for MockWingetManager {
    fn is_installed(&self) -> bool {
        self.is_installed
    }

    async fn is_installed_async(&self) -> bool {
        self.is_installed
    }

    async fn get_installed_packages(
        &self,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.installed_packages.clone())
    }

    async fn get_upgradable_packages(
        &self,
    ) -> Result<HashMap<String, UpgradeInfo>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.upgradable_packages.clone())
    }

    async fn install(
        &self,
        id: &str,
        version: Option<&str>,
        accept_agreements: bool,
        silent: bool,
    ) -> bool {
        self.install_calls.lock().unwrap().push((
            id.to_string(),
            version.map(|s| s.to_string()),
            accept_agreements,
            silent,
        ));
        if let Some(per_id) = &self.install_per_id {
            for (pid, result) in per_id {
                if pid == id {
                    return *result;
                }
            }
        }
        self.install_result
    }

    async fn uninstall(&self, id: &str, _accept_agreements: bool, _silent: bool) -> bool {
        let _ = id;
        self.uninstall_result
    }

    async fn upgrade(&self, id: &str, _accept_agreements: bool, _silent: bool) -> bool {
        let _ = id;
        *self.upgrade_calls.lock().unwrap() += 1;
        self.upgrade_result
    }

    async fn package_exists(
        &self,
        id: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        self.package_exists_calls
            .lock()
            .unwrap()
            .push(id.to_string());
        if self.package_exists_throw {
            return Err("network error".into());
        }
        if let Some(map) = &self.package_exists_by_id {
            return Ok(*map.get(id).unwrap_or(&true));
        }
        Ok(self.package_exists_result.unwrap_or(true))
    }

    async fn ensure_installed(&self) {}
}
