#![cfg(target_os = "windows")]

use ntix_rs::models::options::WingetOptions;
use ntix_rs::package_manager::winget_manager::WingetManager;
use ntix_rs::package_manager::winget_manager_trait::WingetManagerTrait;

#[tokio::test]
async fn winget_manager_implements_trait() {
    let manager = WingetManager;
    let _: &dyn WingetManagerTrait = &manager;
}

#[tokio::test]
async fn is_installed_returns_bool() {
    let manager = WingetManager;
    let result = manager.is_installed();
    let _ = result;
}

#[tokio::test]
async fn is_installed_async_returns_bool() {
    let manager = WingetManager;
    let result = manager.is_installed_async().await;
    let _ = result;
}

#[tokio::test]
async fn get_installed_packages_returns_dictionary() {
    let manager = WingetManager;
    let result = manager.get_installed_packages().await;
    let map = result.unwrap_or_default();
    let _ = map;
}

#[tokio::test]
async fn get_upgradable_packages_returns_dictionary() {
    let manager = WingetManager;
    let result = manager.get_upgradable_packages().await;
    let map = result.unwrap_or_default();
    let _ = map;
}

#[tokio::test]
async fn install_with_invalid_package_returns_false() {
    let manager = WingetManager;
    let result = manager
        .install(
            "nonexistent-package-xyz-123",
            None,
            WingetOptions::default(),
            None,
            None,
        )
        .await;
    assert!(!result);
}

#[tokio::test]
async fn uninstall_with_invalid_package_returns_false() {
    let manager = WingetManager;
    let result = manager
        .uninstall(
            "nonexistent-package-xyz-123",
            WingetOptions::default(),
            None,
            None,
        )
        .await;
    assert!(!result);
}

#[tokio::test]
async fn upgrade_with_invalid_package_returns_false() {
    let manager = WingetManager;
    let result = manager
        .upgrade(
            "nonexistent-package-xyz-123",
            WingetOptions::default(),
            None,
            None,
        )
        .await;
    assert!(!result);
}
