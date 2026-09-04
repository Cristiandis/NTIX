mod commands;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply desired state (install/remove packages)
    Apply {
        /// Path to configuration file (default: ~/ntix/config.lua)
        config_path: Option<PathBuf>,

        /// Show what would change without applying
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// Don't remove packages not in config
        #[arg(long = "no-gc")]
        no_gc: bool,

        /// Stop on first package failure instead of continuing
        #[arg(long = "stop-on-failure")]
        stop_on_failure: bool,

        /// Adopt already-installed packages into NTIX state
        #[arg(short = 'a', long = "adopt")]
        adopt: bool,

        /// Check for and apply available upgrades
        #[arg(short = 'u', long = "upgrade")]
        upgrade: bool,

        /// Manage arbitrary config files declared in the configFiles table
        #[arg(short = 'c', long = "apply-configs")]
        apply_config: bool,
    },

    /// Show what would change
    Diff {
        /// Path to configuration file (default: ~/ntix/config.lua)
        config_path: Option<PathBuf>,

        /// Show packages that would be adopted
        #[arg(short = 'a', long = "adopt")]
        adopt: bool,

        /// Check for and apply available upgrades
        #[arg(short = 'u', long = "upgrade")]
        upgrade: bool,

        /// Show config files declared in the configFiles table
        #[arg(short = 'c', long = "apply-configs")]
        apply_config: bool,
    },

    /// Show current NTIX state
    State,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Apply {
            config_path,
            dry_run,
            no_gc,
            stop_on_failure,
            adopt,
            upgrade,
            apply_config,
        } => {
            commands::apply(
                config_path,
                dry_run,
                no_gc,
                stop_on_failure,
                adopt,
                upgrade,
                apply_config,
            )
            .await?
        }
        Commands::Diff {
            config_path,
            adopt,
            upgrade,
            apply_config,
        } => commands::diff_cmd(config_path, adopt, upgrade, apply_config).await?,
        Commands::State => commands::state_cmd()?,
    };

    std::process::exit(exit_code);
}
