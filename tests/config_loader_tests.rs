use std::fs;
use std::path::PathBuf;

use ntix_rs::config::config_loader;
use ntix_rs::config::config_loader::ensure_default_config;

/// Creates a temp dir (unique per call) and returns its path.
fn temp_dir() -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("ntix_test_{}_{}", std::process::id(), uuid_short()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn uuid_short() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
        ^ (std::process::id() as u64) << 32
}

/// Writes a placeholder `test.lua` into `dir` and returns its path.
fn placeholder_config_path(dir: &std::path::Path) -> PathBuf {
    let path = dir.join("test.lua");
    fs::write(&path, "return { options = {}, pkgs = {} }").unwrap();
    path
}

/// Escapes a filesystem path for safe embedding inside a Lua string literal.
fn lua_escape(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "\\\\")
}

#[test]
fn load_from_string_valid_config_returns_ntix_config() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = {
            winget = { enable = true, acceptAgreements = true, silent = true, disableInteractivity = true },
            chocolatey = { enable = true, yes = true },
            scoop = { enable = true, buckets = { "main", "extras" } }
        }
        pkgs = {
            winget = { "Microsoft.VisualStudioCode" },
            chocolatey = { "git" },
            scoop = { "fd" }
        }
        return { options = options, pkgs = pkgs }
    "#;

    let config = config_loader::load_from_string(lua, path).unwrap();
    assert!(config.options.winget.enable);
    assert!(config.options.chocolatey.enable);
    assert!(config.options.scoop.enable);
    assert_eq!(config.winget_packages.len(), 1);
    assert_eq!(config.choco_packages.len(), 1);
    assert_eq!(config.scoop_packages.len(), 1);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_missing_winget_flags_default_to_false() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = {
            winget = { enable = true, acceptAgreements = true },
            chocolatey = { enable = true, yes = true },
            scoop = { enable = true, buckets = { "main", "extras" } }
        }
        pkgs = {
            winget = { "Microsoft.VisualStudioCode" },
            chocolatey = { "git" },
            scoop = { "fd" }
        }
        return { options = options, pkgs = pkgs }
    "#;

    let config = config_loader::load_from_string(lua, path).unwrap();
    assert!(config.options.winget.enable);
    assert!(!config.options.winget.silent);
    assert!(!config.options.winget.disable_interactivity);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_missing_config_file_throws_file_not_found() {
    let dir = temp_dir();
    let err = config_loader::load(dir.join("nonexistent.lua")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Config file not found"), "got: {msg}");
    assert!(msg.contains("nonexistent.lua"), "got: {msg}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_missing_options_table_throws() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        pkgs = {
            winget = { "test" }
        }
        return { pkgs = pkgs }
    "#;

    let err = config_loader::load_from_string(lua, path).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("missing top-level 'options' table"),
        "got: {msg}"
    );
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_missing_pkgs_table_throws() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = {
            winget = { enable = true }
        }
        return { options = options }
    "#;

    let err = config_loader::load_from_string(lua, path).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("missing top-level 'pkgs' table"), "got: {msg}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_missing_winget_in_options_defaults_to_empty() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = {
            chocolatey = { enable = true }
        }
        pkgs = {
            winget = { "test" }
        }
        return { options = options, pkgs = pkgs }
    "#;

    let config = config_loader::load_from_string(lua, path).unwrap();
    assert!(!config.options.winget.enable);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_with_import_single_file_merges_options_and_packages() {
    let dir = temp_dir();
    let base_config = r#"
        return {
            options = {
                winget = { enable = true, acceptAgreements = true },
                chocolatey = { enable = true, yes = true }
            },
            pkgs = {
                winget = { "Microsoft.VisualStudioCode", "Git.Git" },
                chocolatey = { "git" }
            }
        }
    "#;
    fs::write(dir.join("base.lua"), base_config).unwrap();

    let main_config = r#"
        import("base.lua")
        return { options = options, pkgs = pkgs }
    "#;
    let main_path = dir.join("main.lua");
    fs::write(&main_path, "return {}").unwrap();

    let config = config_loader::load_from_string(main_config, main_path).unwrap();

    assert!(config.options.winget.enable);
    assert!(config.options.winget.accept_agreement);
    assert!(config.options.chocolatey.enable);
    assert!(config.options.chocolatey.yes);

    assert_eq!(config.winget_packages.len(), 2);
    let winget_ids: Vec<&str> = config
        .winget_packages
        .iter()
        .map(|p| p.id.as_str())
        .collect();
    assert!(winget_ids.contains(&"Microsoft.VisualStudioCode"));
    assert!(winget_ids.contains(&"Git.Git"));

    assert_eq!(config.choco_packages.len(), 1);
    assert_eq!(config.choco_packages[0].id, "git");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_with_import_and_inline_tables_merges_both() {
    let dir = temp_dir();
    let base_config = r#"
        return {
            options = {
                winget = { acceptAgreements = true }
            },
            pkgs = {
                winget = { "Microsoft.VisualStudioCode" }
            }
        }
    "#;
    fs::write(dir.join("base.lua"), base_config).unwrap();

    let main_config = r#"
        import("base.lua")
        return {
            options = {
                winget = { enable = true }
            },
            pkgs = {
                winget = { "Git.Git" }
            }
        }
    "#;
    let main_path = dir.join("main.lua");
    fs::write(&main_path, "return {}").unwrap();

    let config = config_loader::load_from_string(main_config, main_path).unwrap();

    assert!(config.options.winget.enable);
    assert!(config.options.winget.accept_agreement);

    let ids: Vec<&str> = config
        .winget_packages
        .iter()
        .map(|p| p.id.as_str())
        .collect();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"Microsoft.VisualStudioCode"));
    assert!(ids.contains(&"Git.Git"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_with_import_missing_file_throws() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        import("nonexistent.lua")
        return { options = options, pkgs = pkgs }
    "#;

    let err = config_loader::load_from_string(lua, path).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("nonexistent.lua"), "got: {msg}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_with_import_nested_imports_merge_correctly() {
    let dir = temp_dir();
    let nested = dir.join("nested");
    fs::create_dir_all(&nested).unwrap();

    let base_config = r#"
        return {
            options = { winget = { enable = true } },
            pkgs = { winget = { "Base.Package" } }
        }
    "#;
    fs::write(nested.join("base.lua"), base_config).unwrap();

    let ext_config = r#"
        import("base.lua")
        return {
            options = options,
            pkgs = { winget = { "Extended.Package" } }
        }
    "#;
    fs::write(nested.join("ext.lua"), ext_config).unwrap();

    let main_config = r#"
        import("nested/ext.lua")
        return { options = options, pkgs = pkgs }
    "#;
    let main_path = dir.join("main.lua");
    fs::write(&main_path, "return {}").unwrap();

    let config = config_loader::load_from_string(main_config, main_path).unwrap();
    assert!(config.options.winget.enable);

    assert_eq!(config.winget_packages.len(), 2);
    let ids: Vec<&str> = config
        .winget_packages
        .iter()
        .map(|p| p.id.as_str())
        .collect();
    assert!(ids.contains(&"Base.Package"));
    assert!(ids.contains(&"Extended.Package"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_with_import_package_deduplication_by_id() {
    let dir = temp_dir();
    let pkg1_config = r#"
        return {
            options = { winget = { enable = true } },
            pkgs = {
                winget = {
                    "Unique.Package",
                    { id = "Duplicate.Package", version = "1.0" }
                }
            }
        }
    "#;
    fs::write(dir.join("pkg1.lua"), pkg1_config).unwrap();

    let pkg2_config = r#"
        return {
            pkgs = {
                winget = {
                    { id = "Duplicate.Package", version = "2.0" },
                    "Another.Unique"
                }
            }
        }
    "#;
    fs::write(dir.join("pkg2.lua"), pkg2_config).unwrap();

    let main_config = r#"
        import({ "pkg1.lua", "pkg2.lua" })
        return { options = options, pkgs = pkgs }
    "#;
    let main_path = dir.join("main.lua");
    fs::write(&main_path, "return {}").unwrap();

    let config = config_loader::load_from_string(main_config, main_path).unwrap();

    assert_eq!(config.winget_packages.len(), 3);
    let ids: Vec<&str> = config
        .winget_packages
        .iter()
        .map(|p| p.id.as_str())
        .collect();
    assert!(ids.contains(&"Unique.Package"));
    assert!(ids.contains(&"Another.Unique"));

    let dup = config
        .winget_packages
        .iter()
        .find(|p| p.id == "Duplicate.Package")
        .unwrap();
    assert_eq!(dup.version, Some("2.0".to_string()));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_with_import_array_of_paths_imports_all() {
    let dir = temp_dir();
    let winget_config = r#"
        return {
            pkgs = { winget = { "Winget.Package" } }
        }
    "#;
    fs::write(dir.join("winget.lua"), winget_config).unwrap();

    let scoop_config = r#"
        return {
            pkgs = { scoop = { "Scoop.Package" } }
        }
    "#;
    fs::write(dir.join("scoop.lua"), scoop_config).unwrap();

    let main_config = r#"
        import({ "winget.lua", "scoop.lua" })
        return { options = options, pkgs = pkgs }
    "#;
    let main_path = dir.join("main.lua");
    fs::write(&main_path, "return {}").unwrap();

    let config = config_loader::load_from_string(main_config, main_path).unwrap();
    assert_eq!(config.winget_packages.len(), 1);
    assert_eq!(config.winget_packages[0].id, "Winget.Package");
    assert_eq!(config.scoop_packages.len(), 1);
    assert_eq!(config.scoop_packages[0].id, "Scoop.Package");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_with_import_deep_merge_options_preserves_nested_keys() {
    let dir = temp_dir();
    let base_config = r#"
        return {
            options = {
                winget = {
                    enable = true,
                    acceptAgreements = true,
                    disableInteractivity = false
                },
                scoop = {
                    enable = true,
                    buckets = { "main" }
                }
            },
            pkgs = { winget = {} }
        }
    "#;
    fs::write(dir.join("base.lua"), base_config).unwrap();

    let main_config = r#"
        import("base.lua")
        -- Modify global options table
        options.winget.disableInteractivity = true
        options.scoop.buckets = { "main", "extras" }
        return { options = options, pkgs = pkgs }
    "#;
    let main_path = dir.join("main.lua");
    fs::write(&main_path, "return {}").unwrap();

    let config = config_loader::load_from_string(main_config, main_path).unwrap();
    assert!(config.options.winget.enable);
    assert!(config.options.winget.accept_agreement);
    assert!(config.options.winget.disable_interactivity);
    assert!(config.options.scoop.enable);
    let bucket_names: Vec<&str> = config
        .options
        .scoop
        .buckets
        .iter()
        .map(|b| b.name.as_str())
        .collect();
    assert!(bucket_names.contains(&"main"));
    assert!(bucket_names.contains(&"extras"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_empty_pkgs_returns_empty_lists() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = { winget = { enable = true } }
        pkgs = {
            winget = {},
            chocolatey = {},
            scoop = {}
        }
        return { options = options, pkgs = pkgs }
    "#;

    let config = config_loader::load_from_string(lua, path).unwrap();
    assert!(config.winget_packages.is_empty());
    assert!(config.choco_packages.is_empty());
    assert!(config.scoop_packages.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_missing_pkgs_sub_keys_returns_empty_lists() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = { winget = { enable = true } }
        pkgs = {}
        return { options = options, pkgs = pkgs }
    "#;

    let config = config_loader::load_from_string(lua, path).unwrap();
    assert!(config.winget_packages.is_empty());
    assert!(config.choco_packages.is_empty());
    assert!(config.scoop_packages.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_malformed_lua_throws_syntax_error() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        return { options = , pkgs = {} }
    "#;

    let err = config_loader::load_from_string(lua, path).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Lua syntax error"), "got: {msg}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_runtime_error_throws_runtime_error() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        local x = nil
        x.foo()
    "#;

    let err = config_loader::load_from_string(lua, path).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Lua runtime error"), "got: {msg}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_package_table_without_id_skipped() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = { winget = { enable = true } }
        pkgs = {
            winget = {
                { version = "1.0" }
            }
        }
        return { options = options, pkgs = pkgs }
    "#;

    let config = config_loader::load_from_string(lua, path).unwrap();
    assert!(config.winget_packages.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_package_table_id_no_version_version_is_none() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = { winget = { enable = true } }
        pkgs = {
            winget = {
                { id = "test-pkg" }
            }
        }
        return { options = options, pkgs = pkgs }
    "#;

    let config = config_loader::load_from_string(lua, path).unwrap();
    assert_eq!(config.winget_packages.len(), 1);
    assert_eq!(config.winget_packages[0].id, "test-pkg");
    assert!(config.winget_packages[0].version.is_none());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_returns_non_table_throws() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        return 42
    "#;

    let err = config_loader::load_from_string(lua, path).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("must return a table"), "got: {msg}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_import_invalid_arg_throws() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        import(42)
        return { options = options, pkgs = pkgs }
    "#;

    let err = config_loader::load_from_string(lua, path).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("expects a file path string"), "got: {msg}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn ensure_default_config_explicit_missing_path_not_created() {
    let dir = temp_dir();
    let config_path = dir.join("config.lua");

    let result = ensure_default_config(Some(config_path.clone()));
    assert_eq!(result, config_path);
    // An explicit, non-existent path is returned untouched and is NOT
    // auto-created; loading it later should report "config not found".
    assert!(!config_path.exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn ensure_default_config_explicit_path_returns_path() {
    let dir = temp_dir();
    let config_path = dir.join("config.lua");
    let result = ensure_default_config(Some(config_path.clone()));
    assert_eq!(result, config_path);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn ensure_default_config_already_exists_does_not_overwrite() {
    let dir = temp_dir();
    let config_path = dir.join("config.lua");
    fs::write(&config_path, "custom content").unwrap();
    let result = ensure_default_config(Some(config_path.clone()));
    assert_eq!(result, config_path);
    assert_eq!(fs::read_to_string(&config_path).unwrap(), "custom content");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_scoop_bucket_table_form_parses_url() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = {
            scoop = {
                enable = true,
                buckets = {
                    { name = "main", url = "https://github.com/ScoopInstaller/Main" }
                }
            }
        }
        pkgs = {
            scoop = { "rg" }
        }
        return { options = options, pkgs = pkgs }
    "#;

    let config = config_loader::load_from_string(lua, path).unwrap();
    assert_eq!(config.options.scoop.buckets.len(), 1);
    assert_eq!(config.options.scoop.buckets[0].name, "main");
    assert_eq!(
        config.options.scoop.buckets[0].url,
        Some("https://github.com/ScoopInstaller/Main".to_string())
    );
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_scoop_options_all_flags() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = {
            scoop = {
                enable = true,
                global = true,
                independent = true,
                noCache = true,
                skipHashCheck = true,
                arch = "64bit"
            }
        }
        pkgs = { scoop = { "rg" } }
        return { options = options, pkgs = pkgs }
    "#;

    let config = config_loader::load_from_string(lua, path).unwrap();
    assert!(config.options.scoop.enable);
    assert!(config.options.scoop.global);
    assert!(config.options.scoop.independent);
    assert!(config.options.scoop.no_cache);
    assert!(config.options.scoop.skip_hash_check);
    assert_eq!(config.options.scoop.arch, Some("64bit".to_string()));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_string_choco_options_all_flags() {
    let dir = temp_dir();
    let path = placeholder_config_path(&dir);
    let lua = r#"
        options = {
            chocolatey = {
                enable = true,
                yes = true,
                force = true,
                ignoreDependencies = true,
                allowDowngrade = true,
                skipPowerShell = true,
                params = "/params",
                pre = true
            }
        }
        pkgs = { chocolatey = { "git" } }
        return { options = options, pkgs = pkgs }
    "#;

    let config = config_loader::load_from_string(lua, path).unwrap();
    assert!(config.options.chocolatey.enable);
    assert!(config.options.chocolatey.yes);
    assert!(config.options.chocolatey.force);
    assert!(config.options.chocolatey.ignore_dependencies);
    assert!(config.options.chocolatey.allow_downgrade);
    assert!(config.options.chocolatey.skip_power_shell);
    assert_eq!(
        config.options.chocolatey.params,
        Some("/params".to_string())
    );
    assert!(config.options.chocolatey.pre);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_config_files_parses_dest_keyed_entries() {
    let dir = temp_dir();
    let src = dir.join("conf.d");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("kitty.conf"), "font_size 11").unwrap();

    let dest = src
        .parent()
        .unwrap()
        .join("dest")
        .join("kitty.conf")
        .to_string_lossy()
        .to_string();

    let lua = format!(
        r#"
        options = {{}}
        pkgs = {{}}
        configFiles = {{ ["{dest}"] = "conf.d/kitty.conf" }}
        return {{ options = options, pkgs = pkgs, configFiles = configFiles }}
    "#,
        dest = lua_escape(&std::path::Path::new(&dest))
    );
    let path = placeholder_config_path(&dir);
    let config = config_loader::load_from_string(&lua, path).unwrap();
    assert_eq!(config.config_files.len(), 1);
    let entry = &config.config_files[0];
    assert_eq!(entry.dest, std::path::PathBuf::from(&dest));
    assert_eq!(entry.src, src.join("kitty.conf"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_config_files_absolute_src_is_used_as_is() {
    let dir = temp_dir();
    let abs_src = dir.join("abs.conf");
    fs::write(&abs_src, "x=1").unwrap();

    let dest = dir
        .join("dest")
        .join("abs.conf")
        .to_string_lossy()
        .to_string();
    let lua = format!(
        r#"
        options = {{}}
        pkgs = {{}}
        configFiles = {{ ["{dest}"] = "{src}" }}
        return {{ options = options, pkgs = pkgs, configFiles = configFiles }}
    "#,
        dest = lua_escape(&std::path::Path::new(&dest)),
        src = lua_escape(&abs_src)
    );
    let path = placeholder_config_path(&dir);
    let config = config_loader::load_from_string(&lua, path).unwrap();
    assert_eq!(config.config_files.len(), 1);
    assert_eq!(config.config_files[0].src, abs_src);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_config_files_relative_dest_errors() {
    let dir = temp_dir();
    let src = dir.join("x.conf");
    fs::write(&src, "x=1").unwrap();

    let lua = r#"
        options = {}
        pkgs = {}
        configFiles = { ["relative/path.conf"] = "x.conf" }
        return { options = options, pkgs = pkgs, configFiles = configFiles }
    "#;
    let path = placeholder_config_path(&dir);
    let err = config_loader::load_from_string(lua, path).unwrap_err();
    assert!(
        format!("{err}").contains("absolute"),
        "expected absolute-path error, got: {err}"
    );
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_config_files_missing_src_errors() {
    let dir = temp_dir();
    let dest = dir
        .join("dest")
        .join("nope.conf")
        .to_string_lossy()
        .to_string();
    let lua = format!(
        r#"
        options = {{}}
        pkgs = {{}}
        configFiles = {{ ["{dest}"] = "does_not_exist.conf" }}
        return {{ options = options, pkgs = pkgs, configFiles = configFiles }}
    "#,
        dest = lua_escape(&std::path::Path::new(&dest))
    );
    let path = placeholder_config_path(&dir);
    let err = config_loader::load_from_string(&lua, path).unwrap_err();
    assert!(
        format!("{err}").contains("not found"),
        "expected missing source error, got: {err}"
    );
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_config_files_from_import_merges() {
    let dir = temp_dir();
    let base_dir = dir.join("base");
    fs::create_dir_all(&base_dir).unwrap();
    let base_src = base_dir.join("base.conf");
    fs::write(&base_src, "base=1").unwrap();
    let base_dest = dir
        .join("dest")
        .join("base.conf")
        .to_string_lossy()
        .to_string();

    fs::write(
        base_dir.join("base.lua"),
        format!(
            r#"
            options = options
            pkgs = {{}}
            configFiles = {{ ["{dest}"] = "{src}" }}
            return {{ options = options, pkgs = pkgs, configFiles = configFiles }}
        "#,
            dest = lua_escape(&std::path::Path::new(&base_dest)),
            src = lua_escape(&base_src)
        ),
    )
    .unwrap();

    let lua = r#"
        options = {}
        pkgs = {}
        import("base/base.lua")
        return { options = options, pkgs = pkgs, configFiles = configFiles }
    "#;
    let path = placeholder_config_path(&dir);
    let config = config_loader::load_from_string(lua, path).unwrap();
    assert_eq!(config.config_files.len(), 1);
    assert_eq!(
        config.config_files[0].dest,
        std::path::PathBuf::from(base_dest)
    );
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_config_files_src_not_string_errors() {
    let dir = temp_dir();
    let dest = dir
        .join("dest")
        .join("x.conf")
        .to_string_lossy()
        .to_string();
    let lua = format!(
        r#"
        options = {{}}
        pkgs = {{}}
        configFiles = {{ ["{dest}"] = 123 }}
        return {{ options = options, pkgs = pkgs, configFiles = configFiles }}
    "#,
        dest = lua_escape(&std::path::Path::new(&dest))
    );
    let path = placeholder_config_path(&dir);
    let err = config_loader::load_from_string(&lua, path).unwrap_err();
    assert!(
        format!("{err}").contains("must be a source path string"),
        "expected source-must-be-string error, got: {err}"
    );
    fs::remove_dir_all(&dir).unwrap();
}
