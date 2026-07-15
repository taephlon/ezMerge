use crate::db::Database;
use colored::Colorize;

pub fn execute_overlay(subcommand: &str, db: &Database) -> anyhow::Result<()> {
    match subcommand {
        "list" => {
            println!("{}", "📚 Available Gentoo Overlays (Curated Catalog)".bold().cyan());
            println!("--------------------------------------------------");
            for overlay in &db.overlays {
                let stars = (overlay.trust_score.round() as usize).min(5);
                let star_str = "★".repeat(stars) + &"☆".repeat(5 - stars);
                println!(
                    "{} (Trust: {} {})",
                    overlay.name.bold().magenta(),
                    overlay.trust_score.to_string().yellow(),
                    star_str.yellow()
                );
                println!("  URL:         {}", overlay.url.underline().dimmed());
                println!("  Packages:    {:<10} Maintainers: {}", overlay.packages_count, overlay.maintainers);
                println!("  Last Sync:   {}", overlay.last_update.blue());
                println!("  Description: {}", overlay.description.italic());
                println!();
            }
        }
        other => {
            anyhow::bail!("Unknown overlay subcommand '{}'. Supported subcommands: list", other);
        }
    }
    Ok(())
}
