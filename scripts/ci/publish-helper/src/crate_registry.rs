use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub fn publish_crate(dry_run: bool, crates_io_token: &str, cargo_toml_path: &Path) -> Result<()> {
    let crate_dir = cargo_toml_path
        .parent()
        .context("Failed to find parent directory of Cargo.toml")?;

    let mut cargo_command = Command::new("cargo");
    cargo_command.arg("publish");

    if dry_run {
        cargo_command.arg("--dry-run");
    }

    let output = cargo_command
        .arg(format!("--token={}", crates_io_token))
        .current_dir(crate_dir)
        .output()
        .context("Failed to execute 'cargo publish' command")?;

    if !output.status.success() {
        bail!("Crate publishing failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    println!("Crate published successfully: {}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

// Check if a crate version is uploaded to crates.io
pub fn is_crate_version_uploaded(name: &str, version: &str) -> bool {
    let output = Command::new("curl")
        .arg(format!("https://crates.io/api/v1/crates/{}/{}", name, version))
        .output()
        .expect("Failed to execute curl");

    let response: serde_json::Value = serde_json::from_slice(&output.stdout).expect("Failed to parse JSON");
    response.get("version").is_some()
}
