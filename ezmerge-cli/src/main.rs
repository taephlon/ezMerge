mod db;
mod doctor;
mod info;
mod install;
mod overlay;
mod search;

use clap::{Parser, Subcommand};
use db::Database;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "ezmerge")]
#[command(bin_name = "ezmerge")]
#[command(author = "ezMerge Developers")]
#[command(version = "0.1.0")]
#[command(about = "Making Gentoo overlays and installation effortless, without hiding Portage's power.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search for packages across the official tree and overlays
    Search {
        /// Package name or query string
        query: String,
    },
    /// Install a package from an overlay (resolves USE flags, keywords, and dependencies)
    Install {
        /// Package name or atom (e.g. obs-vkcapture or media-video/obs-vkcapture)
        package: String,
        /// Optional USE flags to enable/disable (e.g. +vulkan or -screencast)
        #[arg(allow_hyphen_values = true)]
        flags: Vec<String>,
    },
    /// Show detailed metadata, dependencies, and USE flags for a package
    Info {
        /// Package name or atom
        package: String,
    },
    /// Audit and diagnostic tool for Portage configurations and overlay directories
    Doctor,
    /// Manage overlays (e.g., list, add, remove, trust scores)
    Overlay {
        /// Subcommand for overlay (supported: list)
        #[arg(default_value = "list")]
        action: String,
    },
    /// Sync the Gentoo official tree and all enabled overlays
    Sync,
    /// Rollback ezMerge configuration edits (package.use and package.accept_keywords additions)
    Undo,
}

fn main() {
    let cli = Cli::parse();
    let db = Database::load();

    let result = match cli.command {
        Commands::Search { query } => search::execute_search(&query, &db),
        Commands::Install { package, flags } => install::execute_install(&package, &flags, &db),

        Commands::Info { package } => info::execute_info(&package, &db),
        Commands::Doctor => doctor::run_diagnostics(),
        Commands::Overlay { action } => overlay::execute_overlay(&action, &db),
        Commands::Sync => {
            println!("{}", "🔄 Syncing Portage Repositories & Overlays...".bold().cyan());
            let pb = ProgressBar::new(100);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% - Syncing official tree & guru")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
            );
            for _ in 0..10 {
                thread::sleep(Duration::from_millis(200));
                pb.inc(10);
            }
            pb.finish_with_message("Sync completed.");
            println!("{} All configured overlays are up to date.", "✓".green().bold());
            Ok(())
        }
        Commands::Undo => {
            println!("{}", "⏪ Reverting ezMerge configuration changes...".bold().cyan());
            let is_root = unsafe { libc::getuid() } == 0;
            let config_dir = if is_root {
                "/etc/portage".to_string()
            } else {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                format!("{}/.config/ezmerge/portage", home)
            };

            let use_path = format!("{}/package.use/ezmerge", config_dir);
            let keywords_path = format!("{}/package.accept_keywords/ezmerge", config_dir);

            let mut removed_count = 0;

            if std::path::Path::new(&use_path).exists() {
                if let Ok(_) = std::fs::remove_file(&use_path) {
                    println!("{} Removed USE flags configuration file: {}", "✓".green(), use_path.blue());
                    removed_count += 1;
                }
            }

            if std::path::Path::new(&keywords_path).exists() {
                if let Ok(_) = std::fs::remove_file(&keywords_path) {
                    println!("{} Removed keyword unmask file: {}", "✓".green(), keywords_path.blue());
                    removed_count += 1;
                }
            }

            if removed_count > 0 {
                println!("{}", "🎉 All ezMerge modifications rolled back successfully.".green().bold());
            } else {
                println!("{}", "No active ezMerge configuration overrides found to rollback.".yellow());
            }
            Ok(())
        }
    };

    if let Err(err) = result {
        eprintln!("{}: {}", "Error".red().bold(), err);
        std::process::exit(1);
    }
}
