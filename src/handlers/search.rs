use anyhow::{Context, Result, bail};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Low-level: run rg -l <pattern> [files...] and return the list of matching file paths.
pub fn rg_matching_files(
    data_dir: &Path,
    pattern: &str,
    restrict_to: Option<&[String]>,
) -> Result<HashSet<String>> {
    let mut cmd = Command::new("rg");
    cmd.current_dir(data_dir);
    cmd.arg("-l").arg(pattern);
    if let Some(files) = restrict_to {
        if files.is_empty() {
            return Ok(HashSet::new());
        }
        cmd.args(files);
    }
    let output = cmd.output().context("Failed to run ripgrep")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}

/// List every .md file in all entity subdirectories.
pub fn all_md_files(data_dir: &Path) -> Result<HashSet<String>> {
    let mut all = HashSet::new();
    for subdir in &["notes", "bookmarks", "tasks"] {
        let dir = data_dir.join(subdir);
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir).context("reading data dir")? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md")
                && let Ok(rel) = path.strip_prefix(data_dir)
            {
                all.insert(rel.to_string_lossy().to_string());
            }
        }
    }
    Ok(all)
}

pub fn ensure_rg() -> Result<()> {
    if Command::new("rg").arg("--version").output().is_err() {
        bail!(
            "ripgrep (`rg`) is not installed or not in PATH.\nInstall it with: cargo install ripgrep"
        );
    }
    Ok(())
}

pub fn rip(data_dir: &Path, args: &[String]) -> Result<()> {
    ensure_rg()?;

    let mut cmd = Command::new("rg");
    cmd.current_dir(data_dir);
    cmd.args(args);

    let status = cmd.status().context("Failed to execute ripgrep")?;

    // Ripgrep exits with 1 if no matches, 2 on error, 0 on success.
    if !status.success() && status.code() != Some(1) {
        bail!("ripgrep failed with status: {}", status);
    }

    Ok(())
}
