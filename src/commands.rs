use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use ntix_rs::config::config_loader;
use ntix_rs::execution::execution_engine;
use ntix_rs::lock::lock_file::LockFile;
use ntix_rs::models::diff_result::DiffResult;
use ntix_rs::models::options::ScoopBucket;
use ntix_rs::models::package_spec::PackageSpec;
use ntix_rs::models::{import_node::ImportNode, ntix_config::NTIXConfig};
use ntix_rs::state_management::state_service;
use ntix_rs::{diff, process_helper};
use std::path::PathBuf;
use std::time::Duration;

/// Returns `None` when a default config was just created and the caller
/// should exit early (the user still needs to edit it).
async fn resolve_and_compute(
    config_path: Option<PathBuf>,
    command_name: &str,
    adopt: bool,
    upgrade: bool,
    apply_config: bool,
) -> Result<Option<(NTIXConfig, PathBuf, DiffResult)>, Box<dyn std::error::Error>> {
    let is_new = config_path.is_none() && !config_loader::DEFAULT_CONFIG_PATH.is_file();
    let resolved_path = config_loader::ensure_default_config(config_path);

    if is_new {
        println!(
            "{}",
            format!("Created default config at {}", resolved_path.display()).green()
        );
        println!(
            "Edit it to add your packages, then run {} again.",
            format!("ntix {command_name}").bold()
        );
        return Ok(None);
    }

    let config = config_loader::load(resolved_path.clone())?;
    let state = state_service::load_state(None).unwrap_or_default();

    let config_file_name = resolved_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.yellow} {msg}")?);
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message(config_file_name.clone().bold().to_string());

    let mut diff: DiffResult = diff::diff_engine::compute_diff(
        &config, &state, None, None, None, adopt, upgrade, true, None, &spinner,
    )
    .await?;
    spinner.finish_and_clear();

    if apply_config {
        diff::diff_engine::compute_config_files_diff(&mut diff, &config, &state);
    }

    print_diff_tree(&config_file_name, &config, &diff, apply_config);

    for w in &diff.warnings {
        eprintln!("{}", format!("Warning: {}", w).yellow());
    }

    Ok(Some((config, resolved_path, diff)))
}

pub async fn apply(
    config_path: Option<PathBuf>,
    dry_run: bool,
    no_gc: bool,
    stop_on_failure: bool,
    adopt: bool,
    upgrade: bool,
    apply_config: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    if !process_helper::is_running_as_admin() {
        eprintln!(
            "{}",
            "Error: ntix apply requires administrator privileges.".red()
        );
        eprintln!("Please re-run in an elevated terminal (Run as Administrator).");
        return Ok(1);
    }

    let Some((config, _resolved_path, mut diff)) =
        resolve_and_compute(config_path, "apply", adopt, upgrade, apply_config).await?
    else {
        return Ok(0);
    };

    if no_gc {
        diff.to_remove.clear();
    }

    if dry_run {
        println!("\n{}", "(Dry run - no changes made)".yellow());
        return Ok(0);
    }

    if diff.is_empty() {
        return Ok(0);
    }

    let mut state = state_service::load_state(None).unwrap_or_default();
    let _lock = LockFile::new(None, true)?;
    let state_path = state_service::get_state_path()?;
    let success = execution_engine::apply_diff(
        &diff,
        &config.options,
        &mut state,
        &state_path,
        stop_on_failure,
        None,
        None,
        Some(&config),
        apply_config,
        Some(&|line: &str| println!("{line}")),
        Some(&|err: &str| eprintln!("{}", err.red())),
        None,
    )
    .await;

    if success {
        println!("\n{}", "Done.".green());
        Ok(0)
    } else {
        eprintln!("\n{}", "Some operations failed.".red());
        Ok(1)
    }
}

pub async fn diff_cmd(
    config_path: Option<PathBuf>,
    adopt: bool,
    upgrade: bool,
    apply_config: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    if resolve_and_compute(config_path, "diff", adopt, upgrade, apply_config)
        .await?
        .is_none()
    {
        return Ok(0);
    }
    Ok(0)
}

pub fn state_cmd() -> Result<i32, Box<dyn std::error::Error>> {
    let Some(state) = state_service::load_state(None) else {
        println!("{}", "No state file found.".yellow());
        return Ok(0);
    };

    println!("{}", "NTIX State:".bold());

    if state.winget.is_empty() && state.chocolatey.is_empty() && state.scoop.is_empty() {
        println!("  {}", "(empty)".dimmed());
    } else {
        for (id, ver) in &state.winget {
            println!("  {}", format!("winget: {id} ({ver})").cyan());
        }
        for (id, ver) in &state.chocolatey {
            println!("  {}", format!("chocolatey: {id} ({ver})").magenta());
        }
        for (id, ver) in &state.scoop {
            println!("  {}", format!("scoop: {id} ({ver})").blue());
        }
    }

    if !state.config_files.is_empty() {
        println!("  {}", "config files:".bold());
        for (dest, hash) in &state.config_files {
            println!("    {}", format!("{dest} ({hash:.8})").white());
        }
    }

    Ok(0)
}

fn source_color(source: &str, text: &str) -> colored::ColoredString {
    match source.to_lowercase().as_str() {
        "winget" => text.truecolor(139, 0, 139),
        "chocolatey" => text.blue(),
        "scoop" => text.magenta(),
        _ => text.normal(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    Imports,
    ToInstall,
    ToUpgrade,
    ToAdopt,
    ToSkip,
    BucketsToAdd,
    BucketsToRemove,
    ToRemove,
    ConfigFiles,
}

fn print_diff_tree(
    config_file_name: &str,
    config: &NTIXConfig,
    diff: &DiffResult,
    apply_config: bool,
) {
    let sections = [
        (Section::Imports, !config.imports.is_empty()),
        (Section::ToInstall, !diff.to_install.is_empty()),
        (Section::ToUpgrade, !diff.to_upgrade.is_empty()),
        (Section::ToAdopt, !diff.to_adopt.is_empty()),
        (Section::ToSkip, !diff.to_skip.is_empty()),
        (Section::BucketsToAdd, !diff.buckets_to_add.is_empty()),
        (Section::BucketsToRemove, !diff.buckets_to_remove.is_empty()),
        (Section::ToRemove, !diff.to_remove.is_empty()),
        (
            Section::ConfigFiles,
            apply_config
                && (!diff.config_files_to_create.is_empty()
                    || !diff.config_files_to_update.is_empty()
                    || !diff.config_files_no_longer_managed.is_empty()),
        ),
    ];
    let last_present = sections.iter().rposition(|(_, present)| *present);

    println!("{}", config_file_name.bold());

    for (i, (section, present)) in sections.iter().enumerate() {
        if *present {
            render_section(config, diff, *section, Some(i) == last_present);
        }
    }

    if diff.is_empty() {
        println!("{}", "Nothing to do.".dimmed());
    }
}

fn render_section(config: &NTIXConfig, diff: &DiffResult, section: Section, is_last: bool) {
    match section {
        Section::Imports => {
            println!(
                "{}",
                tree_branch("", is_last) + &"imports".dimmed().to_string()
            );
            print_import_children(&config.imports, &tree_continuation("", is_last));
        }
        Section::ToInstall => {
            println!(
                "{}",
                tree_branch("", is_last)
                    + &format!("\u{2191} To install ({})", diff.to_install.len())
                        .green()
                        .to_string()
            );
            print_grouped(
                &diff.to_install,
                VersionStyle::Paren,
                &tree_continuation("", is_last),
            );
        }
        Section::ToUpgrade => {
            println!(
                "{}",
                tree_branch("", is_last)
                    + &format!("\u{2191} To upgrade ({})", diff.to_upgrade.len())
                        .yellow()
                        .to_string()
            );
            print_grouped(
                &diff.to_upgrade,
                VersionStyle::Arrow,
                &tree_continuation("", is_last),
            );
        }
        Section::ToAdopt => {
            println!(
                "{}",
                tree_branch("", is_last)
                    + &format!("\u{271a} To adopt ({})", diff.to_adopt.len())
                        .cyan()
                        .to_string()
            );
            print_grouped(
                &diff.to_adopt,
                VersionStyle::Paren,
                &tree_continuation("", is_last),
            );
        }
        Section::ToSkip => {
            println!(
                "{}",
                tree_branch("", is_last)
                    + &format!("\u{2713} Already managed ({})", diff.to_skip.len())
                        .dimmed()
                        .to_string()
            );
        }
        Section::BucketsToAdd => {
            println!(
                "{}",
                tree_branch("", is_last)
                    + &format!("\u{2191} Buckets to add ({})", diff.buckets_to_add.len())
                        .green()
                        .to_string()
            );
            print_buckets(&diff.buckets_to_add, is_last);
        }
        Section::BucketsToRemove => {
            println!(
                "{}",
                tree_branch("", is_last)
                    + &format!(
                        "\u{2193} Buckets to remove ({})",
                        diff.buckets_to_remove.len()
                    )
                    .red()
                    .to_string()
            );
            print_buckets(&diff.buckets_to_remove, is_last);
        }
        Section::ToRemove => {
            println!(
                "{}",
                tree_branch("", is_last)
                    + &format!("\u{2717} Orphans ({})", diff.to_remove.len())
                        .red()
                        .to_string()
            );
            print_grouped(
                &diff.to_remove,
                VersionStyle::None,
                &tree_continuation("", is_last),
            );
        }
        Section::ConfigFiles => {
            let total = diff.config_files_to_create.len()
                + diff.config_files_to_update.len()
                + diff.config_files_no_longer_managed.len();
            println!(
                "{}",
                tree_branch("", is_last)
                    + &format!("\u{2691} Config files ({total})")
                        .white()
                        .to_string()
            );
            let prefix = tree_continuation("", is_last);
            for entry in &diff.config_files_to_create {
                println!(
                    "{}",
                    prefix.clone()
                        + "["
                        + "new".green().to_string().as_str()
                        + "] "
                        + &entry.dest.display().to_string()
                );
            }
            for entry in &diff.config_files_to_update {
                println!(
                    "{}",
                    prefix.clone()
                        + "["
                        + "update".yellow().to_string().as_str()
                        + "] "
                        + &entry.dest.display().to_string()
                );
            }
            for dest in &diff.config_files_no_longer_managed {
                println!(
                    "{}{}{}",
                    prefix.clone(),
                    "[orphan]".red(),
                    " ".to_string() + dest.as_str()
                );
            }
        }
    }
}

fn print_buckets(buckets: &[ScoopBucket], is_last: bool) {
    let mut buckets: Vec<_> = buckets.iter().collect();
    buckets.sort_by(|a, b| a.name.cmp(&b.name));
    for (j, bucket) in buckets.iter().enumerate() {
        println!(
            "{}{}",
            tree_branch(&tree_continuation("", is_last), j + 1 == buckets.len()),
            bucket.name.magenta()
        );
    }
}

fn tree_branch(prefix: &str, is_last: bool) -> String {
    format!(
        "{}{}",
        prefix,
        if is_last {
            "\u{2514}\u{2500}\u{2500} "
        } else {
            "\u{251c}\u{2500}\u{2500} "
        }
    )
}

fn tree_continuation(prefix: &str, is_last: bool) -> String {
    format!("{}{}", prefix, if is_last { "    " } else { "\u{2502}   " })
}

fn print_import_children(imports: &[ImportNode], prefix: &str) {
    let count = imports.len();
    for (i, import) in imports.iter().enumerate() {
        println!(
            "{}{}",
            tree_branch(prefix, i + 1 == count),
            import.path.display().to_string().dimmed()
        );
        if !import.children.is_empty() {
            print_import_children(&import.children, &tree_continuation(prefix, i + 1 == count));
        }
    }
}

#[derive(Clone, Copy)]
enum VersionStyle {
    None,
    Paren,
    Arrow,
}

fn print_grouped(packages: &[PackageSpec], style: VersionStyle, prefix: &str) {
    let mut sources: Vec<&str> = packages.iter().map(|p| p.source.as_str()).collect();
    sources.sort();
    sources.dedup();

    let count = sources.len();
    for (i, source) in sources.iter().enumerate() {
        let mut group: Vec<&PackageSpec> =
            packages.iter().filter(|p| p.source == *source).collect();
        group.sort_by(|a, b| a.id.cmp(&b.id));
        let source_last = i + 1 == count;
        let child_prefix = tree_continuation(prefix, source_last);

        if group.len() == 1 {
            let pkg = group[0];
            println!(
                "{}{}",
                tree_branch(prefix, source_last),
                source_color(
                    source,
                    &format!("{source}: {}{}", pkg.id, version_suffix(pkg, style))
                )
            );
        } else {
            println!(
                "{}{}",
                tree_branch(prefix, source_last),
                source_color(source, &format!("{source} ({})", group.len()))
            );
            let pkg_count = group.len();
            for (j, pkg) in group.iter().enumerate() {
                println!(
                    "{}{}",
                    tree_branch(&child_prefix, j + 1 == pkg_count),
                    source_color(source, &format!("{}{}", pkg.id, version_suffix(pkg, style)))
                );
            }
        }
    }
}

fn version_suffix(pkg: &PackageSpec, style: VersionStyle) -> String {
    let version = match pkg.version.as_deref() {
        Some(v) => v,
        None => return String::new(),
    };
    match style {
        VersionStyle::None => String::new(),
        VersionStyle::Paren => format!(" ({version})"),
        VersionStyle::Arrow => format!(" \u{2192} {version}"),
    }
}
