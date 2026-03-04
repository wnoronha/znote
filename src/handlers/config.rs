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
    let backend = std::env::var("ZNOTE_STORAGE_BACKEND").unwrap_or_else(|_| "markdown".to_string());
    println!("{:<15} {}", "Storage:".bold(), backend);

    if backend == "dolt" {
        let host = std::env::var("ZNOTE_DOLT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("ZNOTE_DOLT_PORT").unwrap_or_else(|_| "3306".to_string());
        let user = std::env::var("ZNOTE_DOLT_USER").unwrap_or_else(|_| "root".to_string());
        let dbname = std::env::var("ZNOTE_DOLT_DB").unwrap_or_else(|_| "znote".to_string());
        println!(
            "{:<15} {}@{}:{}/{}",
            "Dolt server:".bold(),
            user,
            host,
            port,
            dbname
        );
    }
    Ok(())
}
