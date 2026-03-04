# znote — Agent Reference

## Overview
`znote` is a minimal, high-performance CLI (Rust 2024) for **notes**, **bookmarks**, and **tasks** stored as Markdown with YAML frontmatter.

## Tech Stack
`clap` (CLI), `serde` (JSON/YAML), `uuid` (v4), `chrono` (UTC), `walkdir` (FS), `colored` (stdout), `anyhow` (Errors), `regex`.

## Project Layout
- `src/main.rs`: Entry point & dispatch.
- `src/commands/mod.rs`: `clap` structs/enums.
- `src/handlers/`: Logic (note, bookmark, task, search, graph, validate, completions). Includes `read_stdin` helper in `mod.rs`.
- `src/models/`: Structs (note, bookmark, task) with lifecycle logic.
- `src/storage/`: persistence to `{dir}/{type}/{id}.md`.
- `examples/`: Reference data for development.

## CLI Commands
- `note | bookmark | task`: `add | list | view | update | edit | delete <id>` (supports `-` for stdin in content/description)
- `task item`: `add | check | uncheck | update | remove`
- `search`: `rip <args>` (ripgrep) | `query <expr>` (boolean logic: `tag:x AND type:y`)
- `graph`: `show | dot | json | mermaid` (flags: `--tag`, `-y/--entity-type`, `--without-isolated`)
- `validate`: `frontmatter` (checks/repairs YAML and link formats)
- `config`: `show`
- `completions`: `bash | zsh | fish | powershell`

## Data Model & Conventions
- **IDs**: UUIDv4 strings. Stored in `znote: <type>/v1/<id>` field.
- **Tags**: Space-separated strings using `#hashtag` convention.
- **Links**: Space-separated relationships in `rel:id` format. WikiLinks supported in content.
- **Hooks**: Scripts in `{dir}/hooks/` named `<event>.sh` (e.g., `after_save.sh`).
- **Development**: Follow TDD (`src/tests.rs`). Branch off `main`. 
  - Use `make lint && make test` for validation.
  - Use `make dev` + `make ui-watch` (separate terminals) for auto-reloading development.
- **Commits**: Follow `docs/git_commit_message_format.md`. Use relative paths to avoid PII leak. NEVER include absolute paths or local system information in commit messages.

## Development & CI Best Practices
- **CI Dependencies**: Ensure `ripgrep` (`apt-get install ripgrep`) and Node.js (`ui/dist/` build) are available before running Rust tests. The binary embeds UI assets at compile-time.
- **Environment Variables**: Avoid `unsafe { env::set_var(...) }` in tests as they are unstable in parallel CI environments. Prefer passing configuration (like tokens) via `AppState`.
- **String Slicing**: Always use the `truncate_id` helper or check bounds before slicing IDs (e.g., `&id[..8]`) to prevent panics on short IDs.
- **Formatting & Linting**: Always run `make lint && make test` locally before pushing. CI enforces strict Clippy (`-D warnings`) and Rustfmt checks.
- **UI State**: In React components, avoid synchronous `setState` inside `useEffect` to satisfy strict ESLint rules (`react-hooks/set-state-in-effect`).

## Current State
- All CRUD and Graph operations are functional.
- Web UI (React/Vite) integrated with token-based authentication and live storage connection.
- Mermaid.js output supported with custom styling.
- Shell completions and ID autocompletion integrated.
- Support for reading content from stdin via `-` argument for `add` and `update` commands.
- Data directory defaults to `~/.local/share/znote/`, override with `ZNOTE_DIR`.
