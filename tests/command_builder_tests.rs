use ntix_rs::models::options::{ChocoOptions, ScoopOptions};
use ntix_rs::package_manager::command_builder;

#[test]
fn build_choco_install_basic() {
    let cmd = command_builder::build_choco_install("test", None, ChocoOptions::default()).unwrap();
    assert_eq!(cmd, "choco install test");
}

#[test]
fn build_choco_install_with_yes() {
    let opts = ChocoOptions {
        yes: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_install("test", None, opts).unwrap();
    assert!(cmd.contains("-y"));
}

#[test]
fn build_choco_install_with_version_uses_version_flag() {
    let opts = ChocoOptions {
        yes: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_install("nodejs", Some("16.14.2"), opts).unwrap();
    assert_eq!(cmd, "choco install nodejs --version 16.14.2 -y");
}

#[test]
fn build_choco_install_without_version_no_version_flag() {
    let cmd =
        command_builder::build_choco_install("nodejs", None, ChocoOptions::default()).unwrap();
    assert_eq!(cmd, "choco install nodejs");
}

#[test]
fn build_choco_install_with_force() {
    let opts = ChocoOptions {
        force: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_install("test", None, opts).unwrap();
    assert!(cmd.contains("--force"));
}

#[test]
fn build_choco_install_with_ignore_dependencies() {
    let opts = ChocoOptions {
        ignore_dependencies: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_install("test", None, opts).unwrap();
    assert!(cmd.contains("--ignore-dependencies"));
}

#[test]
fn build_choco_install_with_allow_downgrade() {
    let opts = ChocoOptions {
        allow_downgrade: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_install("test", None, opts).unwrap();
    assert!(cmd.contains("--allow-downgrade"));
}

#[test]
fn build_choco_install_with_skip_power_shell() {
    let opts = ChocoOptions {
        skip_power_shell: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_install("test", None, opts).unwrap();
    assert!(cmd.contains("--skip-scripts"));
}

#[test]
fn build_choco_install_with_pre() {
    let opts = ChocoOptions {
        pre: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_install("test", None, opts).unwrap();
    assert!(cmd.contains("--pre"));
}

#[test]
fn build_choco_install_with_params() {
    let opts = ChocoOptions {
        params: Some("/GitAndUnixToolsOnPath".to_string()),
        ..Default::default()
    };
    let cmd = command_builder::build_choco_install("git", None, opts).unwrap();
    assert!(cmd.contains("--params=\"'/GitAndUnixToolsOnPath'\""));
}

#[test]
fn build_choco_install_all_flags() {
    let opts = ChocoOptions {
        yes: true,
        force: true,
        ignore_dependencies: true,
        allow_downgrade: true,
        skip_power_shell: true,
        pre: true,
        params: Some("/GitAndUnixToolsOnPath".to_string()),
        ..Default::default()
    };
    let cmd = command_builder::build_choco_install("git", Some("2.40.0"), opts).unwrap();
    assert_eq!(
        cmd,
        "choco install git --version 2.40.0 -y --force --ignore-dependencies --allow-downgrade --skip-scripts --pre --params=\"'/GitAndUnixToolsOnPath'\""
    );
}

#[test]
fn build_scoop_install_basic() {
    let cmd =
        command_builder::build_scoop_install("nodejs", None, ScoopOptions::default()).unwrap();
    assert_eq!(cmd, "scoop install nodejs");
}

#[test]
fn build_scoop_install_with_version_uses_at_syntax() {
    let cmd =
        command_builder::build_scoop_install("nodejs", Some("16.14.2"), ScoopOptions::default())
            .unwrap();
    assert_eq!(cmd, "scoop install nodejs@16.14.2");
}

#[test]
fn build_scoop_install_global() {
    let opts = ScoopOptions {
        global: true,
        ..Default::default()
    };
    let cmd = command_builder::build_scoop_install("nodejs", None, opts).unwrap();
    assert!(cmd.contains("-g"));
}

#[test]
fn build_scoop_install_independent() {
    let opts = ScoopOptions {
        independent: true,
        ..Default::default()
    };
    let cmd = command_builder::build_scoop_install("nodejs", None, opts).unwrap();
    assert!(cmd.contains("-i"));
}

#[test]
fn build_scoop_install_no_cache() {
    let opts = ScoopOptions {
        no_cache: true,
        ..Default::default()
    };
    let cmd = command_builder::build_scoop_install("nodejs", None, opts).unwrap();
    assert!(cmd.contains("-k"));
}

#[test]
fn build_scoop_install_skip_hash_check() {
    let opts = ScoopOptions {
        skip_hash_check: true,
        ..Default::default()
    };
    let cmd = command_builder::build_scoop_install("nodejs", None, opts).unwrap();
    assert!(cmd.contains("-s"));
}

#[test]
fn build_scoop_install_arch() {
    let opts = ScoopOptions {
        arch: Some("64bit".to_string()),
        ..Default::default()
    };
    let cmd = command_builder::build_scoop_install("nodejs", None, opts).unwrap();
    assert!(cmd.contains("--arch 64bit"));
}

#[test]
fn build_scoop_install_all_flags() {
    let opts = ScoopOptions {
        global: true,
        independent: true,
        no_cache: true,
        skip_hash_check: true,
        arch: Some("64bit".to_string()),
        ..Default::default()
    };
    let cmd = command_builder::build_scoop_install("nodejs", Some("20.0.0"), opts).unwrap();
    assert_eq!(cmd, "scoop install nodejs@20.0.0 -g -i -k -s --arch 64bit");
}

#[test]
fn build_choco_upgrade_with_yes() {
    let opts = ChocoOptions {
        yes: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_upgrade("nodejs", opts).unwrap();
    assert_eq!(cmd, "choco upgrade nodejs -y");
}

#[test]
fn build_choco_upgrade_basic() {
    let cmd = command_builder::build_choco_upgrade("nodejs", ChocoOptions::default()).unwrap();
    assert_eq!(cmd, "choco upgrade nodejs");
}

#[test]
fn build_choco_upgrade_with_force() {
    let opts = ChocoOptions {
        force: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_upgrade("nodejs", opts).unwrap();
    assert!(cmd.contains("--force"));
}

#[test]
fn build_choco_upgrade_with_pre() {
    let opts = ChocoOptions {
        pre: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_upgrade("nodejs", opts).unwrap();
    assert!(cmd.contains("--pre"));
}

#[test]
fn build_scoop_upgrade_basic() {
    let cmd = command_builder::build_scoop_upgrade("nodejs", ScoopOptions::default()).unwrap();
    assert_eq!(cmd, "scoop update nodejs");
}

#[test]
fn build_scoop_upgrade_global() {
    let opts = ScoopOptions {
        global: true,
        ..Default::default()
    };
    let cmd = command_builder::build_scoop_upgrade("nodejs", opts).unwrap();
    assert!(cmd.contains("-g"));
}

#[test]
fn build_choco_uninstall_with_yes() {
    let opts = ChocoOptions {
        yes: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_uninstall("nodejs", opts).unwrap();
    assert_eq!(cmd, "choco uninstall nodejs -y");
}

#[test]
fn build_choco_uninstall_basic() {
    let cmd = command_builder::build_choco_uninstall("nodejs", ChocoOptions::default()).unwrap();
    assert_eq!(cmd, "choco uninstall nodejs");
}

#[test]
fn build_choco_uninstall_with_force() {
    let opts = ChocoOptions {
        force: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_uninstall("nodejs", opts).unwrap();
    assert!(cmd.contains("--force"));
}

#[test]
fn build_choco_uninstall_with_ignore_dependencies() {
    let opts = ChocoOptions {
        ignore_dependencies: true,
        ..Default::default()
    };
    let cmd = command_builder::build_choco_uninstall("nodejs", opts).unwrap();
    assert!(cmd.contains("--ignore-dependencies"));
}

#[test]
fn build_scoop_uninstall_basic() {
    let cmd = command_builder::build_scoop_uninstall("nodejs", ScoopOptions::default()).unwrap();
    assert_eq!(cmd, "scoop uninstall nodejs");
}

#[test]
fn build_scoop_uninstall_global() {
    let opts = ScoopOptions {
        global: true,
        ..Default::default()
    };
    let cmd = command_builder::build_scoop_uninstall("nodejs", opts).unwrap();
    assert!(cmd.contains("-g"));
}

#[test]
fn build_choco_search_includes_limit_output() {
    let cmd = command_builder::build_choco_search("7zip").unwrap();
    assert_eq!(cmd, "choco search 7zip --limit-output");
}

#[test]
fn build_scoop_info_returns_scoop_info_command() {
    let cmd = command_builder::build_scoop_info("7zip").unwrap();
    assert_eq!(cmd, "scoop info 7zip");
}

#[test]
fn build_scoop_bucket_add_with_name_only() {
    let cmd = command_builder::build_scoop_bucket_add("main", None).unwrap();
    assert_eq!(cmd, "scoop bucket add main");
}

#[test]
fn build_scoop_bucket_add_with_url() {
    let cmd = command_builder::build_scoop_bucket_add(
        "ntix",
        Some("https://github.com/Cristiandis/scoop-ntix"),
    )
    .unwrap();
    assert_eq!(
        cmd,
        "scoop bucket add ntix https://github.com/Cristiandis/scoop-ntix"
    );
}

#[test]
fn build_scoop_bucket_list_returns_command() {
    let cmd = command_builder::build_scoop_bucket_list();
    assert_eq!(cmd, "scoop bucket list");
}

#[test]
fn build_scoop_bucket_remove_returns_command() {
    let cmd = command_builder::build_scoop_bucket_remove("versions").unwrap();
    assert_eq!(cmd, "scoop bucket rm versions");
}

#[test]
fn validate_id_valid_ids_pass_through() {
    assert!(command_builder::validate_id("test").is_ok());
    assert!(command_builder::validate_id("my-package").is_ok());
    assert!(command_builder::validate_id("nodejs").is_ok());
    assert!(command_builder::validate_id("Package.Name").is_ok());
    assert!(command_builder::validate_id("some_pkg").is_ok());
}

#[test]
fn validate_id_empty_id_throws() {
    assert!(command_builder::validate_id("").is_err());
}

#[test]
fn validate_id_special_chars_throws() {
    assert!(command_builder::validate_id("pkg; rm -rf /").is_err());
}

#[test]
fn validate_id_pipe_chars_throws() {
    assert!(command_builder::validate_id("pkg|cat /etc/passwd").is_err());
}
