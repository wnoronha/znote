---
name: znote
description: Manage notes, bookmarks, and tasks using a fast CLI tool with YAML frontmatter and WikiLinks.
---

# znote Skill

`znote` is a minimal, high-performance CLI for managing **notes**, **bookmarks**, and **tasks** stored as Markdown files with YAML frontmatter. It supports Obsidian-style `[[WikiLinks]]`, `#hashtags`, and powerful search/graph capabilities.

## User Configuration
The default data directory is `~/.local/share/znote`, but it can be overridden with the `ZNOTE_DIR` environment variable.

## Core Entities
All entities are stored in `{dir}/{type}/{id}.md`.
- **Note**: General markdown content.
- **Bookmark**: A title and a URL.
- **Task**: A title with an optional checklist of items.

## Command Reference

### Note Management
- `znote note add <content> [-T <title>] [--tags #tag] [--links #rel]`: Create a new note. You can optionally omit title and place a `# Header` inside `<content>`.
- `znote note list`: List all notes.
- `znote note view <id>`: View note content and metadata.
- `znote note update <id> [--title "New"] [--content "New"] [--tags "#new"] [--links "#new"]`: Update fields.
- `znote note edit <id>`: Open note in `$EDITOR`.
- `znote note delete <id>`: Delete a note.

### Bookmark Management
- `znote bookmark add <url> [-T <title>] [--tags #tag] [--links #rel]`: Create a new bookmark. Title is auto-extracted from url if omitted.
- `znote bookmark list`: List all bookmarks.
- `znote bookmark view <id>`: View bookmark.
- `znote bookmark update <id> [--url "new-url"]`: Update bookmark.
- `znote bookmark delete <id>`: Delete a bookmark.

### Task Management
- `znote task add <content> [-T <title>] [--tags #tag] [--links #rel]`: Create a new task. You can optionally omit title and place a `# Header` inside `<content>`.
- `znote task list`: List all tasks.
- `znote task view <id>`: View task and checklist.
- `znote task item <task_id> add <text>`: Add item to task checklist.
- `znote task item <task_id> check <index>`: Mark item as done.
- `znote task item <task_id> uncheck <index>`: Mark item as pending.
- `znote task item <task_id> update <index> [--text "New"]`: Update item text.
- `znote task item <task_id> remove <index>`: Remove item.

### Search & Discovery
- `znote search rip <args>`: Direct ripgrep search in the data directory.
- `znote search query <expr>`: Boolean logic search (e.g., `tag:rust AND type:note`).
- `znote graph show`: Print graph nodes and edges.
- `znote graph mermaid`: Output Mermaid.js diagram.

### System
- `znote validate frontmatter`: Check and repair YAML/links.
- `znote config show`: Show current configuration.
- `znote completions <shell>`: Generate completion scripts.

## Conventions
- **IDs**: Use the 8-character UUID prefix to target any entity.
- **Frontmatter**: Files use a unified `znote: <type>/v1/<id>` field.
- **Tags & Links**: Space-separated strings (e.g., `tags: #work #done`).
- **WikiLinks**: Use `[[target_id]]` or `![[target_id#section]]` for embedding/linking.

