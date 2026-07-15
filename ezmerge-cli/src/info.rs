use crate::db::Database;
use colored::Colorize;

pub fn execute_info(package_name: &str, db: &Database) -> anyhow::Result<()> {
    let pkg = match db.find_package(package_name) {
        Some(p) => p,
        None => {
            anyhow::bail!(
                "Package '{}' not found. Try searching for it first using `ezmerge search`.",
                package_name
            );
        }
    };

    let overlay_info = db.find_overlay(&pkg.overlay);

    println!("{}", "==================================================".dimmed());
    println!(
        "{}  {}",
        "📦 Package:".bold().cyan(),
        pkg.atom.bold().magenta()
    );
    println!("{}", "==================================================".dimmed());

    println!("{:<15} {}", "Latest Version:".bold(), pkg.version.green());
    println!("{:<15} {}", "License:".bold(), pkg.license.yellow());
    println!("{:<15} {}", "Homepage:".bold(), pkg.homepage.underline().blue());
    println!(
        "{:<15} {}",
        "Keywords:".bold(),
        pkg.keywords
            .iter()
            .map(|k| {
                if k.starts_with('~') {
                    k.yellow().to_string()
                } else {
                    k.green().to_string()
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    );

    if pkg.masked {
        println!(
            "{:<15} {}",
            "Masked:".bold(),
            "Yes (Ebuild is masked in repository)".red().bold()
        );
        if let Some(reason) = &pkg.mask_reason {
            println!("{:<15} {}", "Mask Reason:".bold(), reason.red().italic());
        }
    } else {
        println!("{:<15} {}", "Masked:".bold(), "No".green());
    }

    println!();
    println!("{}", "📝 Description:".bold().cyan());
    println!("  {}", pkg.description);
    println!();

    println!("{}", "🌐 Overlay Source:".bold().cyan());
    if let Some(overlay) = overlay_info {
        let stars = (overlay.trust_score.round() as usize).min(5);
        let star_str = "★".repeat(stars) + &"☆".repeat(5 - stars);
        println!(
            "  {} (Trust Score: {} {})",
            overlay.name.bold().blue(),
            overlay.trust_score.to_string().yellow(),
            star_str.yellow()
        );
        println!("  URL: {}", overlay.url.underline().dimmed());
        println!("  Info: {}", overlay.description);
    } else {
        println!("  {} (Local or custom overlay)", pkg.overlay.bold().blue());
    }
    println!();

    println!("{}", "🔗 Dependencies:".bold().cyan());
    if pkg.dependencies.is_empty() {
        println!("  None");
    } else {
        for dep in &pkg.dependencies {
            println!("  ├── {}", dep.dimmed());
        }
    }
    println!();

    println!("{}", "⚙  Available USE Flags:".bold().cyan());
    if pkg.use_flags.is_empty() {
        println!("  No USE flags defined.");
    } else {
        println!("  {:<15} {:<10} {:<30}", "USE Flag", "Default", "Description");
        println!("  {:<15} {:<10} {:<30}", "--------", "-------", "-----------");
        for flag in &pkg.use_flags {
            let default_str = if flag.default {
                "+ (Enabled)".green()
            } else {
                "- (Disabled)".red()
            };
            println!(
                "  {:<15} {:<10} {:<30}",
                flag.name.bold(),
                default_str,
                flag.description.dimmed()
            );
        }
    }
    println!("{}", "==================================================".dimmed());

    Ok(())
}
