use ntix_rs::models::package_manager::PackageManager;
use ntix_rs::package_manager::package_manager_detector;

mod common;
use common::{MockCommandRunner, winget_list_command, winget_list_table};

#[tokio::test]
async fn get_installed_packages_merges_all_sources() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("vim", "1.0", None)]),
    );
    runner
        .output_responses
        .insert("choco list".to_string(), "ripgrep|14.1.0\n".to_string());
    runner.output_responses.insert(
        "scoop list".to_string(),
        "Installed apps matching '':\n\nName    Version   Source\n----    -------   ------\nfd      10.4.2    main\n"
            .to_string(),
    );

    let result = package_manager_detector::get_installed_packages(Some(&runner)).await;
    assert_eq!(result.winget.get("vim"), Some(&"1.0".to_string()));
    assert_eq!(
        result.chocolatey.get("ripgrep"),
        Some(&"14.1.0".to_string())
    );
    assert_eq!(result.scoop.get("fd"), Some(&"10.4.2".to_string()));
}

#[tokio::test]
async fn get_installed_packages_winget_empty_returns_empty_winget() {
    let runner = MockCommandRunner::new();
    let result = package_manager_detector::get_installed_packages(Some(&runner)).await;
    assert!(result.winget.is_empty());
}

#[tokio::test]
async fn get_installed_packages_choco_two_field_format_detected() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "choco list".to_string(),
        "ripgrep|14.1.0\nfd|10.4.2\n".to_string(),
    );

    let result = package_manager_detector::get_installed_packages(Some(&runner)).await;
    assert_eq!(
        result.chocolatey.get("ripgrep"),
        Some(&"14.1.0".to_string())
    );
    assert_eq!(result.chocolatey.get("fd"), Some(&"10.4.2".to_string()));
}

#[tokio::test]
async fn get_installed_packages_scoop_table_format_detected() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop list".to_string(),
        "Installed apps matching '':\n\nName    Version   Source\n----    -------   ------\nripgrep 15.2.0    main\nfd      10.4.2    main\n"
            .to_string(),
    );

    let result = package_manager_detector::get_installed_packages(Some(&runner)).await;
    assert_eq!(result.scoop.get("ripgrep"), Some(&"15.2.0".to_string()));
    assert_eq!(result.scoop.get("fd"), Some(&"10.4.2".to_string()));
}

#[tokio::test]
async fn get_installed_packages_scoop_empty_returns_empty() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop list".to_string(),
        "There aren't any apps installed.".to_string(),
    );

    let result = package_manager_detector::get_installed_packages(Some(&runner)).await;
    assert!(result.scoop.is_empty());
}

#[tokio::test]
async fn get_upgradable_packages_winget() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        winget_list_command(),
        winget_list_table(&[("rg", "1.0", Some("2.0"))]),
    );

    let result =
        package_manager_detector::get_upgradable_packages(PackageManager::Winget, Some(&runner))
            .await;
    assert_eq!(result.get("rg").unwrap().current_version, "1.0");
    assert_eq!(result.get("rg").unwrap().available_version, "2.0");
}

#[tokio::test]
async fn get_upgradable_packages_choco() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "choco outdated".to_string(),
        "git|2.30.0|2.40.0|\n".to_string(),
    );

    let result = package_manager_detector::get_upgradable_packages(
        PackageManager::Chocolatey,
        Some(&runner),
    )
    .await;
    assert_eq!(result.get("git").unwrap().current_version, "2.30.0");
    assert_eq!(result.get("git").unwrap().available_version, "2.40.0");
}

#[tokio::test]
async fn get_upgradable_packages_scoop() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop status".to_string(),
        "[{\"name\":\"rg\",\"current_version\":\"13.0.0\",\"latest_version\":\"14.0.3\"}]"
            .to_string(),
    );

    let result =
        package_manager_detector::get_upgradable_packages(PackageManager::Scoop, Some(&runner))
            .await;
    assert_eq!(result.get("rg").unwrap().current_version, "13.0.0");
    assert_eq!(result.get("rg").unwrap().available_version, "14.0.3");
}

#[tokio::test]
async fn validate_packages_exist_choco_found() {
    let mut runner = MockCommandRunner::new();
    runner
        .output_responses
        .insert("choco search git".to_string(), "git|2.40.0\n".to_string());

    let result = package_manager_detector::validate_packages_exist(
        PackageManager::Chocolatey,
        &["git".to_string()],
        Some(&runner),
    )
    .await;
    assert_eq!(result.get("git"), Some(&Some(true)));
    assert!(runner.commands().iter().any(|c| c.contains("choco search")));
}

#[tokio::test]
async fn validate_packages_exist_choco_not_found_returns_false() {
    let mut runner = MockCommandRunner::new();
    runner
        .output_responses
        .insert("choco search".to_string(), "".to_string());

    let result = package_manager_detector::validate_packages_exist(
        PackageManager::Chocolatey,
        &["nonexistent".to_string()],
        Some(&runner),
    )
    .await;
    assert_eq!(result.get("nonexistent"), Some(&Some(false)));
}

#[tokio::test]
async fn validate_packages_exist_choco_error_is_unverifiable() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "choco search".to_string(),
        "ERROR: Failed to access the source".to_string(),
    );

    let result = package_manager_detector::validate_packages_exist(
        PackageManager::Chocolatey,
        &["any-pkg".to_string()],
        Some(&runner),
    )
    .await;
    assert_eq!(result.get("any-pkg"), Some(&None));
}

#[tokio::test]
async fn validate_packages_exist_scoop_found() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop info rg".to_string(),
        "Name        : rg\nVersion     : 14.0.3\n".to_string(),
    );

    let result = package_manager_detector::validate_packages_exist(
        PackageManager::Scoop,
        &["rg".to_string()],
        Some(&runner),
    )
    .await;
    assert_eq!(result.get("rg"), Some(&Some(true)));
    assert!(runner.commands().iter().any(|c| c.contains("scoop info")));
}

#[tokio::test]
async fn validate_packages_exist_scoop_not_found_returns_false() {
    let mut runner = MockCommandRunner::new();
    runner.output_responses.insert(
        "scoop info nonexistent".to_string(),
        "Couldn't find manifest for 'nonexistent'".to_string(),
    );

    let result = package_manager_detector::validate_packages_exist(
        PackageManager::Scoop,
        &["nonexistent".to_string()],
        Some(&runner),
    )
    .await;
    assert_eq!(result.get("nonexistent"), Some(&Some(false)));
}
