pub mod agent;
pub mod bookmark;
pub mod completions;
pub mod config;
pub mod graph;
pub mod note;
pub mod query;
pub mod search;
pub mod serve;
pub mod task;
pub mod validate;
pub mod wiki;

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

/// Open the specified file path in the user's $EDITOR.
pub fn run_editor(path: &Path) -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_default();
    if editor.is_empty() {
        bail!(
            "$EDITOR environment variable is not set.\nPlease set it (e.g., `export EDITOR=nano` or `export EDITOR=vim`) and try again."
        );
    }

    let status = Command::new(&editor)
        .arg(path)
        .status()
        .with_context(|| format!("Failed to open editor: {}", editor))?;

    if !status.success() {
        bail!("Editor closed with a non-zero status.");
    }
    Ok(())
}
