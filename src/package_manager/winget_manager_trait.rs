use async_trait::async_trait;
use std::collections::HashMap;

use crate::models::installed_packages::UpgradeInfo;
use crate::models::options::WingetOptions;
use crate::package_manager::command_runner::LineCallback;

#[async_trait]
pub trait WingetManagerTrait: Send + Sync {
    fn is_installed(&self) -> bool;
    async fn is_installed_async(&self) -> bool;
    async fn get_installed_packages(
        &self,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_upgradable_packages(
        &self,
    ) -> Result<HashMap<String, UpgradeInfo>, Box<dyn std::error::Error + Send + Sync>>;
    async fn install(
        &self,
        id: &str,
        version: Option<&str>,
        options: WingetOptions,
        on_output: Option<LineCallback<'_>>,
        on_error: Option<LineCallback<'_>>,
    ) -> bool;
    async fn uninstall(
        &self,
        id: &str,
        options: WingetOptions,
        on_output: Option<LineCallback<'_>>,
        on_error: Option<LineCallback<'_>>,
    ) -> bool;
    async fn upgrade(
        &self,
        id: &str,
        options: WingetOptions,
        on_output: Option<LineCallback<'_>>,
        on_error: Option<LineCallback<'_>>,
    ) -> bool;
    async fn package_exists(
        &self,
        id: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn ensure_installed(&self);
}
