use std::path::Path;
use std::process::Command;
use colored::Colorize;

pub struct DoctorReport {
    pub portage_installed: bool,
    pub write_permissions: bool,
    pub profile_exists: bool,
    pub make_conf_exists: bool,
    pub repos_conf_exists: bool,
    pub has_overlays: bool,
}

pub fn run_diagnostics() -> anyhow::Result<()> {
    println!("{}", "🔍 Running ezMerge System Diagnostics...".bold().cyan());
    println!("--------------------------------------------------");

    let mut report = DoctorReport {
        portage_installed: false,
        write_permissions: false,
        profile_exists: false,
        make_conf_exists: false,
        repos_conf_exists: false,
        has_overlays: false,
    };

    // 1. Check Portage / emerge
    print!("Checking for Portage installation... ");
    let emerge_check = Command::new("emerge").arg("--version").output();
    if let Ok(output) = emerge_check {
        report.portage_installed = true;
        let ver = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("Unknown Version")
            .to_string();
        println!("{} (Found: {})", "✓".green().bold(), ver.blue());
    } else {
        println!("{} (emerge command not found. Are you on Gentoo?)", "✗".red().bold());
    }

    // 2. Check Profile
    print!("Checking make.profile symlink... ");
    let profile_path = Path::new("/etc/portage/make.profile");
    if profile_path.exists() {
        report.profile_exists = true;
        if let Ok(link_target) = std::fs::read_link(profile_path) {
            println!("{} (Target: {})", "✓".green().bold(), link_target.to_string_lossy().blue());
        } else {
            println!("{}", "✓ (Path exists but could not read symlink target)".green());
        }
    } else {
        println!("{} (make.profile symlink missing at /etc/portage/make.profile)", "✗".red().bold());
    }

    // 3. Check make.conf
    print!("Checking /etc/portage/make.conf... ");
    let make_conf = Path::new("/etc/portage/make.conf");
    let make_conf_old = Path::new("/etc/make.conf");
    if make_conf.exists() {
        report.make_conf_exists = true;
        println!("{} (Found at /etc/portage/make.conf)", "✓".green().bold());
    } else if make_conf_old.exists() {
        report.make_conf_exists = true;
        println!("{} (Found legacy /etc/make.conf - recommendation: migrate to /etc/portage/make.conf)", "⚠".yellow().bold());
    } else {
        println!("{} (make.conf not found)", "✗".red().bold());
    }

    // 4. Check repos.conf
    print!("Checking repos.conf overlay config... ");
    let repos_conf_dir = Path::new("/etc/portage/repos.conf");
    let repos_conf_file = Path::new("/etc/portage/repos.conf.conf"); // sometimes exists
    if repos_conf_dir.exists() || repos_conf_file.exists() {
        report.repos_conf_exists = true;
        println!("{} (Found configured repos.conf)", "✓".green().bold());
    } else {
        println!("{} (repos.conf not configured - Portage will use defaults)", "⚠".yellow().bold());
    }

    // 5. Check overlay repositories
    print!("Scanning active overlays... ");
    let repos_dir = Path::new("/var/db/repos");
    if repos_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(repos_dir) {
            let overlay_names: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|name| name != "gentoo")
                .collect();

            if !overlay_names.is_empty() {
                report.has_overlays = true;
                println!(
                    "{} (Found {} overlays: {})",
                    "✓".green().bold(),
                    overlay_names.len(),
                    overlay_names.join(", ").blue()
                );
            } else {
                println!("{} (Only official 'gentoo' tree enabled)", "✓".green().bold());
            }
        } else {
            println!("{} (Found /var/db/repos but could not list directories)", "⚠".yellow().bold());
        }
    } else {
        println!("{} (/var/db/repos not found)", "⚠".yellow().bold());
    }

    // 6. Check write permissions to /etc/portage
    print!("Checking write permissions to /etc/portage... ");
    let test_dir = Path::new("/etc/portage");
    if test_dir.exists() {
        // try to write a dummy file or check metadata
        let metadata = std::fs::metadata(test_dir);
        if let Ok(meta) = metadata {
            if !meta.permissions().readonly() {
                // To be completely sure, try creating a temporary directory/file or just trust metadata
                // Actually, standard check is if running as root
                let is_root = unsafe { libc::getuid() } == 0;
                if is_root {
                    report.write_permissions = true;
                    println!("{} (Running as root - write access allowed)", "✓".green().bold());
                } else {
                    println!("{} (Read-only for normal user. Run as root/sudo for installs)", "⚠".yellow().bold());
                }
            } else {
                println!("{} (Read-only metadata)", "✗".red().bold());
            }
        } else {
            println!("{} (Failed to read permissions metadata)", "✗".red().bold());
        }
    } else {
        println!("{} (/etc/portage directory does not exist)", "✗".red().bold());
    }

    println!("--------------------------------------------------");
    if report.portage_installed && report.profile_exists && report.make_conf_exists {
        println!("{}", "🎉 System is healthy and ready for ezMerge!".green().bold());
    } else {
        println!("{}", "⚠ Some configuration discrepancies detected. Read checks above.".yellow().bold());
    }

    Ok(())
}
