use anyhow::Result;
use colored::*;
use std::path::Path;

pub fn show(data_dir: &Path) -> Result<()> {
    println!("{}", "Znote Configuration".bold().green());
    println!("-------------------");
    println!("{:<15} {}", "Data Directory:".bold(), data_dir.display());
    println!("{:<15} {}", "Version:".bold(), env!("CARGO_PKG_VERSION"));
    println!(
        "{:<15} {}",
        "Editor:".bold(),
        std::env::var("EDITOR").unwrap_or_else(|_| "not set".to_string())
    );
    Ok(())
}
