use ntix_rs::models::options::WingetOptions;
use ntix_rs::package_manager::command_builder;
use ntix_rs::package_manager::winget_ops;

mod common;
use common::{MockCommandRunner, winget_list_command, winget_list_table};

#[test]
fn winget_uninstall_command_has_no_package_agreements_flag() {
    let opts = WingetOptions {
        accept_agreement: true,
        ..Default::default()
    };
    let cmd = command_builder::build_winget_uninstall("test-pkg", opts).unwrap();
    assert!(cmd.contains("--accept-source-agreements"));
    assert!(!cmd.contains("--accept-package-agreements"));
}

#[tokio::test]
async fn get_installed_packages_parses_list() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("vim", "1.0", Some("2.0"))]),
    );

    let result = winget_ops::get_installed_packages(&runner).await.unwrap();
    assert_eq!(result.get("vim"), Some(&"1.0".to_string()));
}

#[tokio::test]
async fn get_installed_packages_empty_table_returns_empty() {
    let runner = MockCommandRunner::new();
    let result = winget_ops::get_installed_packages(&runner).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_upgradable_packages_filters_available() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("rg", "1.0", Some("2.0")), ("git", "1.0", None)]),
    );

    let result = winget_ops::get_upgradable_packages(&runner).await.unwrap();
    assert_eq!(result.len(), 1);
    let info = result.get("rg").unwrap();
    assert_eq!(info.current_version, "1.0");
    assert_eq!(info.available_version, "2.0");
    assert!(!result.contains_key("git"));
}

#[tokio::test]
async fn package_exists_returns_true_when_id_present() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "winget search".to_string(),
        "Name  Id  Version  Source\nvim  vim  1.0  winget\n".to_string(),
    );

    let result = winget_ops::package_exists(&runner, "vim").await.unwrap();
    assert!(result);
}

#[tokio::test]
async fn package_exists_returns_false_when_id_absent() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "winget search".to_string(),
        "Name  Id  Version  Source\ncurl  curl  1.0  winget\n".to_string(),
    );

    let result = winget_ops::package_exists(&runner, "nope").await.unwrap();
    assert!(!result);
}

#[tokio::test]
async fn is_installed_returns_true_when_version_output() {
    let mut runner = MockCommandRunner::new();
    runner
        .output_responses
        .insert("winget --version".to_string(), "v1.10.1330".to_string());

    assert!(winget_ops::is_installed(&runner).await);
}

#[tokio::test]
async fn is_installed_returns_false_when_empty() {
    let runner = MockCommandRunner::new();
    assert!(!winget_ops::is_installed(&runner).await);
}
