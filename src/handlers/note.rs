use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use colored::Colorize;

use crate::commands::{NoteAddArgs, UpdateArgs};
use crate::hooks::{self, HookContext, HookEvent};
use crate::models::note::Note;
use crate::storage;

/// Parse a comma- or space-separated tag string into a Vec of `#tag` strings.
/// Ensures every tag starts with `#`.
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

pub fn add(data_dir: &Path, args: &NoteAddArgs) -> Result<()> {
    let tags = args.tags.as_deref().map(parse_tags).unwrap_or_default();
    let links = args.links.as_deref().map(parse_tags).unwrap_or_default();
    let content = args.content.clone();
    let title = args.title.clone().unwrap_or_default();
    let mut note = Note::new(title.clone(), content, tags);
    let mut clean_links = Vec::new();
    for l in links {
        clean_links.push(l.strip_prefix('#').unwrap_or(&l).to_string());
    }
    note.links = clean_links;
    let id = note.id.clone();

    hooks::run(
        data_dir,
        HookEvent::BeforeAdd,
        &HookContext {
            entity_type: "note",
            id: Some(&id),
            title: Some(&title),
            path: None,
            old_content: None,
            new_content: Some(&storage::serialize_note(&note)?),
        },
    )?;

    storage::save_note(data_dir, &note)?;

    let path = storage::get_path(data_dir, "notes", &id)?;
    let saved = std::fs::read_to_string(&path).unwrap_or_default();
    hooks::run(
        data_dir,
        HookEvent::AfterAdd,
        &HookContext {
            entity_type: "note",
            id: Some(&id),
            title: Some(&title),
            path: Some(&path),
            old_content: None,
            new_content: Some(&saved),
        },
    )?;

    println!("{} {}", "Created note".green().bold(), id.cyan());
    Ok(())
}

pub fn list(data_dir: &Path) -> Result<()> {
    let mut notes = storage::list_notes(data_dir)?;

    if notes.is_empty() {
        println!("{}", "No notes found.".dimmed());
        return Ok(());
    }

    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    println!("{}", "Notes".bold().underline());
    for note in &notes {
        let tag_str = if note.tags.is_empty() {
            String::new()
        } else {
            format!("  {}", note.tags.join(" ").dimmed())
        };
        println!(
            "{} {} {}{}",
            "•".dimmed(),
            note.id[..8].cyan(),
            note.title.bold(),
            tag_str
        );
    }
    Ok(())
}

pub fn view(data_dir: &Path, id: &str) -> Result<()> {
    let note = storage::load_note(data_dir, id)?;
    let path = storage::get_path(data_dir, "notes", id)?;

    hooks::run(
        data_dir,
        HookEvent::BeforeView,
        &HookContext {
            entity_type: "note",
            id: Some(&note.id),
            title: Some(&note.title),
            path: Some(&path),
            old_content: None,
            new_content: None,
        },
    )?;

    println!("{}", note.title.bold().underline());
    println!(
        "{} {} | {} {}",
        "id:".dimmed(),
        note.id.cyan(),
        "updated:".dimmed(),
        note.updated_at
            .format("%Y-%m-%d %H:%M UTC")
            .to_string()
            .dimmed(),
    );
    if !note.tags.is_empty() {
        println!("{} {}", "tags:".dimmed(), note.tags.join(" ").yellow());
    }
    if !note.links.is_empty() {
        let links = storage::format_links(data_dir, &note.links);
        println!("{} {}", "outgoing links:".dimmed(), links.join(", ").blue());
    }
    let incoming = storage::get_incoming_links(data_dir, &note.id);
    if !incoming.is_empty() {
        println!(
            "{} {}",
            "incoming links:".dimmed(),
            incoming.join(", ").blue()
        );
    }
    println!("{}", "---".dimmed());
    if !note.content.is_empty() {
        let content = super::wiki::render_content(data_dir, &note.content);
        termimad::print_text(&content);
    }

    hooks::run(
        data_dir,
        HookEvent::AfterView,
        &HookContext {
            entity_type: "note",
            id: Some(&note.id),
            title: Some(&note.title),
            path: Some(&path),
            old_content: None,
            new_content: None,
        },
    )?;
    Ok(())
}

pub fn update(data_dir: &Path, args: &UpdateArgs) -> Result<()> {
    let mut note = storage::load_note(data_dir, &args.id)?;
    let path = storage::get_path(data_dir, "notes", &args.id)?;
    let old_content = std::fs::read_to_string(&path).unwrap_or_default();

    if let Some(title) = &args.title {
        note.title = title.clone();
    }
    if let Some(content) = &args.content {
        note.content = content.clone();
    }
    if let Some(tags_raw) = &args.tags {
        note.tags = parse_tags(tags_raw);
    }
    if let Some(links_raw) = &args.links {
        let parsed = parse_tags(links_raw);
        let mut clean_links = Vec::new();
        for l in parsed {
            clean_links.push(l.strip_prefix('#').unwrap_or(&l).to_string());
        }
        note.links = clean_links;
    }
    note.updated_at = Utc::now();

    let new_content = storage::serialize_note(&note)?;
    hooks::run(
        data_dir,
        HookEvent::BeforeSave,
        &HookContext {
            entity_type: "note",
            id: Some(&note.id),
            title: Some(&note.title),
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: Some(&new_content),
        },
    )?;

    storage::save_note(data_dir, &note)?;

    hooks::run(
        data_dir,
        HookEvent::AfterSave,
        &HookContext {
            entity_type: "note",
            id: Some(&note.id),
            title: Some(&note.title),
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: Some(&new_content),
        },
    )?;

    println!("{} {}", "Updated note".green().bold(), note.id.cyan());
    Ok(())
}

pub fn delete(data_dir: &Path, id: &str) -> Result<()> {
    let note = storage::load_note(data_dir, id)?;
    let path = storage::get_path(data_dir, "notes", id)?;
    let old_content = std::fs::read_to_string(&path).unwrap_or_default();

    hooks::run(
        data_dir,
        HookEvent::BeforeDelete,
        &HookContext {
            entity_type: "note",
            id: Some(id),
            title: Some(&note.title),
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: None,
        },
    )?;

    storage::delete_note(data_dir, id)?;

    hooks::run(
        data_dir,
        HookEvent::AfterDelete,
        &HookContext {
            entity_type: "note",
            id: Some(id),
            title: Some(&note.title),
            path: None,
            old_content: Some(&old_content),
            new_content: None,
        },
    )?;

    println!("{} {}", "Deleted note".red().bold(), id.cyan());
    Ok(())
}

pub fn edit(data_dir: &Path, id: &str) -> Result<()> {
    let path = storage::get_path(data_dir, "notes", id)?;
    let full_id = path.file_stem().unwrap().to_str().unwrap().to_string();
    let _ = storage::load_note(data_dir, id)?;
    let old_content = std::fs::read_to_string(&path).unwrap_or_default();

    hooks::run(
        data_dir,
        HookEvent::BeforeEdit,
        &HookContext {
            entity_type: "note",
            id: Some(&full_id),
            title: None,
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: None,
        },
    )?;

    let tmp_dir = tempfile::tempdir()?;
    let tmp_entity_dir = tmp_dir.path().join("notes");
    std::fs::create_dir_all(&tmp_entity_dir)?;
    let tmp_path = tmp_entity_dir.join(format!("{}.md", full_id));
    std::fs::copy(&path, &tmp_path)?;

    super::run_editor(&tmp_path)?;

    match storage::load_note(tmp_dir.path(), &full_id) {
        Ok(mut note) => {
            note.updated_at = Utc::now();
            let new_content = std::fs::read_to_string(&tmp_path).unwrap_or_default();

            hooks::run(
                data_dir,
                HookEvent::AfterEdit,
                &HookContext {
                    entity_type: "note",
                    id: Some(&full_id),
                    title: Some(&note.title),
                    path: Some(&path),
                    old_content: Some(&old_content),
                    new_content: Some(&new_content),
                },
            )?;

            if let Err(e) = storage::save_note(data_dir, &note) {
                println!("{} {}", "Error: Failed to save changes:".red(), e);
            } else {
                println!("{} {}", "Edited note".green().bold(), note.id.cyan());
            }
        }
        Err(e) => {
            println!(
                "{} {}\n{}",
                "Error: Validation failed. Aborting changes for note"
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
