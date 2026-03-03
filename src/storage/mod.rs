use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::models::bookmark::Bookmark;
use crate::models::note::Note;
use crate::models::task::{Task, TaskItem};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
pub mod dolt;

pub fn is_dolt_backend() -> bool {
    std::env::var("ZNOTE_STORAGE_BACKEND")
        .map(|v| v.to_lowercase() == "dolt")
        .unwrap_or(false)
}

/// Return the path for a given entity directory, creating it if absent.
fn entity_dir(data_dir: &Path, entity: &str) -> Result<PathBuf> {
    let dir = data_dir.join(entity);
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    Ok(dir)
}

/// Build the full file path for an item: `<dir>/<uuid>.md`
fn item_path(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.md"))
}

/// Resolve a full or partial ID to the full UUID string.
fn resolve_id(dir: &Path, id_prefix: &str) -> Result<String> {
    let exact_path = dir.join(format!("{id_prefix}.md"));
    if exact_path.exists() {
        return Ok(id_prefix.to_string());
    }

    let mut matches = Vec::new();
    if dir.exists() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && stem.starts_with(id_prefix)
            {
                matches.push(stem.to_string());
            }
        }
    }

    match matches.len() {
        0 => anyhow::bail!("No item found matching '{}'", id_prefix),
        1 => Ok(matches.pop().unwrap()),
        _ => anyhow::bail!("Ambiguous ID '{}': matches multiple items", id_prefix),
    }
}

/// Returns the exact file path for a given category and partial ID
pub fn get_path(data_dir: &Path, category: &str, id: &str) -> Result<PathBuf> {
    let dir = entity_dir(data_dir, category)?;
    let full_id = resolve_id(&dir, id)?;
    Ok(item_path(&dir, &full_id))
}

/// Split a `.md` file into (frontmatter_str, body_str).
/// Expects the file to start with `---\n` and have a closing `---\n`.
fn split_frontmatter(raw: &str) -> Result<(&str, &str)> {
    let raw = raw.trim_start_matches('\u{feff}'); // strip BOM if present
    let after_open = raw
        .strip_prefix("---\n")
        .or_else(|| raw.strip_prefix("---\r\n"))
        .context("Missing opening --- in frontmatter")?;

    // Find the closing ---
    let close_pat_unix = "\n---\n";
    let close_pat_win = "\n---\r\n";
    let close_pat_eof = "\n---";

    let (fm, body) = if let Some(pos) = after_open.find(close_pat_unix) {
        (
            &after_open[..pos],
            &after_open[pos + close_pat_unix.len()..],
        )
    } else if let Some(pos) = after_open.find(close_pat_win) {
        (&after_open[..pos], &after_open[pos + close_pat_win.len()..])
    } else if let Some(pos) = after_open.find(close_pat_eof) {
        (&after_open[..pos], &after_open[pos + close_pat_eof.len()..])
    } else {
        anyhow::bail!("Missing closing --- in frontmatter");
    };

    Ok((fm, body))
}

// ---------------------------------------------------------------------------
// Unified Frontmatter
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Frontmatter {
    pub znote: String, // "type/version/id"
    #[serde(default)]
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub tags: String,
    pub links: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Frontmatter {
    pub fn parse_znote(&self) -> Result<(&str, &str, &str)> {
        let parts: Vec<&str> = self.znote.split('/').collect();
        if parts.len() != 3 {
            anyhow::bail!("Invalid znote field format: '{}'", self.znote);
        }
        Ok((parts[0], parts[1], parts[2]))
    }
}

fn vec_to_space_string(v: &[String]) -> String {
    v.join(" ")
}

fn space_string_to_vec(s: &str) -> Vec<String> {
    s.split_whitespace().map(|s| s.to_string()).collect()
}

fn extract_title(title: &str, content: &str) -> String {
    if !title.trim().is_empty() {
        return title.to_string();
    }
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            return stripped.trim().to_string();
        } else if let Some(stripped) = trimmed.strip_prefix("## ") {
            return stripped.trim().to_string();
        } else if let Some(stripped) = trimmed.strip_prefix("### ") {
            return stripped.trim().to_string();
        }
    }
    "Untitled".to_string()
}

fn generate_safe_bookmark_title(raw_url: &str) -> String {
    if let Ok(mut parsed) = url::Url::parse(raw_url) {
        let _ = parsed.set_password(None);
        match parsed.scheme() {
            "http" | "https" => parsed.host_str().unwrap_or(raw_url).to_string(),
            "file" => Path::new(parsed.path())
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Local File")
                .to_string(),
            _ => parsed.to_string(),
        }
    } else {
        "Untitled Bookmark".to_string()
    }
}

// ---------------------------------------------------------------------------
// Note storage
// ---------------------------------------------------------------------------

pub fn serialize_note(note: &Note) -> Result<String> {
    let fm = Frontmatter {
        znote: format!("note/v1/{}", note.id),
        title: note.title.clone(),
        url: None,
        tags: vec_to_space_string(&note.tags),
        links: vec_to_space_string(&note.links),
        created_at: note.created_at,
        updated_at: note.updated_at,
    };
    let fm_str = serde_yaml::to_string(&fm).context("Failed to serialise note frontmatter")?;
    Ok(format!("---\n{}---\n{}", fm_str, note.content))
}

pub fn save_note_fs(data_dir: &Path, note: &Note) -> Result<()> {
    let dir = entity_dir(data_dir, "notes")?;
    let path = item_path(&dir, &note.id);
    let content = serialize_note(note)?;
    fs::write(&path, content).with_context(|| format!("Failed to write note: {}", path.display()))
}

pub fn load_note_fs(data_dir: &Path, id: &str) -> Result<Note> {
    let dir = entity_dir(data_dir, "notes")?;
    let full_id = resolve_id(&dir, id)?;
    let path = item_path(&dir, &full_id);

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read note: {}", path.display()))?;
    let (fm_str, body) = split_frontmatter(&raw)?;

    // Try new format first
    if let Ok(fm) = serde_yaml::from_str::<Frontmatter>(fm_str) {
        let (_, _, stored_id) = fm.parse_znote()?;
        let content_str = body.trim_start_matches('\n');
        return Ok(Note {
            id: stored_id.to_string(),
            title: extract_title(&fm.title, content_str),
            content: content_str.to_string(),
            tags: space_string_to_vec(&fm.tags),
            links: space_string_to_vec(&fm.links),
            created_at: fm.created_at,
            updated_at: fm.updated_at,
        });
    }

    // Fallback to legacy format
    #[derive(serde::Deserialize)]
    struct LegacyNoteFrontmatter {
        #[serde(default)]
        title: String,
        #[serde(default)]
        tags: Vec<String>,
        #[serde(default)]
        links: Vec<String>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let fm: LegacyNoteFrontmatter =
        serde_yaml::from_str(fm_str).context("Failed to parse note frontmatter (legacy or new)")?;

    let content_str = body.trim_start_matches('\n');
    Ok(Note {
        id: full_id,
        title: extract_title(&fm.title, content_str),
        content: content_str.to_string(),
        tags: fm.tags,
        links: fm.links,
        created_at: fm.created_at,
        updated_at: fm.updated_at,
    })
}

pub fn delete_note_fs(data_dir: &Path, id: &str) -> Result<()> {
    let dir = entity_dir(data_dir, "notes")?;
    let full_id = resolve_id(&dir, id)?;
    let path = item_path(&dir, &full_id);
    fs::remove_file(&path).with_context(|| format!("Failed to delete note: {}", path.display()))
}

pub fn list_notes_fs(data_dir: &Path) -> Result<Vec<Note>> {
    let dir = entity_dir(data_dir, "notes")?;
    load_all_from_dir(&dir, |id| load_note_fs(data_dir, id))
}

// ---------------------------------------------------------------------------
// Bookmark storage
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Bookmark storage
// ---------------------------------------------------------------------------

pub fn serialize_bookmark(bookmark: &Bookmark) -> Result<String> {
    let fm = Frontmatter {
        znote: format!("bookmark/v1/{}", bookmark.id),
        title: bookmark.title.clone(),
        url: Some(bookmark.url.clone()),
        tags: vec_to_space_string(&bookmark.tags),
        links: vec_to_space_string(&bookmark.links),
        created_at: bookmark.created_at,
        updated_at: bookmark.updated_at,
    };
    let fm_str = serde_yaml::to_string(&fm).context("Failed to serialise bookmark frontmatter")?;
    let body = bookmark.description.as_deref().unwrap_or("");
    Ok(if body.is_empty() {
        format!("---\n{}---\n", fm_str)
    } else {
        format!("---\n{}---\n{}", fm_str, body)
    })
}

pub fn save_bookmark_fs(data_dir: &Path, bookmark: &Bookmark) -> Result<()> {
    let dir = entity_dir(data_dir, "bookmarks")?;
    let path = item_path(&dir, &bookmark.id);
    let content = serialize_bookmark(bookmark)?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write bookmark: {}", path.display()))
}

pub fn load_bookmark_fs(data_dir: &Path, id: &str) -> Result<Bookmark> {
    let dir = entity_dir(data_dir, "bookmarks")?;
    let full_id = resolve_id(&dir, id)?;
    let path = item_path(&dir, &full_id);

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read bookmark: {}", path.display()))?;
    let (fm_str, body) = split_frontmatter(&raw)?;

    let description = {
        let text = body.trim_start_matches('\n');
        if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        }
    };

    // Try new format first
    if let Ok(fm) = serde_yaml::from_str::<Frontmatter>(fm_str) {
        let (_, _, stored_id) = fm.parse_znote()?;
        let url = fm.url.clone().unwrap_or_default();
        let extracted = extract_title(&fm.title, body.trim_start_matches('\n'));
        let final_title = if extracted == "Untitled" && !url.is_empty() {
            generate_safe_bookmark_title(&url)
        } else {
            extracted
        };

        return Ok(Bookmark {
            id: stored_id.to_string(),
            url,
            title: final_title,
            description,
            tags: space_string_to_vec(&fm.tags),
            links: space_string_to_vec(&fm.links),
            created_at: fm.created_at,
            updated_at: fm.updated_at,
        });
    }

    // Fallback to legacy
    #[derive(serde::Deserialize)]
    struct LegacyBookmarkFrontmatter {
        #[serde(default)]
        title: String,
        url: String,
        #[serde(default)]
        tags: Vec<String>,
        #[serde(default)]
        links: Vec<String>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let fm: LegacyBookmarkFrontmatter =
        serde_yaml::from_str(fm_str).context("Failed to parse bookmark frontmatter")?;

    let extracted = extract_title(&fm.title, body.trim_start_matches('\n'));
    let final_title = if extracted == "Untitled" && !fm.url.is_empty() {
        generate_safe_bookmark_title(&fm.url)
    } else {
        extracted
    };

    Ok(Bookmark {
        id: full_id,
        url: fm.url.clone(),
        title: final_title,
        description,
        tags: fm.tags,
        links: fm.links,
        created_at: fm.created_at,
        updated_at: fm.updated_at,
    })
}

pub fn delete_bookmark_fs(data_dir: &Path, id: &str) -> Result<()> {
    let dir = entity_dir(data_dir, "bookmarks")?;
    let full_id = resolve_id(&dir, id)?;
    let path = item_path(&dir, &full_id);
    fs::remove_file(&path).with_context(|| format!("Failed to delete bookmark: {}", path.display()))
}

pub fn list_bookmarks_fs(data_dir: &Path) -> Result<Vec<Bookmark>> {
    let dir = entity_dir(data_dir, "bookmarks")?;
    load_all_from_dir(&dir, |id| load_bookmark_fs(data_dir, id))
}

// ---------------------------------------------------------------------------
// Task storage
// Tasks are serialised as YAML frontmatter (title, tags, timestamps) +
// a markdown body that encodes the task items as a GFM checklist:
//   - [ ] item text  #tag1 #tag2
//   - [x] done item  #tag3
// ---------------------------------------------------------------------------

/// Serialise task items to a GFM checklist body.
fn items_to_body(items: &[TaskItem]) -> String {
    items
        .iter()
        .map(|item| {
            let check = if item.completed { "x" } else { " " };
            let tag_suffix = if item.tags.is_empty() {
                String::new()
            } else {
                format!("  {}", item.tags.join(" "))
            };
            format!("- [{check}] {}{tag_suffix}", item.text)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse a GFM checklist body back into (Option<description>, Vec<TaskItem>).
fn body_to_components(body: &str) -> (Option<String>, Vec<TaskItem>) {
    let mut description_lines = Vec::new();
    let mut items = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        let rest = trimmed
            .strip_prefix("- [ ] ")
            .map(|r| (false, r))
            .or_else(|| {
                trimmed
                    .strip_prefix("- [x] ")
                    .map(|r| (true, r))
                    .or_else(|| trimmed.strip_prefix("- [X] ").map(|r| (true, r)))
            })
            .or_else(|| {
                trimmed
                    .strip_prefix("-- [ ] ")
                    .map(|r| (false, r))
                    .or_else(|| {
                        trimmed
                            .strip_prefix("-- [x] ")
                            .map(|r| (true, r))
                            .or_else(|| trimmed.strip_prefix("-- [X] ").map(|r| (true, r)))
                    })
            });

        if let Some((completed, rest)) = rest {
            let mut text_parts: Vec<&str> = Vec::new();
            let mut tags: Vec<String> = Vec::new();
            for word in rest.split_whitespace() {
                if word.starts_with('#') {
                    tags.push(word.to_string());
                } else {
                    text_parts.push(word);
                }
            }
            let text = text_parts.join(" ");
            items.push(TaskItem {
                text,
                completed,
                tags,
            });
        } else {
            description_lines.push(line);
        }
    }

    let desc = description_lines.join("\n").trim().to_string();
    let opt_desc = if desc.is_empty() { None } else { Some(desc) };

    (opt_desc, items)
}

pub fn serialize_task(task: &Task) -> Result<String> {
    let fm = Frontmatter {
        znote: format!("task/v1/{}", task.id),
        title: task.title.clone(),
        url: None,
        tags: vec_to_space_string(&task.tags),
        links: vec_to_space_string(&task.links),
        created_at: task.created_at,
        updated_at: task.updated_at,
    };
    let fm_str = serde_yaml::to_string(&fm).context("Failed to serialise task frontmatter")?;
    let items_body = items_to_body(&task.items);
    let desc_body = task.description.as_deref().unwrap_or("");
    Ok(if desc_body.is_empty() {
        format!("---\n{}---\n{}", fm_str, items_body)
    } else {
        format!("---\n{}---\n{}\n\n{}", fm_str, desc_body, items_body)
    })
}

pub fn save_task_fs(data_dir: &Path, task: &Task) -> Result<()> {
    let dir = entity_dir(data_dir, "tasks")?;
    let path = item_path(&dir, &task.id);
    let content = serialize_task(task)?;
    fs::write(&path, content).with_context(|| format!("Failed to write task: {}", path.display()))
}

pub fn load_task_fs(data_dir: &Path, id: &str) -> Result<Task> {
    let dir = entity_dir(data_dir, "tasks")?;
    let full_id = resolve_id(&dir, id)?;
    let path = item_path(&dir, &full_id);

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read task: {}", path.display()))?;
    let (fm_str, body) = split_frontmatter(&raw)?;

    let (description, items) = body_to_components(body);

    // Try new format first
    if let Ok(fm) = serde_yaml::from_str::<Frontmatter>(fm_str) {
        let (_, _, stored_id) = fm.parse_znote()?;
        return Ok(Task {
            id: stored_id.to_string(),
            title: extract_title(&fm.title, body.trim_start_matches('\n')),
            description,
            tags: space_string_to_vec(&fm.tags),
            links: space_string_to_vec(&fm.links),
            items,
            created_at: fm.created_at,
            updated_at: fm.updated_at,
        });
    }

    // Fallback to legacy
    #[derive(serde::Deserialize)]
    struct LegacyTaskFrontmatter {
        #[serde(default)]
        title: String,
        #[serde(default)]
        tags: Vec<String>,
        #[serde(default)]
        links: Vec<String>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let fm: LegacyTaskFrontmatter =
        serde_yaml::from_str(fm_str).context("Failed to parse task frontmatter")?;

    Ok(Task {
        id: full_id,
        title: extract_title(&fm.title, body.trim_start_matches('\n')),
        description,
        tags: fm.tags,
        links: fm.links,
        items,
        created_at: fm.created_at,
        updated_at: fm.updated_at,
    })
}

pub fn delete_task_fs(data_dir: &Path, id: &str) -> Result<()> {
    let dir = entity_dir(data_dir, "tasks")?;
    let full_id = resolve_id(&dir, id)?;
    let path = item_path(&dir, &full_id);
    fs::remove_file(&path).with_context(|| format!("Failed to delete task: {}", path.display()))
}

pub fn list_tasks_fs(data_dir: &Path) -> Result<Vec<Task>> {
    let dir = entity_dir(data_dir, "tasks")?;
    load_all_from_dir(&dir, |id| load_task_fs(data_dir, id))
}

// ---------------------------------------------------------------------------
// Shared utility
// ---------------------------------------------------------------------------

/// Walk a directory, collect every `*.md` file, strip the `.md` extension to
/// get the UUID, and call `loader(id)` for each one.
fn load_all_from_dir<T>(dir: &Path, loader: impl Fn(&str) -> Result<T>) -> Result<Vec<T>> {
    let mut items = Vec::new();
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            match loader(stem) {
                Ok(item) => items.push(item),
                Err(e) => eprintln!("Warning: skipping {}: {e}", path.display()),
            }
        }
    }
    Ok(items)
}

pub fn get_entity_type(data_dir: &Path, id: &str) -> Option<&'static str> {
    if let Ok(dir) = entity_dir(data_dir, "notes")
        && resolve_id(&dir, id).is_ok()
    {
        return Some("note");
    }
    if let Ok(dir) = entity_dir(data_dir, "bookmarks")
        && resolve_id(&dir, id).is_ok()
    {
        return Some("bookmark");
    }
    if let Ok(dir) = entity_dir(data_dir, "tasks")
        && resolve_id(&dir, id).is_ok()
    {
        return Some("task");
    }
    None
}

pub fn format_links(data_dir: &Path, links: &[String]) -> Vec<String> {
    links
        .iter()
        .map(|link| {
            if let Some((rel, target)) = link.split_once(':') {
                let entity_type = get_entity_type(data_dir, target).unwrap_or("unknown");
                format!("[{}] {}: {}", entity_type, rel, target)
            } else {
                link.clone()
            }
        })
        .collect()
}

pub fn get_incoming_links(data_dir: &Path, target_id: &str) -> Vec<String> {
    let mut incoming = Vec::new();

    let truncate = |id: &str| {
        if id.len() <= 8 {
            id.to_string()
        } else {
            id[..8].to_string()
        }
    };

    if let Ok(notes) = list_notes(data_dir) {
        for n in notes {
            if n.id.starts_with(target_id) {
                continue;
            }
            for link in &n.links {
                if let Some((rel, target)) = link.split_once(":")
                    && (target_id.starts_with(target) || target.starts_with(target_id))
                {
                    incoming.push(format!("[note] {}: {}", rel, truncate(&n.id)));
                }
            }
        }
    }
    if let Ok(bms) = list_bookmarks(data_dir) {
        for b in bms {
            if b.id.starts_with(target_id) {
                continue;
            }
            for link in &b.links {
                if let Some((rel, target)) = link.split_once(":")
                    && (target_id.starts_with(target) || target.starts_with(target_id))
                {
                    incoming.push(format!("[bookmark] {}: {}", rel, truncate(&b.id)));
                }
            }
        }
    }
    if let Ok(tasks) = list_tasks(data_dir) {
        for t in tasks {
            if t.id.starts_with(target_id) {
                continue;
            }
            for link in &t.links {
                if let Some((rel, target)) = link.split_once(":")
                    && (target_id.starts_with(target) || target.starts_with(target_id))
                {
                    incoming.push(format!("[task] {}: {}", rel, truncate(&t.id)));
                }
            }
        }
    }

    incoming.sort();
    incoming.dedup();
    incoming
}

// ---------------------------------------------------------------------------
// Dolt Wrappers
// ---------------------------------------------------------------------------
pub fn save_note(data_dir: &std::path::Path, note: &Note) -> Result<()> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        db.save_note(note)
    } else {
        crate::storage::save_note_fs(data_dir, note)
    }
}
pub fn load_note(data_dir: &std::path::Path, id: &str) -> Result<Note> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        let dir = entity_dir(data_dir, "notes")?;
    let full_id = resolve_id(&dir, id)?;
    db.load_note(&full_id)
    } else {
        crate::storage::load_note_fs(data_dir, id)
    }
}
pub fn delete_note(data_dir: &std::path::Path, id: &str) -> Result<()> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        let dir = entity_dir(data_dir, "notes")?;
    let full_id = resolve_id(&dir, id)?;
    db.delete_note(&full_id)
    } else {
        crate::storage::delete_note_fs(data_dir, id)
    }
}
pub fn list_notes(data_dir: &std::path::Path) -> Result<Vec<Note>> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        db.list_notes()
    } else {
        crate::storage::list_notes_fs(data_dir)
    }
}

pub fn save_bookmark(data_dir: &std::path::Path, bookmark: &Bookmark) -> Result<()> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        db.save_bookmark(bookmark)
    } else {
        crate::storage::save_bookmark_fs(data_dir, bookmark)
    }
}
pub fn load_bookmark(data_dir: &std::path::Path, id: &str) -> Result<Bookmark> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        let dir = entity_dir(data_dir, "bookmarks")?;
    let full_id = resolve_id(&dir, id)?;
    db.load_bookmark(&full_id)
    } else {
        crate::storage::load_bookmark_fs(data_dir, id)
    }
}
pub fn delete_bookmark(data_dir: &std::path::Path, id: &str) -> Result<()> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        let dir = entity_dir(data_dir, "bookmarks")?;
    let full_id = resolve_id(&dir, id)?;
    db.delete_bookmark(&full_id)
    } else {
        crate::storage::delete_bookmark_fs(data_dir, id)
    }
}
pub fn list_bookmarks(data_dir: &std::path::Path) -> Result<Vec<Bookmark>> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        db.list_bookmarks()
    } else {
        crate::storage::list_bookmarks_fs(data_dir)
    }
}

pub fn save_task(data_dir: &std::path::Path, task: &Task) -> Result<()> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        db.save_task(task)
    } else {
        crate::storage::save_task_fs(data_dir, task)
    }
}
pub fn load_task(data_dir: &std::path::Path, id: &str) -> Result<Task> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        let dir = entity_dir(data_dir, "tasks")?;
    let full_id = resolve_id(&dir, id)?;
    db.load_task(&full_id)
    } else {
        crate::storage::load_task_fs(data_dir, id)
    }
}
pub fn delete_task(data_dir: &std::path::Path, id: &str) -> Result<()> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        let dir = entity_dir(data_dir, "tasks")?;
    let full_id = resolve_id(&dir, id)?;
    db.delete_task(&full_id)
    } else {
        crate::storage::delete_task_fs(data_dir, id)
    }
}
pub fn list_tasks(data_dir: &std::path::Path) -> Result<Vec<Task>> {
    if is_dolt_backend() {
        let db = dolt::DoltStorage::new(data_dir);
        // db.init_db();
        db.list_tasks()
    } else {
        crate::storage::list_tasks_fs(data_dir)
    }
}

pub fn sync(data_dir: &std::path::Path) -> Result<()> {
    if !is_dolt_backend() {
        anyhow::bail!("Sync is only supported when using the dolt backend (ZNOTE_STORAGE_BACKEND=dolt)");
    }
    let db = dolt::DoltStorage::new(data_dir);
    // db.init_db();
    db.import_from_fs()
}
