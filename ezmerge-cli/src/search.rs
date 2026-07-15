use crate::db::Database;
use colored::Colorize;
use std::fs;
use std::path::Path;


use std::collections::HashSet;

/// Checks if a package is explicitly added to the user's @world profile.
pub fn is_in_world(atom: &str) -> bool {
    let paths_to_try = [
        "/var/lib/portage/world",
        "world_mock",
    ];

    for path in &paths_to_try {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                if line.trim() == atom {
                    return true;
                }
            }
        }
    }
    false
}

/// Checks if a package version is currently installed on the Gentoo system by inspecting /var/db/pkg.
pub fn is_installed(atom: &str) -> bool {
    let parts: Vec<&str> = atom.split('/').collect();
    if parts.len() == 2 {
        let category = parts[0];
        let name = parts[1];
        
        let path_str = format!("/var/db/pkg/{}", category);
        let path = Path::new(&path_str);
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let file_name = entry.file_name().to_string_lossy().into_owned();
                    if file_name.starts_with(name) {
                        return true;
                    }
                }
            }
        }
    }
    
    // Developer fallback mocks
    if atom == "sys-apps/portage" || atom == "app-shells/zsh" {
        return true;
    }
    
    false
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct SearchResult {

    pub atom: String,
    pub name: String,
    pub category: String,
    pub version: String,
    pub overlay: String,
    pub homepage: String,
    pub description: String,
    pub license: String,
    pub masked: bool,
}

/// Crawls a package directory to find ebuild versions, description, and homepage.
fn extract_ebuild_info(pkg_path: &Path) -> Option<(String, String, String, String)> {
    let mut ebuilds = Vec::new();
    if let Ok(entries) = fs::read_dir(pkg_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "ebuild") {
                ebuilds.push(path);
            }
        }
    }
    
    if ebuilds.is_empty() {
        return None;
    }
    
    // Sort ebuilds to find the latest version
    ebuilds.sort();
    let latest_ebuild = ebuilds.last()?;
    
    // Extract version from filename (e.g. neovim-0.10.0.ebuild -> 0.10.0)
    let stem = latest_ebuild.file_stem()?.to_string_lossy().into_owned();
    let pkg_name = pkg_path.file_name()?.to_string_lossy().into_owned();
    let version = stem.strip_prefix(&format!("{}-", pkg_name))
        .unwrap_or(&stem)
        .to_string();

    let mut homepage = String::new();
    let mut description = String::new();
    let mut license = String::new();

    if let Ok(content) = fs::read_to_string(latest_ebuild) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("HOMEPAGE=") {
                let val = line.strip_prefix("HOMEPAGE=").unwrap_or("");
                homepage = val.trim_matches(|c| c == '"' || c == '\'' || c == '`').to_string();
            } else if line.starts_with("DESCRIPTION=") {
                let val = line.strip_prefix("DESCRIPTION=").unwrap_or("");
                description = val.trim_matches(|c| c == '"' || c == '\'' || c == '`').to_string();
            } else if line.starts_with("LICENSE=") {
                let val = line.strip_prefix("LICENSE=").unwrap_or("");
                license = val.trim_matches(|c| c == '"' || c == '\'' || c == '`').to_string();
            }
        }
    }

    if license.is_empty() {
        license = "Unknown".to_string();
    }

    Some((version, homepage, description, license))
}

/// Crawls the local Portage tree and enabled overlays (/var/db/repos)
pub fn scan_system_packages(query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();
    let repos_dir = Path::new("/var/db/repos");
    if !repos_dir.exists() {
        return results;
    }

    if let Ok(repo_entries) = fs::read_dir(repos_dir) {
        for repo_entry in repo_entries.filter_map(|e| e.ok()) {
            if !repo_entry.path().is_dir() {
                continue;
            }
            let repo_name = repo_entry.file_name().to_string_lossy().into_owned();
            if repo_name.starts_with('.') {
                continue;
            }

            if let Ok(cat_entries) = fs::read_dir(repo_entry.path()) {
                for cat_entry in cat_entries.filter_map(|e| e.ok()) {
                    if !cat_entry.path().is_dir() {
                        continue;
                    }
                    let cat_name = cat_entry.file_name().to_string_lossy().into_owned();
                    // Skip metadata, profiles, licenses, eclasses
                    if cat_name == "profiles" || cat_name == "metadata" || cat_name == "eclass" || cat_name == "licenses" {
                        continue;
                    }

                    if let Ok(pkg_entries) = fs::read_dir(cat_entry.path()) {
                        for pkg_entry in pkg_entries.filter_map(|e| e.ok()) {
                            if !pkg_entry.path().is_dir() {
                                continue;
                            }
                            let pkg_name = pkg_entry.file_name().to_string_lossy().into_owned();
                            let atom = format!("{}/{}", cat_name, pkg_name);

                            // Match either by package name or the category/package atom path
                            if atom.to_lowercase().contains(&query_lower) || pkg_name.to_lowercase().contains(&query_lower) {
                                if let Some((version, homepage, description, license)) = extract_ebuild_info(&pkg_entry.path()) {
                                    results.push(SearchResult {
                                        atom,
                                        name: pkg_name,
                                        category: cat_name.clone(),
                                        version,
                                        overlay: repo_name.clone(),
                                        homepage,
                                        description,
                                        license,
                                        masked: false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

pub fn execute_search(query: &str, db: &Database) -> anyhow::Result<()> {
    println!("Searching packages matching '{}'...", query.cyan());
    println!();

    // 1. Scan live system packages in /var/db/repos/ (Gentoo Portage tree + enabled overlays)
    let mut matches = scan_system_packages(query);

    // Track matching atoms to avoid adding duplicates from curated database
    let mut matched_atoms: HashSet<String> = matches.iter().map(|m| m.atom.clone()).collect();

    // 2. Query our curated overlay database (db.json) for additional matches (e.g. overlays not yet enabled)
    let query_lower = query.to_lowercase();
    for pkg in &db.packages {
        if !matched_atoms.contains(&pkg.atom) {
            if pkg.name.to_lowercase().contains(&query_lower)
                || pkg.atom.to_lowercase().contains(&query_lower)
                || pkg.description.to_lowercase().contains(&query_lower)
            {
                matched_atoms.insert(pkg.atom.clone());
                matches.push(SearchResult {
                    atom: pkg.atom.clone(),
                    name: pkg.name.clone(),
                    category: pkg.atom.split('/').next().unwrap_or("").to_string(),
                    version: pkg.version.clone(),
                    overlay: pkg.overlay.clone(),
                    homepage: pkg.homepage.clone(),
                    description: pkg.description.clone(),
                    license: pkg.license.clone(),
                    masked: pkg.masked,
                });
            }
        }
    }

    // Sort matches alphabetically by category/package name
    matches.sort_by(|a, b| a.atom.cmp(&b.atom));

    // 3. Render results in classic emerge -s layout
    for pkg in &matches {
        println!("*  {}", pkg.atom.bold().green());

        let installed_ver = if is_installed(&pkg.atom) {
            pkg.version.green().to_string()
        } else {
            "[ Not Installed ]".dimmed().to_string()
        };

        let in_world_str = if is_in_world(&pkg.atom) {
            "yes".green().bold().to_string()
        } else {
            "no".to_string()
        };

        let repo_str = if pkg.overlay == "gentoo" {
            "gentoo (official)".green().to_string()
        } else {
            format!("@{} (third-party overlay)", pkg.overlay).cyan().to_string()
        };

        let mask_label = if pkg.masked {
            format!(" ({})", "masked".red().bold())
        } else {
            "".to_string()
        };

        println!("      {:<26} {}{}", "Latest version available:".bold(), pkg.version, mask_label);
        println!("      {:<26} {}", "Latest version installed:".bold(), installed_ver);
        println!("      {:<26} {}", "Installed in @world:".bold(), in_world_str);
        println!("      {:<26} {}", "Overlay repository:".bold(), repo_str);
        println!("      {:<26} {}", "Homepage:".bold(), pkg.homepage.underline().blue());
        println!("      {:<26} {}", "Description:".bold(), pkg.description);
        println!("      {:<26} {}", "License:".bold(), pkg.license.yellow());
        println!();
    }

    if !matches.is_empty() {
        println!(
            "Found {} matching packages.",
            matches.len().to_string().cyan()
        );
    } else {
        println!("{}", "No packages found matching search criteria.".yellow());
    }

    Ok(())
}
