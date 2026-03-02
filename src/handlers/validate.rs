use std::fs;
use std::path::Path;

use anyhow::Result;
use colored::Colorize;

use crate::hooks::{self, HookContext, HookEvent};
use crate::storage;

#[allow(clippy::type_complexity)]
pub fn frontmatter(data_dir: &Path) -> Result<()> {
    hooks::run(
        data_dir,
        HookEvent::BeforeValidate,
        &HookContext {
            entity_type: "",
            id: None,
            title: None,
            path: None,
            old_content: None,
            new_content: None,
        },
    )?;

    let mut total = 0;
    let mut invalid = 0;
    let mut check_category =
        |category: &str, mut process: Box<dyn FnMut(&Path, &str) -> Result<()>>| {
            let dir = data_dir.join(category);
            if !dir.exists() {
                return;
            }

            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        total += 1;
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                            && let Err(e) = process(data_dir, stem)
                        {
                            invalid += 1;
                            println!(
                                "{} {}: {:#}",
                                "Invalid frontmatter in".red().bold(),
                                path.display(),
                                e
                            );
                        }
                    }
                }
            }
        };

    check_category(
        "notes",
        Box::new(|d, i| {
            let mut note = storage::load_note(d, i)?;
            let mut fixed = false;

            let path = storage::get_path(d, "notes", i)?;
            let raw = fs::read_to_string(&path).unwrap_or_default();
            if !raw.contains("znote: note/v1/") {
                fixed = true;
                println!(
                    "{} in note {}",
                    "Updating note to new frontmatter format".yellow(),
                    i.cyan()
                );
            }
            if raw.contains("---\n\n") || raw.contains("---\r\n\r\n") {
                fixed = true;
                println!(
                    "{} in note {}",
                    "Auto-fixed empty lines after frontmatter".yellow(),
                    i.cyan()
                );
            }

            for link in &note.links {
                if let Some((rel, target)) = link.split_once(':') {
                    if rel.trim().is_empty() || target.trim().is_empty() {
                        anyhow::bail!(
                            "Invalid link format '{}' (expected relationship:uuid)",
                            link
                        );
                    }
                } else {
                    anyhow::bail!(
                        "Invalid link format '{}' (expected relationship:uuid)",
                        link
                    );
                }
            }

            for tag in &mut note.tags {
                if !tag.starts_with('#') {
                    *tag = format!("#{tag}");
                    fixed = true;
                    println!(
                        "{} in note {}",
                        "Auto-fixed missing '#' on tags".yellow(),
                        i.cyan()
                    );
                }
            }
            if note.updated_at < note.created_at {
                note.updated_at = note.created_at;
                fixed = true;
                println!(
                    "{} in note {}",
                    "Auto-fixed updated_at before created_at".yellow(),
                    i.cyan()
                );
            }
            if fixed {
                storage::save_note(d, &note)?;
                println!("{} note {}", "Saved auto-fixes for".green(), i.cyan());
            }
            Ok(())
        }),
    );

    check_category(
        "bookmarks",
        Box::new(|d, i| {
            let mut bm = storage::load_bookmark(d, i)?;
            let mut fixed = false;

            let path = storage::get_path(d, "bookmarks", i)?;
            let raw = fs::read_to_string(&path).unwrap_or_default();
            if !raw.contains("znote: bookmark/v1/") {
                fixed = true;
                println!(
                    "{} in bookmark {}",
                    "Updating bookmark to new frontmatter format".yellow(),
                    i.cyan()
                );
            }
            if raw.contains("---\n\n") || raw.contains("---\r\n\r\n") {
                fixed = true;
                println!(
                    "{} in bookmark {}",
                    "Auto-fixed empty lines after frontmatter".yellow(),
                    i.cyan()
                );
            }

            for link in &bm.links {
                if let Some((rel, target)) = link.split_once(':') {
                    if rel.trim().is_empty() || target.trim().is_empty() {
                        anyhow::bail!(
                            "Invalid link format '{}' (expected relationship:uuid)",
                            link
                        );
                    }
                } else {
                    anyhow::bail!(
                        "Invalid link format '{}' (expected relationship:uuid)",
                        link
                    );
                }
            }

            for tag in &mut bm.tags {
                if !tag.starts_with('#') {
                    *tag = format!("#{tag}");
                    fixed = true;
                    println!(
                        "{} in bookmark {}",
                        "Auto-fixed missing '#' on tags".yellow(),
                        i.cyan()
                    );
                }
            }
            if bm.updated_at < bm.created_at {
                bm.updated_at = bm.created_at;
                fixed = true;
                println!(
                    "{} in bookmark {}",
                    "Auto-fixed updated_at before created_at".yellow(),
                    i.cyan()
                );
            }
            if fixed {
                storage::save_bookmark(d, &bm)?;
                println!("{} bookmark {}", "Saved auto-fixes for".green(), i.cyan());
            }
            Ok(())
        }),
    );

    check_category(
        "tasks",
        Box::new(|d, i| {
            let mut task = storage::load_task(d, i)?;
            let mut fixed = false;

            let path = storage::get_path(d, "tasks", i)?;
            let raw = fs::read_to_string(&path).unwrap_or_default();
            if !raw.contains("znote: task/v1/") {
                fixed = true;
                println!(
                    "{} in task {}",
                    "Updating task to new frontmatter format".yellow(),
                    i.cyan()
                );
            }
            if raw.contains("---\n\n") || raw.contains("---\r\n\r\n") {
                fixed = true;
                println!(
                    "{} in task {}",
                    "Auto-fixed empty lines after frontmatter".yellow(),
                    i.cyan()
                );
            }

            for link in &task.links {
                if let Some((rel, target)) = link.split_once(':') {
                    if rel.trim().is_empty() || target.trim().is_empty() {
                        anyhow::bail!(
                            "Invalid link format '{}' (expected relationship:uuid)",
                            link
                        );
                    }
                } else {
                    anyhow::bail!(
                        "Invalid link format '{}' (expected relationship:uuid)",
                        link
                    );
                }
            }

            for tag in &mut task.tags {
                if !tag.starts_with('#') {
                    *tag = format!("#{tag}");
                    fixed = true;
                    println!(
                        "{} in task {}",
                        "Auto-fixed missing '#' on tags".yellow(),
                        i.cyan()
                    );
                }
            }
            for item in &mut task.items {
                for tag in &mut item.tags {
                    if !tag.starts_with('#') {
                        *tag = format!("#{tag}");
                        fixed = true;
                    }
                }
            }
            if task.updated_at < task.created_at {
                task.updated_at = task.created_at;
                fixed = true;
                println!(
                    "{} in task {}",
                    "Auto-fixed updated_at before created_at".yellow(),
                    i.cyan()
                );
            }
            if fixed {
                storage::save_task(d, &task)?;
                println!("{} task {}", "Saved auto-fixes for".green(), i.cyan());
            }
            Ok(())
        }),
    );

    if invalid > 0 {
        anyhow::bail!(
            "Validation failed: {} of {} files have invalid frontmatter.",
            invalid,
            total
        );
    }

    if total == 0 {
        println!("{}", "No markdown files found to validate.".dimmed());
    } else {
        println!(
            "{}",
            format!("Validated {} file(s) successfully.", total)
                .green()
                .bold()
        );
    }

    hooks::run(
        data_dir,
        HookEvent::AfterValidate,
        &HookContext {
            entity_type: "",
            id: None,
            title: None,
            path: None,
            old_content: None,
            new_content: None,
        },
    )?;

    Ok(())
}
