use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UseFlag {
    pub name: String,
    pub description: String,
    pub default: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Package {
    pub name: String,
    pub atom: String,
    pub version: String,
    pub overlay: String,
    pub homepage: String,
    pub description: String,
    pub license: String,
    pub use_flags: Vec<UseFlag>,
    pub keywords: Vec<String>,
    pub masked: bool,
    pub mask_reason: Option<String>,
    pub dependencies: Vec<String>,
}

impl Package {
    /// Enriches the USE flag descriptions by querying local Gentoo profiles.
    pub fn enrich_use_flags(&mut self) {
        for flag in &mut self.use_flags {
            // Query local Gentoo Profiles database for official descriptions if local database descriptions are short
            if let Some(desc) = get_local_use_desc(&self.atom, &flag.name) {
                flag.description = desc;
            } else if let Some(desc) = get_global_use_desc(&flag.name) {
                flag.description = desc;
            }

            if flag.description.is_empty() {
                flag.description = format!("Enable {} support", flag.name);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Overlay {
    pub name: String,
    pub url: String,
    pub trust_score: f32,
    pub packages_count: usize,
    pub maintainers: usize,
    pub last_update: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Database {
    pub overlays: Vec<Overlay>,
    pub packages: Vec<Package>,
}

impl Database {
    /// Load the database from a given file path, or fall back to the embedded JSON metadata.
    pub fn load() -> Self {
        let paths_to_try = [
            "ezmerge-api/db.json",
            "../ezmerge-api/db.json",
            "db.json",
        ];

        let mut db: Database = 'load_db: {
            for path in &paths_to_try {
                if Path::new(path).exists() {
                    if let Ok(content) = fs::read_to_string(path) {
                        if let Ok(db) = serde_json::from_str::<Database>(&content) {
                            break 'load_db db;
                        }
                    }
                }
            }
            let embedded_json = include_str!("../../ezmerge-api/db.json");
            serde_json::from_str(embedded_json).expect("Failed to parse embedded package database")
        };

        // Enrich packages with live descriptions from local Gentoo portage tree
        for pkg in &mut db.packages {
            pkg.enrich_use_flags();
        }

        db
    }

    /// Finds package by name or atom (returns owned package, searching live system overlays as fallback).
    pub fn find_package(&self, name: &str) -> Option<Package> {
        if let Some(p) = self.packages.iter().find(|p| p.name == name || p.atom == name) {
            return Some(p.clone());
        }

        // Search live repositories as fallback
        parse_system_package(name)
    }

    pub fn find_overlay(&self, name: &str) -> Option<&Overlay> {
        self.overlays.iter().find(|o| o.name == name)
    }
}

/// Dynamic resolver for global USE flag descriptions from Portage tree
pub fn get_global_use_desc(flag: &str) -> Option<String> {
    let paths = [
        "/var/db/repos/gentoo/profiles/use.desc",
        "/usr/portage/profiles/use.desc",
    ];
    for path in &paths {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(2, " - ").collect();
                if parts.len() == 2 && parts[0].trim() == flag {
                    return Some(parts[1].trim().to_string());
                }
            }
        }
    }
    None
}

/// Dynamic resolver for ebuild local USE flag descriptions from Portage tree
pub fn get_local_use_desc(atom: &str, flag: &str) -> Option<String> {
    let paths = [
        "/var/db/repos/gentoo/profiles/use.local.desc",
        "/usr/portage/profiles/use.local.desc",
    ];
    for path in &paths {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let prefix = format!("{}:{}", atom, flag);
                if line.starts_with(&prefix) {
                    let parts: Vec<&str> = line.splitn(2, " - ").collect();
                    if parts.len() == 2 {
                        return Some(parts[1].trim().to_string());
                    }
                }
            }
        }
    }
    None
}

/// Parses the ebuild file to create a complete dynamic Package struct.
pub fn parse_system_package(name: &str) -> Option<Package> {
    let parts: Vec<&str> = name.split('/').collect();
    let (category, pkg_name) = if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        // Resolve package short name (e.g. qutebrowser -> www-client/qutebrowser)
        let repos_dir = Path::new("/var/db/repos");
        let mut found_cat = String::new();
        if repos_dir.exists() {
            if let Ok(repo_entries) = fs::read_dir(repos_dir) {
                'outer: for repo_entry in repo_entries.filter_map(|e| e.ok()) {
                    if !repo_entry.path().is_dir() { continue; }
                    if let Ok(cat_entries) = fs::read_dir(repo_entry.path()) {
                        for cat_entry in cat_entries.filter_map(|e| e.ok()) {
                            if !cat_entry.path().is_dir() { continue; }
                            let cat_name = cat_entry.file_name().to_string_lossy().into_owned();
                            if cat_name == "profiles" || cat_name == "metadata" || cat_name == "eclass" || cat_name == "licenses" {
                                continue;
                            }
                            if cat_entry.path().join(name).is_dir() {
                                found_cat = cat_name;
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        if found_cat.is_empty() {
            return None;
        }
        (found_cat, name.to_string())
    };

    let atom = format!("{}/{}", category, pkg_name);
    let repos_dir = Path::new("/var/db/repos");
    if !repos_dir.exists() {
        return None;
    }

    if let Ok(repo_entries) = fs::read_dir(repos_dir) {
        for repo_entry in repo_entries.filter_map(|e| e.ok()) {
            let pkg_path = repo_entry.path().join(&category).join(&pkg_name);
            if pkg_path.is_dir() {
                if let Ok(ebuild_entries) = fs::read_dir(&pkg_path) {
                    let mut ebuilds: Vec<PathBuf> = ebuild_entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.is_file() && p.extension().map_or(false, |ext| ext == "ebuild"))
                        .collect();

                    if ebuilds.is_empty() {
                        continue;
                    }
                    ebuilds.sort();
                    let latest_ebuild = ebuilds.last()?;

                    let file_stem = latest_ebuild.file_stem()?.to_string_lossy().into_owned();
                    let version = file_stem.strip_prefix(&format!("{}-", pkg_name))
                        .unwrap_or(&file_stem)
                        .to_string();

                    let repo_name = repo_entry.file_name().to_string_lossy().into_owned();

                    if let Ok(content) = fs::read_to_string(latest_ebuild) {
                        return Some(parse_ebuild_content(
                            pkg_name.clone(),
                            atom.clone(),
                            version,
                            repo_name,
                            &content
                        ));
                    }
                }
            }
        }
    }

    None
}

fn parse_ebuild_content(
    name: String,
    atom: String,
    version: String,
    overlay: String,
    content: &str,
) -> Package {
    let mut homepage = String::new();
    let mut description = String::new();
    let mut license = String::new();
    let mut use_flags = Vec::new();
    let mut keywords = Vec::new();
    let mut dependencies = Vec::new();

    let mut in_rdepend = false;
    let mut rdepend_raw = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("HOMEPAGE=") {
            homepage = line.splitn(2, '=').nth(1).unwrap_or("").trim_matches(|c| c == '"' || c == '\'' || c == '`').to_string();
        } else if line.starts_with("DESCRIPTION=") {
            description = line.splitn(2, '=').nth(1).unwrap_or("").trim_matches(|c| c == '"' || c == '\'' || c == '`').to_string();
        } else if line.starts_with("LICENSE=") {
            license = line.splitn(2, '=').nth(1).unwrap_or("").trim_matches(|c| c == '"' || c == '\'' || c == '`').to_string();
        } else if line.starts_with("KEYWORDS=") {
            let val = line.splitn(2, '=').nth(1).unwrap_or("").trim_matches(|c| c == '"' || c == '\'' || c == '`');
            keywords = val.split_whitespace().map(|s| s.to_string()).collect();
        } else if line.starts_with("IUSE=") {
            let val = line.strip_prefix("IUSE=").unwrap_or("").trim_matches(|c| c == '"' || c == '\'' || c == '`');
            for flag_raw in val.split_whitespace() {
                let (flag_name, default) = if flag_raw.starts_with('+') {
                    (flag_raw.strip_prefix('+').unwrap_or(flag_raw), true)
                } else {
                    (flag_raw, false)
                };
                
                // Skip special compiler flags like abi_x86_64, etc.
                if flag_name.contains('_') && !flag_name.starts_with("python_targets") {
                    continue;
                }

                use_flags.push(UseFlag {
                    name: flag_name.to_string(),
                    description: String::new(),
                    default,
                });
            }
        }

        // Read RDEPEND lines
        if line.starts_with("RDEPEND=") {
            in_rdepend = true;
            let val = line.splitn(2, '=').nth(1).unwrap_or("");
            rdepend_raw.push_str(val);
            if val.contains('"') && val.matches('"').count() % 2 == 0 {
                in_rdepend = false;
            }
        } else if in_rdepend {
            rdepend_raw.push_str(" ");
            rdepend_raw.push_str(line);
            if line.contains('"') {
                in_rdepend = false;
            }
        }
    }

    // Process RDEPEND packages
    let clean_rdepend = rdepend_raw.trim_matches(|c| c == '"' || c == '\'' || c == '`' || c == '(' || c == ')');
    for token in clean_rdepend.split_whitespace() {
        if token.starts_with('>') || token.starts_with('<') || token.starts_with('=') || token == "||" || token == "!" {
            continue;
        }
        let clean_token = token.trim_start_matches(|c| c == '>' || c == '<' || c == '=' || c == '!');
        if clean_token.contains('/') {
            let parts: Vec<&str> = clean_token.split('-').collect();
            let atom_path = if parts.len() > 1 {
                let mut base_parts = Vec::new();
                for part in parts {
                    if part.chars().next().map_or(false, |c| c.is_ascii_digit()) || part.starts_with('r') {
                        break;
                    }
                    base_parts.push(part);
                }
                base_parts.join("-")
            } else {
                clean_token.to_string()
            };

            
            let clean_atom = atom_path.split(|c| c == '[' || c == ':').next().unwrap_or("").to_string();
            if clean_atom.contains('/') && !dependencies.contains(&clean_atom) {
                dependencies.push(clean_atom);
            }
        }
    }

    dependencies.truncate(5);

    let mut pkg = Package {
        name,
        atom,
        version,
        overlay,
        homepage,
        description,
        license,
        use_flags,
        keywords,
        masked: false,
        mask_reason: None,
        dependencies,
    };

    pkg.enrich_use_flags();
    pkg
}
