use std::{
    cell::RefCell,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    rc::Rc,
    sync::LazyLock,
};

use mlua::{Lua, MultiValue, Table, Value};

use crate::models::{
    config_file::ConfigFileEntry,
    import_node::{ImportNode, ImportNodeBuilder, ImportNodeBuilderRef},
    ntix_config::NTIXConfig,
    options::{ChocoOptions, NTIXOptions, ScoopBucket, ScoopOptions, WingetOptions},
    package_entry::PackageEntry,
};

const PACKAGE_LIST_KEYS: [&str; 3] = ["winget", "chocolatey", "scoop"];

static DEFAULT_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join("ntix")
});

pub static DEFAULT_CONFIG_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| DEFAULT_CONFIG_DIR.join("config.lua"));

const DEFAULT_CONFIG_CONTENT: &str = r#"
return {
    -- Don't declare the same package under multiple managers.
    -- This causes conflicts during install/uninstall
    options = {},
    pkgs = {}
}
"#;

pub fn ensure_default_config(config_path: Option<PathBuf>) -> PathBuf {
    match config_path {
        Some(path) => path,
        None => {
            let default = DEFAULT_CONFIG_PATH.clone();
            if !default.is_file() {
                if let Some(dir) = default.parent() {
                    fs::create_dir_all(dir).expect("Error creating default config dir.");
                }
                fs::write(&default, DEFAULT_CONFIG_CONTENT)
                    .expect("Error writing default config file");
            }
            default
        }
    }
}

pub fn load(config_path: PathBuf) -> Result<NTIXConfig, Box<dyn Error>> {
    if !config_path.is_file() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Config file not found: {}", config_path.display()),
        )));
    }

    let script = fs::read_to_string(&config_path)?;
    load_from_string(&script, config_path)
}

pub fn load_from_string(
    lua_script: &str,
    config_path: PathBuf,
) -> Result<NTIXConfig, Box<dyn Error>> {
    let state = Lua::new();

    let full_config_path = fs::canonicalize(&config_path)?;
    let root_dir = full_config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();

    if !root_dir.as_os_str().is_empty() {
        let pkg: mlua::Table = state.globals().get("package")?;
        let current_path: String = pkg.get("path")?;
        let root_dir_str = root_dir.to_string_lossy().replace('\\', "/");
        pkg.set("path", format!("{};{}/?.lua", current_path, root_dir_str))?;
    }

    let global_options = state.create_table()?;
    let global_pkgs = state.create_table()?;
    let global_config_files = state.create_table()?;
    for key in PACKAGE_LIST_KEYS {
        global_pkgs.set(key, state.create_table()?)?;
    }
    state.globals().set("options", &global_options)?;
    state.globals().set("pkgs", &global_pkgs)?;
    state.globals().set("configFiles", &global_config_files)?;

    let import_root = ImportNodeBuilder::new(&config_path);
    register_import_function(
        &state,
        &full_config_path,
        &global_options,
        &global_pkgs,
        &global_config_files,
        Rc::clone(&import_root),
    )?;

    let result = state.load(lua_script).eval::<MultiValue>();

    let results = result.map_err(|e| -> Box<dyn Error> {
        match &e {
            mlua::Error::SyntaxError { .. } => format!("Lua syntax error: {}", e).into(),
            mlua::Error::RuntimeError(_) => format!("Lua runtime error: {}", e).into(),
            _ => e.into(),
        }
    })?;

    let first_result = results
        .into_iter()
        .next()
        .ok_or_else(|| -> Box<dyn Error> { "Lua script returned no value".into() })?;

    if let Value::Table(root_table) = &first_result {
        let had_options = root_table.contains_key("options")?;
        let had_pkgs = root_table.contains_key("pkgs")?;
        merge_returned_table(
            &state,
            &global_options,
            &global_pkgs,
            &global_config_files,
            root_table,
        )?;
        if had_options {
            root_table.set("options", &global_options)?;
        }
        if had_pkgs {
            root_table.set("pkgs", &global_pkgs)?;
        }
        root_table.set("configFiles", &global_config_files)?;
    }

    let config = parse_config(first_result, config_path.clone())?;

    let children: Vec<ImportNode> = import_root
        .borrow()
        .children
        .iter()
        .map(ImportNodeBuilder::to_owned_tree)
        .collect();

    Ok(NTIXConfig {
        imports: children,
        ..config
    })
}

fn register_import_function(
    lua: &Lua,
    root_config_path: &Path,
    global_options: &Table,
    global_pkgs: &Table,
    global_config_files: &Table,
    import_root: ImportNodeBuilderRef,
) -> mlua::Result<()> {
    let directory_stack: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(vec![
        root_config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default(),
    ]));
    let node_stack: Rc<RefCell<Vec<ImportNodeBuilderRef>>> =
        Rc::new(RefCell::new(vec![Rc::clone(&import_root)]));

    let root_config_path_c = root_config_path.to_path_buf();
    let global_options_c = global_options.clone();
    let global_pkgs_c = global_pkgs.clone();
    let global_config_files_c = global_config_files.clone();
    let directory_stack_c = Rc::clone(&directory_stack);
    let node_stack_c = Rc::clone(&node_stack);

    let import_fn = lua.create_function(move |lua, arg: Value| -> mlua::Result<()> {
        let mut paths: Vec<String> = Vec::new();

        match &arg {
            Value::String(s) => paths.push(s.to_str()?.to_string()),
            Value::Table(t) => {
                for pair in t.clone().pairs::<Value, Value>() {
                    let (key, value) = pair?;
                    if !matches!(key, Value::Integer(_)) {
                        continue;
                    }
                    if let Value::String(s) = value {
                        let p = s.to_str()?.to_string();
                        if !p.trim().is_empty() {
                            paths.push(p);
                        }
                    }
                }
            }
            _ => {
                return Err(mlua::Error::RuntimeError(
                    "import() expects a file path string or an array of file path strings".into(),
                ));
            }
        }

        let current_dir = directory_stack_c
            .borrow()
            .last()
            .cloned()
            .unwrap_or_default();

        for relative_path in paths {
            let import_path = std::path::absolute(current_dir.join(&relative_path))
                .map_err(mlua::Error::external)?;

            if !import_path.is_file() {
                return Err(mlua::Error::RuntimeError(format!(
                    "Import file not found: {} (referenced from config)",
                    import_path.display()
                )));
            }

            let root_dir = root_config_path_c
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default();
            let relative_to_root = pathdiff::diff_paths(&import_path, &root_dir)
                .unwrap_or_else(|| import_path.clone());

            let child_node = ImportNodeBuilder::new(relative_to_root);

            node_stack_c
                .borrow()
                .last()
                .expect("node stack should never be empty")
                .borrow_mut()
                .children
                .push(Rc::clone(&child_node));

            // Let the imported file require() siblings from its own directory.
            let import_dir = import_path
                .parent()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            let pkg: Table = lua.globals().get("package")?;
            let current_path: String = pkg.get("path")?;
            pkg.set("path", format!("{};{}/?.lua", current_path, import_dir))?;

            let script = std::fs::read_to_string(&import_path).map_err(mlua::Error::external)?;

            directory_stack_c.borrow_mut().push(
                import_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_default(),
            );
            node_stack_c.borrow_mut().push(Rc::clone(&child_node));

            // Nested import() calls made while this script runs merge into
            // global_options/global_pkgs as their own side effect.
            let result = lua.load(&script).eval::<MultiValue>();

            directory_stack_c.borrow_mut().pop();
            node_stack_c.borrow_mut().pop();

            let import_results = result?;

            if let Some(Value::Table(returned)) = import_results.into_iter().next() {
                merge_returned_table(
                    lua,
                    &global_options_c,
                    &global_pkgs_c,
                    &global_config_files_c,
                    &returned,
                )?;
            }
        }

        Ok(())
    })?;

    lua.globals().set("import", import_fn)
}

fn merge_returned_table(
    lua: &Lua,
    global_options: &Table,
    global_pkgs: &Table,
    global_config_files: &Table,
    returned: &Table,
) -> mlua::Result<()> {
    if let Ok(Value::Table(returned_options)) = returned.get::<Value>("options") {
        deep_merge_table(global_options, &returned_options)?;
    }

    if let Ok(Value::Table(returned_pkgs)) = returned.get::<Value>("pkgs") {
        for key in PACKAGE_LIST_KEYS {
            if let Ok(Value::Table(sub_table)) = returned_pkgs.get::<Value>(key) {
                let target = match global_pkgs.get::<Value>(key)? {
                    Value::Table(t) => t,
                    _ => {
                        let t = lua.create_table()?;
                        global_pkgs.set(key, t.clone())?;
                        t
                    }
                };
                merge_packages_deduped(&target, &sub_table)?;
            }
        }
    }

    if let Ok(Value::Table(returned_config_files)) = returned.get::<Value>("configFiles") {
        deep_merge_table(global_config_files, &returned_config_files)?;
    }

    Ok(())
}

/// Recursively merges `source` into `target`.
/// Nested tables are merged key-by-key; scalars overwrite.
fn deep_merge_table(target: &Table, source: &Table) -> mlua::Result<()> {
    for pair in source.clone().pairs::<Value, Value>() {
        let (key, value) = pair?;

        if let Value::Table(source_sub) = &value
            && let Ok(Value::Table(target_sub)) = target.get::<Value>(key.clone())
        {
            deep_merge_table(&target_sub, source_sub)?;
            continue;
        }

        target.set(key, value)?;
    }

    Ok(())
}

/// Merges package arrays into target, deduplicating by id (later wins).
fn merge_packages_deduped(target: &Table, source: &Table) -> mlua::Result<()> {
    let mut id_to_index: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut next_index: i64 = 1;

    for pair in target.clone().pairs::<Value, Value>() {
        let (key, value) = pair?;
        let idx = match key {
            Value::Integer(i) => i,
            _ => continue,
        };
        next_index = next_index.max(idx + 1);

        if let Some(existing_id) = extract_id(&value)? {
            id_to_index.insert(existing_id, idx);
        }
    }

    for pair in source.clone().pairs::<Value, Value>() {
        let (key, value) = pair?;
        if !matches!(key, Value::Integer(_)) {
            continue;
        }

        let Some(id) = extract_id(&value)? else {
            continue;
        };

        if let Some(&existing_index) = id_to_index.get(&id) {
            target.set(existing_index, value)?;
        } else {
            target.set(next_index, value.clone())?;
            id_to_index.insert(id, next_index);
            next_index += 1;
        }
    }

    Ok(())
}

fn extract_id(entry: &Value) -> mlua::Result<Option<String>> {
    match entry {
        Value::String(s) => Ok(Some(s.to_str()?.to_string())),
        Value::Table(t) => match t.get::<Value>("id")? {
            Value::String(s) => Ok(Some(s.to_str()?.to_string())),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

fn parse_config(result: mlua::Value, config_path: PathBuf) -> Result<NTIXConfig, Box<dyn Error>> {
    let table = match result {
        Value::Table(t) => t,
        other => {
            return Err(format!(
                "Config script must return a table (got {}): {}",
                other.type_name(),
                config_path.display()
            )
            .into());
        }
    };

    let Value::Table(_options) = table.get::<Value>("options")? else {
        return Err("Config error: missing top-level 'options' table".into());
    };

    let Value::Table(_pkgs) = table.get::<Value>("pkgs")? else {
        return Err("Config error: missing top-level 'pkgs' table".into());
    };

    let options = read_options(&table.get("options")?)?;
    let pkgs: Table = table.get("pkgs")?;

    let config_files = match table.get::<Value>("configFiles")? {
        Value::Table(t) => read_config_files(&t, &config_path)?,
        Value::Nil => Vec::new(),
        other => {
            return Err(format!(
                "Config error: 'configFiles' must be a table keyed by destination path (got {}): {}",
                other.type_name(),
                config_path.display()
            )
            .into());
        }
    };

    Ok(NTIXConfig {
        options,
        winget_packages: read_package_list(&pkgs, "winget")?,
        choco_packages: read_package_list(&pkgs, "chocolatey")?,
        scoop_packages: read_package_list(&pkgs, "scoop")?,
        config_files,
        imports: Vec::new(),
    })
}

fn read_config_files(table: &Table, config_path: &Path) -> mlua::Result<Vec<ConfigFileEntry>> {
    let root_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();

    let mut entries = Vec::new();

    for pair in table.clone().pairs::<Value, Value>() {
        let (key, value) = pair?;

        let dest_str = match key {
            Value::String(s) => s.to_str()?.to_string(),
            _ => continue,
        };

        let dest = PathBuf::from(&dest_str);
        if !dest.is_absolute() {
            return Err(mlua::Error::RuntimeError(format!(
                "Config error: configFiles destination must be an absolute path: {}",
                dest_str
            )));
        }

        let src_str = match value {
            Value::String(s) => s.to_str()?.to_string(),
            _ => {
                return Err(mlua::Error::RuntimeError(format!(
                    "Config error: configFiles entry for '{}' must be a source path string",
                    dest_str
                )));
            }
        };

        let mut src = PathBuf::from(&src_str);
        if !src.is_absolute() {
            src = root_dir.join(&src);
        }

        if !src.is_file() {
            return Err(mlua::Error::RuntimeError(format!(
                "Config error: source file not found for configFiles entry '{}': {}",
                dest_str,
                src.display()
            )));
        }

        entries.push(ConfigFileEntry { dest, src });
    }

    Ok(entries)
}

fn read_options(options: &Table) -> mlua::Result<NTIXOptions> {
    let mut winget = WingetOptions::default();
    if let Value::Table(t) = options.get::<Value>("winget")? {
        winget = WingetOptions {
            enable: read_bool(&t.get::<Value>("enable")?, winget.enable),
            accept_agreement: read_bool(
                &t.get::<Value>("acceptAgreements")?,
                winget.accept_agreement,
            ),
            silent: read_bool(&t.get::<Value>("silent")?, winget.silent),
            disable_interactivity: read_bool(
                &t.get::<Value>("disableInteractivity")?,
                winget.disable_interactivity,
            ),
        };
    }

    let mut choco = ChocoOptions::default();
    if let Value::Table(t) = options.get::<Value>("chocolatey")? {
        choco = ChocoOptions {
            enable: read_bool(&t.get::<Value>("enable")?, choco.enable),
            yes: read_bool(&t.get::<Value>("yes")?, choco.yes),
            force: read_bool(&t.get::<Value>("force")?, choco.force),
            ignore_dependencies: read_bool(
                &t.get::<Value>("ignoreDependencies")?,
                choco.ignore_dependencies,
            ),
            allow_downgrade: read_bool(&t.get::<Value>("allowDowngrade")?, choco.allow_downgrade),
            skip_power_shell: read_bool(&t.get::<Value>("skipPowerShell")?, choco.skip_power_shell),
            params: read_string(&t.get::<Value>("params")?, choco.params.clone()).unwrap(),
            pre: read_bool(&t.get::<Value>("pre")?, choco.pre),
        };
    }

    let mut scoop = ScoopOptions::default();
    if let Value::Table(t) = options.get::<Value>("scoop")? {
        let mut buckets = scoop.buckets.clone();
        if let Value::Table(bucket_table) = t.get::<Value>("buckets")? {
            buckets = Vec::new();
            for pair in bucket_table.pairs::<Value, Value>() {
                let (key, value) = pair?;
                if !matches!(key, Value::Integer(_)) {
                    continue;
                }

                let bucket = match value {
                    Value::String(s) => ScoopBucket::new(s.to_str()?.to_string()),
                    Value::Table(bt) => {
                        let name: String = bt.get("name")?;
                        let url: Option<String> = match bt.get::<Value>("url")? {
                            Value::Nil => None,
                            Value::String(s) => Some(s.to_str()?.to_string()),
                            _ => None,
                        };
                        ScoopBucket { name, url }
                    }
                    Value::Nil => continue,
                    other => ScoopBucket::new(other.to_string()?),
                };

                buckets.push(bucket);
            }
        }

        scoop = ScoopOptions {
            enable: read_bool(&t.get::<Value>("enable")?, scoop.enable),
            buckets,
            global: read_bool(&t.get::<Value>("global")?, scoop.global),
            independent: read_bool(&t.get::<Value>("independent")?, scoop.independent),
            no_cache: read_bool(&t.get::<Value>("noCache")?, scoop.no_cache),
            skip_hash_check: read_bool(&t.get::<Value>("skipHashCheck")?, scoop.skip_hash_check),
            arch: read_string(&t.get::<Value>("arch")?, scoop.arch.clone()).unwrap(),
        };
    }

    Ok(NTIXOptions {
        winget,
        chocolatey: choco,
        scoop,
    })
}

fn read_bool(value: &Value, fallback: bool) -> bool {
    match value {
        Value::Boolean(b) => *b,
        _ => fallback,
    }
}

fn read_string(value: &Value, fallback: Option<String>) -> mlua::Result<Option<String>> {
    match value {
        Value::String(s) => Ok(Some(s.to_str()?.to_string())),
        _ => Ok(fallback),
    }
}

fn read_package_list(pkgs: &Table, key: &str) -> mlua::Result<Vec<PackageEntry>> {
    let mut list = Vec::new();

    let Value::Table(entries) = pkgs.get::<Value>(key)? else {
        return Ok(list);
    };

    for pair in entries.pairs::<Value, Value>() {
        let (key, value) = pair?;
        if !matches!(key, Value::Integer(_)) {
            continue;
        }

        match value {
            Value::String(s) => {
                list.push(PackageEntry::new(s.to_str()?.to_string()));
            }
            Value::Table(entry) => {
                let Value::String(id) = entry.get::<Value>("id")? else {
                    continue;
                };
                let version = match entry.get::<Value>("version")? {
                    Value::String(v) => Some(v.to_str()?.to_string()),
                    _ => None,
                };
                list.push(PackageEntry {
                    id: id.to_str()?.to_string(),
                    version,
                });
            }
            _ => {}
        }
    }

    Ok(list)
}
