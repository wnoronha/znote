use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use colored::Colorize;

use crate::commands::{BookmarkAddArgs, UpdateArgs};
use crate::hooks::{self, HookContext, HookEvent};
use crate::models::bookmark::Bookmark;
use crate::storage;

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

pub fn add(data_dir: &Path, args: &BookmarkAddArgs) -> Result<()> {
    let url = args.url.clone();
    let tags = args.tags.as_deref().map(parse_tags).unwrap_or_default();
    let links = args.links.as_deref().map(parse_tags).unwrap_or_default();
    let title = args.title.clone().unwrap_or_default();
    let mut bookmark = Bookmark::new(url, title.clone(), None, tags);
    let mut clean_links = Vec::new();
    for l in links {
        clean_links.push(l.strip_prefix('#').unwrap_or(&l).to_string());
    }
    bookmark.links = clean_links;
    let id = bookmark.id.clone();

    hooks::run(
        data_dir,
        HookEvent::BeforeAdd,
        &HookContext {
            entity_type: "bookmark",
            id: Some(&id),
            title: Some(&title),
            path: None,
            old_content: None,
            new_content: Some(&storage::serialize_bookmark(&bookmark)?),
        },
    )?;

    storage::save_bookmark(data_dir, &bookmark)?;

    let path = storage::get_path(data_dir, "bookmarks", &id)?;
    let saved = std::fs::read_to_string(&path).unwrap_or_default();
    hooks::run(
        data_dir,
        HookEvent::AfterAdd,
        &HookContext {
            entity_type: "bookmark",
            id: Some(&id),
            title: Some(&title),
            path: Some(&path),
            old_content: None,
            new_content: Some(&saved),
        },
    )?;

    println!("{} {}", "Created bookmark".green().bold(), id.cyan());
    Ok(())
}

pub fn list(data_dir: &Path) -> Result<()> {
    let mut bookmarks = storage::list_bookmarks(data_dir)?;
    if bookmarks.is_empty() {
        println!("{}", "No bookmarks found.".dimmed());
        return Ok(());
    }
    bookmarks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    println!("{}", "Bookmarks".bold().underline());
    for bm in &bookmarks {
        let tag_str = if bm.tags.is_empty() {
            String::new()
        } else {
            format!("  {}", bm.tags.join(" ").dimmed())
        };
        println!(
            "{} {} {} {}{}",
            "•".dimmed(),
            bm.id[..8].cyan(),
            bm.title.bold(),
            bm.url.dimmed(),
            tag_str
        );
    }
    Ok(())
}

pub fn view(data_dir: &Path, id: &str) -> Result<()> {
    let bm = storage::load_bookmark(data_dir, id)?;
    let path = storage::get_path(data_dir, "bookmarks", id)?;

    hooks::run(
        data_dir,
        HookEvent::BeforeView,
        &HookContext {
            entity_type: "bookmark",
            id: Some(&bm.id),
            title: Some(&bm.title),
            path: Some(&path),
            old_content: None,
            new_content: None,
        },
    )?;

    println!("{}", bm.title.bold().underline());
    println!(
        "{} {} | {} {}",
        "id:".dimmed(),
        bm.id.cyan(),
        "updated:".dimmed(),
        bm.updated_at
            .format("%Y-%m-%d %H:%M UTC")
            .to_string()
            .dimmed()
    );
    if !bm.tags.is_empty() {
        println!("{} {}", "tags:".dimmed(), bm.tags.join(" ").yellow());
    }
    if !bm.links.is_empty() {
        let links = storage::format_links(data_dir, &bm.links);
        println!("{} {}", "outgoing links:".dimmed(), links.join(", ").blue());
    }
    let incoming = storage::get_incoming_links(data_dir, &bm.id);
    if !incoming.is_empty() {
        println!(
            "{} {}",
            "incoming links:".dimmed(),
            incoming.join(", ").blue()
        );
    }
    println!("{} {}", "url:".dimmed(), bm.url.blue().underline());
    println!("{}", "---".dimmed());
    if let Some(desc) = &bm.description {
        let content = super::wiki::render_content(data_dir, desc);
        termimad::print_text(&content);
    }

    hooks::run(
        data_dir,
        HookEvent::AfterView,
        &HookContext {
            entity_type: "bookmark",
            id: Some(&bm.id),
            title: Some(&bm.title),
            path: Some(&path),
            old_content: None,
            new_content: None,
        },
    )?;
    Ok(())
}

pub fn update(data_dir: &Path, args: &UpdateArgs) -> Result<()> {
    let mut bm = storage::load_bookmark(data_dir, &args.id)?;
    let path = storage::get_path(data_dir, "bookmarks", &args.id)?;
    let old_content = std::fs::read_to_string(&path).unwrap_or_default();

    if let Some(title) = &args.title {
        bm.title = title.clone();
    }
    if let Some(url) = &args.url {
        bm.url = url.clone();
    }
    if let Some(content_raw) = &args.content {
        let content = if content_raw == "-" {
            super::read_stdin()?
        } else {
            content_raw.clone()
        };
        bm.description = if content.is_empty() { None } else { Some(content) };
    }
    if let Some(tags_raw) = &args.tags {
        bm.tags = parse_tags(tags_raw);
    }
    if let Some(links_raw) = &args.links {
        let parsed = parse_tags(links_raw);
        let mut clean_links = Vec::new();
        for l in parsed {
            clean_links.push(l.strip_prefix('#').unwrap_or(&l).to_string());
        }
        bm.links = clean_links;
    }
    bm.updated_at = Utc::now();

    let new_content = storage::serialize_bookmark(&bm)?;
    hooks::run(
        data_dir,
        HookEvent::BeforeSave,
        &HookContext {
            entity_type: "bookmark",
            id: Some(&bm.id),
            title: Some(&bm.title),
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: Some(&new_content),
        },
    )?;

    storage::save_bookmark(data_dir, &bm)?;

    hooks::run(
        data_dir,
        HookEvent::AfterSave,
        &HookContext {
            entity_type: "bookmark",
            id: Some(&bm.id),
            title: Some(&bm.title),
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: Some(&new_content),
        },
    )?;

    println!("{} {}", "Updated bookmark".green().bold(), bm.id.cyan());
    Ok(())
}

pub fn delete(data_dir: &Path, id: &str) -> Result<()> {
    let bm = storage::load_bookmark(data_dir, id)?;
    let path = storage::get_path(data_dir, "bookmarks", id)?;
    let old_content = std::fs::read_to_string(&path).unwrap_or_default();

    hooks::run(
        data_dir,
        HookEvent::BeforeDelete,
        &HookContext {
            entity_type: "bookmark",
            id: Some(id),
            title: Some(&bm.title),
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: None,
        },
    )?;

    storage::delete_bookmark(data_dir, id)?;

    hooks::run(
        data_dir,
        HookEvent::AfterDelete,
        &HookContext {
            entity_type: "bookmark",
            id: Some(id),
            title: Some(&bm.title),
            path: None,
            old_content: Some(&old_content),
            new_content: None,
        },
    )?;

    println!("{} {}", "Deleted bookmark".red().bold(), id.cyan());
    Ok(())
}

pub fn edit(data_dir: &Path, id: &str) -> Result<()> {
    let path = storage::get_path(data_dir, "bookmarks", id)?;
    let full_id = path.file_stem().unwrap().to_str().unwrap().to_string();
    let _ = storage::load_bookmark(data_dir, id)?;
    let old_content = std::fs::read_to_string(&path).unwrap_or_default();

    hooks::run(
        data_dir,
        HookEvent::BeforeEdit,
        &HookContext {
            entity_type: "bookmark",
            id: Some(&full_id),
            title: None,
            path: Some(&path),
            old_content: Some(&old_content),
            new_content: None,
        },
    )?;

    let tmp_dir = tempfile::tempdir()?;
    let tmp_entity_dir = tmp_dir.path().join("bookmarks");
    std::fs::create_dir_all(&tmp_entity_dir)?;
    let tmp_path = tmp_entity_dir.join(format!("{}.md", full_id));
    std::fs::copy(&path, &tmp_path)?;

    super::run_editor(&tmp_path)?;

    match storage::load_bookmark(tmp_dir.path(), &full_id) {
        Ok(mut bm) => {
            bm.updated_at = Utc::now();
            let new_content = std::fs::read_to_string(&tmp_path).unwrap_or_default();
            hooks::run(
                data_dir,
                HookEvent::AfterEdit,
                &HookContext {
                    entity_type: "bookmark",
                    id: Some(&full_id),
                    title: Some(&bm.title),
                    path: Some(&path),
                    old_content: Some(&old_content),
                    new_content: Some(&new_content),
                },
            )?;
            if let Err(e) = storage::save_bookmark(data_dir, &bm) {
                println!("{} {}", "Error: Failed to save changes:".red(), e);
            } else {
                println!("{} {}", "Edited bookmark".green().bold(), bm.id.cyan());
            }
        }
        Err(e) => {
            println!(
                "{} {}\n{}",
                "Error: Validation failed. Aborting changes for bookmark"
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
