use std::path::Path;

use anyhow::{Result, bail};
use chrono::Utc;
use colored::Colorize;

use crate::commands::{ItemAddArgs, ItemUpdateArgs, TaskAddArgs, UpdateArgs};
use crate::hooks::{self, HookContext, HookEvent};
use crate::models::task::Task;
use crate::storage;

/// Parse a comma- or space-separated tag string into a Vec of `#tag` strings.
fn parse_tags(raw: &str) -> Vec<String> {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.starts_with('#') {
                s.to_string()
            } else {
                format!("#{s}")
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Task CRUD handlers
// ---------------------------------------------------------------------------

pub fn add(data_dir: &Path, args: &TaskAddArgs) -> Result<()> {
    let tags = args.tags.as_deref().map(parse_tags).unwrap_or_default();
    let links = args.links.as_deref().map(parse_tags).unwrap_or_default();
    let title = args.title.clone().unwrap_or_default();
    let mut task = Task::new(title.clone(), tags);
    let mut clean_links = Vec::new();
    for l in links {
        clean_links.push(l.strip_prefix('#').unwrap_or(&l).to_string());
    }
    task.links = clean_links;
    let id = task.id.clone();

    hooks::run(
        data_dir,
        HookEvent::BeforeAdd,
        &HookContext {
            entity_type: "task",
            id: Some(&id),
            title: Some(&title),
            path: None,
            old_content: None,
            new_content: Some(&storage::serialize_task(&task)?),
        },
    )?;

    storage::save_task(data_dir, &task)?;

    let path = storage::get_path(data_dir, "tasks", &id)?;
    let saved = std::fs::read_to_string(&path).unwrap_or_default();
    hooks::run(
        data_dir,
        HookEvent::AfterAdd,
        &HookContext {
            entity_type: "task",
            id: Some(&id),
            title: Some(&title),
            path: Some(&path),
            old_content: None,
            new_content: Some(&saved),
        },
    )?;

    println!("{} {}", "Created task".green().bold(), id.cyan());
    Ok(())
}

pub fn list(data_dir: &Path) -> Result<()> {
    let mut tasks = storage::list_tasks(data_dir)?;

    if tasks.is_empty() {
        println!("{}", "No tasks found.".dimmed());
        return Ok(());
    }

    // Sort newest first
    tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    println!("{}", "Tasks".bold().underline());
    for task in &tasks {
        let done = task.items.iter().filter(|i| i.completed).count();
        let total = task.items.len();

        let progress = if total > 0 {
            format!(" [{done}/{total}]").dimmed().to_string()
        } else {
            String::new()
        };

        let tag_str = if task.tags.is_empty() {
            String::new()
        } else {
            format!("  {}", task.tags.join(" ").dimmed())
        };

        println!(
            "{} {} {}{}{}",
            "•".dimmed(),
            task.id[..8].cyan(),
            task.title.bold(),
            progress,
            tag_str,
        );
    }
    Ok(())
}

pub fn view(data_dir: &Path, id: &str) -> Result<()> {
    let task = storage::load_task(data_dir, id)?;
    let path = storage::get_path(data_dir, "tasks", id)?;

    hooks::run(
        data_dir,
        HookEvent::BeforeView,
        &HookContext {
            entity_type: "task",
            id: Some(&task.id),
            title: Some(&task.title),
            path: Some(&path),
            old_content: None,
            new_content: None,
        },
    )?;

    let done = task.items.iter().filter(|i| i.completed).count();
    let total = task.items.len();

    println!("{}", task.title.bold().underline());
    println!(
        "{} {} | {} {} | {} {}/{} done",
        "id:".dimmed(),
        task.id.cyan(),
        "updated:".dimmed(),
        task.updated_at
            .format("%Y-%m-%d %H:%M UTC")
            .to_string()
            .dimmed(),
        "progress:".dimmed(),
        done,
        total,
    );
    if !task.tags.is_empty() {
        println!("{} {}", "tags:".dimmed(), task.tags.join(" ").yellow());
    }
    if !task.links.is_empty() {
        let links = storage::format_links(data_dir, &task.links);
        println!("{} {}", "outgoing links:".dimmed(), links.join(", ").blue());
    }
    let incoming = storage::get_incoming_links(data_dir, &task.id);
    if !incoming.is_empty() {
        println!(
            "{} {}",
            "incoming links:".dimmed(),
            incoming.join(", ").blue()
        );
    }
    println!("{}", "---".dimmed());

    if let Some(desc) = &task.description {
        let content = super::wiki::render_content(data_dir, desc);
        termimad::print_text(&content);
    }

    if task.items.is_empty() {
        println!("\n{}", "No items yet.".dimmed());
    } else {
        println!();
        for (i, item) in task.items.iter().enumerate() {
            let (tick, style): (&str, colored::Color) = if item.completed {
                ("✓", colored::Color::Green)
            } else {
                ("○", colored::Color::White)
            };
            let tag_str = if item.tags.is_empty() {
                String::new()
            } else {
                format!("  {}", item.tags.join(" ").dimmed())
            };
            println!(
                "  {}  {} {}{}",
                format!("{}", i + 1).dimmed(),
                tick.color(style),
                item.text,
                tag_str,
            );
        }
    }
    hooks::run(
        data_dir,
        HookEvent::AfterView,
        &HookContext {
            entity_type: "task",
            id: Some(&task.id),
            title: Some(&task.title),
            path: Some(&path),
            old_content: None,
            new_content: None,
        },
    )?;
    Ok(())
}

pub fn update(data_dir: &Path, args: &UpdateArgs) -> Result<()> {
    let mut task = storage::load_task(data_dir, &args.id)?;
    let path = storage::get_path(data_dir, "tasks", &args.id)?;
    let old_content = std::fs::read_to_string(&path).unwrap_or_default();

    if let Some(title) = &args.title {
        task.title = title.clone();
    }
    if let Some(tags_raw) = &args.tags {
        task.tags = parse_tags(tags_raw);
    }
    if let Some(links_raw) = &args.links {
        let parsed = parse_tags(links_raw);
        let mut clean_links = Vec::new();
        for l in parsed {
            clean_links.push(l.strip_prefix('#').unwrap_or(&l).to_string());
        }
        task.links = clean_links;
    }
    task.updated_at = Utc::now();

    let new_content = storage::serialize_task(&task)?;
    hooks::run(
        data_dir,
        HookEvent::BeforeSave,
        &HookContext {
            entity_type: "task",
            id: Some(&task.id),
            title: Some(&task.title),
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: Some(&new_content),
        },
    )?;

    storage::save_task(data_dir, &task)?;

    hooks::run(
        data_dir,
        HookEvent::AfterSave,
        &HookContext {
            entity_type: "task",
            id: Some(&task.id),
            title: Some(&task.title),
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: Some(&new_content),
        },
    )?;

    println!("{} {}", "Updated task".green().bold(), task.id.cyan());
    Ok(())
}

pub fn delete(data_dir: &Path, id: &str) -> Result<()> {
    let task = storage::load_task(data_dir, id)?;
    let path = storage::get_path(data_dir, "tasks", id)?;
    let old_content = std::fs::read_to_string(&path).unwrap_or_default();

    hooks::run(
        data_dir,
        HookEvent::BeforeDelete,
        &HookContext {
            entity_type: "task",
            id: Some(id),
            title: Some(&task.title),
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: None,
        },
    )?;

    storage::delete_task(data_dir, id)?;

    hooks::run(
        data_dir,
        HookEvent::AfterDelete,
        &HookContext {
            entity_type: "task",
            id: Some(id),
            title: Some(&task.title),
            path: None,
            old_content: Some(&old_content),
            new_content: None,
        },
    )?;

    println!("{} {}", "Deleted task".red().bold(), id.cyan());
    Ok(())
}

pub fn edit(data_dir: &Path, id: &str) -> Result<()> {
    let path = storage::get_path(data_dir, "tasks", id)?;
    let full_id = path.file_stem().unwrap().to_str().unwrap().to_string();
    let _ = storage::load_task(data_dir, id)?;
    let old_content = std::fs::read_to_string(&path).unwrap_or_default();

    hooks::run(
        data_dir,
        HookEvent::BeforeEdit,
        &HookContext {
            entity_type: "task",
            id: Some(&full_id),
            title: None,
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: None,
        },
    )?;

    let tmp_dir = tempfile::tempdir()?;
    let tmp_entity_dir = tmp_dir.path().join("tasks");
    std::fs::create_dir_all(&tmp_entity_dir)?;
    let tmp_path = tmp_entity_dir.join(format!("{}.md", full_id));
    std::fs::copy(&path, &tmp_path)?;

    super::run_editor(&tmp_path)?;

    match storage::load_task(tmp_dir.path(), &full_id) {
        Ok(mut task) => {
            task.updated_at = Utc::now();
            let new_content = std::fs::read_to_string(&tmp_path).unwrap_or_default();
            hooks::run(
                data_dir,
                HookEvent::AfterEdit,
                &HookContext {
                    entity_type: "task",
                    id: Some(&full_id),
                    title: Some(&task.title),
                    path: Some(&path),
                    old_content: Some(&old_content),
                    new_content: Some(&new_content),
                },
            )?;
            if let Err(e) = storage::save_task(data_dir, &task) {
                println!("{} {}", "Error: Failed to save changes:".red(), e);
            } else {
                println!("{} {}", "Edited task".green().bold(), task.id.cyan());
            }
        }
        Err(e) => {
            println!(
                "{} {}\n{}",
                "Error: Validation failed. Aborting changes for task"
                    .red()
                    .bold(),
                id.cyan(),
                e
            );
            anyhow::bail!("Invalid frontmatter");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Item handlers
// ---------------------------------------------------------------------------

/// Validate and resolve a 1-based item index, returning an error on bad input.
fn resolve_index(task: &Task, index: usize) -> Result<usize> {
    if index == 0 || index > task.items.len() {
        bail!(
            "Item index {} is out of range — task '{}' has {} item(s)",
            index,
            task.title,
            task.items.len()
        );
    }
    Ok(index - 1) // convert to 0-based
}

pub fn item_add(data_dir: &Path, task_id: &str, args: &ItemAddArgs) -> Result<()> {
    let mut task = storage::load_task(data_dir, task_id)?;

    let tags = args.tags.as_deref().map(parse_tags).unwrap_or_default();
    task.add_item(args.text.clone(), tags);

    storage::save_task(data_dir, &task)?;

    let n = task.items.len();
    println!(
        "{} item {} to task {}",
        "Added".green().bold(),
        format!("#{n}").cyan(),
        task_id[..8].dimmed(),
    );
    Ok(())
}

/// Shared handler for `check` and `uncheck`.
pub fn item_check(data_dir: &Path, task_id: &str, index: usize, done: bool) -> Result<()> {
    let mut task = storage::load_task(data_dir, task_id)?;
    let idx = resolve_index(&task, index)?;

    task.items[idx].completed = done;
    task.updated_at = Utc::now();
    storage::save_task(data_dir, &task)?;

    let verb = if done {
        "Checked".green()
    } else {
        "Unchecked".yellow()
    };
    println!(
        "{} item {} — \"{}\"",
        verb.bold(),
        format!("#{index}").cyan(),
        task.items[idx].text,
    );
    Ok(())
}

pub fn item_update(data_dir: &Path, task_id: &str, args: &ItemUpdateArgs) -> Result<()> {
    let mut task = storage::load_task(data_dir, task_id)?;
    let idx = resolve_index(&task, args.index)?;

    if let Some(text) = &args.text {
        task.items[idx].text = text.clone();
    }
    if let Some(tags_raw) = &args.tags {
        task.items[idx].tags = parse_tags(tags_raw);
    }
    task.updated_at = Utc::now();
    storage::save_task(data_dir, &task)?;

    println!(
        "{} item {} of task {}",
        "Updated".green().bold(),
        format!("#{}", args.index).cyan(),
        task_id[..8].dimmed(),
    );
    Ok(())
}

pub fn item_remove(data_dir: &Path, task_id: &str, index: usize) -> Result<()> {
    let mut task = storage::load_task(data_dir, task_id)?;
    let idx = resolve_index(&task, index)?;

    let removed = task.items.remove(idx);
    task.updated_at = Utc::now();
    storage::save_task(data_dir, &task)?;

    println!(
        "{} item {} — \"{}\"",
        "Removed".red().bold(),
        format!("#{index}").cyan(),
        removed.text,
    );
    Ok(())
}
