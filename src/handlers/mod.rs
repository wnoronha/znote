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

/// Read the entire content of stdin into a string.
pub fn read_stdin() -> Result<String> {
    read_all(&mut std::io::stdin())
}

/// Helper to read all content from a reader into a String.
pub fn read_all<R: std::io::Read>(mut reader: R) -> Result<String> {
    let mut buffer = String::new();
    reader
        .read_to_string(&mut buffer)
        .context("Failed to read all from source")?;
    Ok(buffer)
}

#[cfg(test)]
mod handlers_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_all() {
        let input = "Hello world\nThis is a test.";
        let reader = Cursor::new(input);
        let result = read_all(reader).unwrap();
        assert_eq!(result, input);
    }
}
