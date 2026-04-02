use std::process::Command as ProcessCommand;

use anyhow::{Result, bail};

/// ark update - check for and install the latest version
pub fn run() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    // Fetch latest release tag from GitHub
    println!("Checking for updates...");
    let output = ProcessCommand::new("curl")
        .args([
            "-sSL",
            "https://api.github.com/repos/milesmcleod/ark/releases/latest",
        ])
        .output()?;

    if !output.status.success() {
        bail!("failed to check for updates. Check your internet connection.");
    }

    let body = String::from_utf8_lossy(&output.stdout);

    // Extract tag_name from JSON (avoid adding a JSON parsing dependency for this)
    let latest_tag = body
        .lines()
        .find(|l| l.contains("\"tag_name\""))
        .and_then(|l| {
            // Extract value from: "tag_name": "v0.1.0",
            let after_colon = l.split(':').nth(1)?;
            let trimmed = after_colon.trim().trim_matches(|c| c == '"' || c == ',');
            Some(trimmed.to_string())
        });

    let latest_tag: String = match latest_tag {
        Some(t) => t,
        None => {
            println!("No releases found. You may be running a development build.");
            return Ok(());
        }
    };

    let latest_version = latest_tag.trim_start_matches('v');

    if latest_version == current_version {
        println!("Already up to date.");
        return Ok(());
    }

    println!(
        "New version available: {} -> {}",
        current_version, latest_version
    );

    // Detect platform
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let target = match (os, arch) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        _ => bail!(
            "unsupported platform: {}-{}. Download manually from GitHub.",
            os,
            arch
        ),
    };

    let artifact = format!("ark-{}", target);
    let url = format!(
        "https://github.com/milesmcleod/ark/releases/download/{}/{}",
        latest_tag, artifact
    );

    // Find where the current binary lives
    let current_exe = std::env::current_exe()?;
    let tmp_path = current_exe.with_extension("update");

    println!("Downloading {}...", url);

    let download = ProcessCommand::new("curl")
        .args(["-sSL", &url, "-o"])
        .arg(&tmp_path)
        .output()?;

    if !download.status.success() {
        bail!("download failed. Check the release exists at: {}", url);
    }

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&tmp_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&tmp_path, perms)?;
    }

    // Replace current binary
    std::fs::rename(&tmp_path, &current_exe)?;

    println!("Updated to ark {}.", latest_version);

    Ok(())
}
