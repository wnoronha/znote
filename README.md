# znote

A minimal, high-performance CLI tool for managing **notes**, **bookmarks**, and **tasks** as plain Markdown files.

## Features

- Plain Markdown files with unified, self-describing frontmatter (`znote: type/v1/id`)
- Scripting-friendly format with space-separated tags and links
- Wiki links (`[[Target]]`) and embeds (`![[Target#Section]]`) à la Obsidian
- Composable search with a boolean expression language
- Full-text search powered by ripgrep
- Lifecycle hooks for git sync, webhooks, linting, and more
- Tag-based organisation with outgoing/incoming link tracking
- Graph visualization with Mermaid, DOT, and JSON exports

## Installation

```bash
git clone https://github.com/you/znote
cd znote
make release        # builds to target/release/znote
```

Add `target/release/znote` to your `$PATH`, or use `make build` for a dev build.

**Requirements:** Rust 1.75+. For search: [ripgrep](https://github.com/BurntSushi/ripgrep).

## Agent Bootstrap

If you are an AI agent, you can quickly bootstrap `znote` and generate a `SKILL.md` with the following command:

```bash
# 1. Download and extract the latest binary (Linux x86_64 example)
curl -L https://github.com/wnoronha/znote/releases/latest/download/znote-x86_64-unknown-linux-gnu.tar.gz | tar xz

# 2. Generate the full SKILL.md
./znote agent skill > SKILL.md
```

*Note: Replace `x86_64-unknown-linux-gnu` with your specific platform (e.g., `aarch64-apple-darwin` for Apple Silicon) as needed.*

## Quick Start

```bash
# Notes
znote note add "Some content here" -T "My first note" -t rust,learning
znote note list
znote note view <id>
znote note edit <id>

# Bookmarks
znote bookmark add "https://doc.rust-lang.org" -T "Rust Docs" -t rust,docs
znote bookmark list

# Tasks
znote task add "Ship v1" -t project
znote task item add <id> "Write tests"
znote task item check <id> 1

# Search
znote search query "(tag:rust OR tag:docs) AND NOT type:task"

# Graph
znote graph                 # text overview
znote graph mermaid         # mermaid.js format
znote graph --tag rust      # filtered graph
```

## Data Directory

By default, data lives in `~/.local/share/znote/`. Override with:

```bash
znote -d /path/to/dir <command>
# or
export ZNOTE_DIR=/path/to/dir
```

## Commands

```
znote [--data-dir/-d <path>]
  note      add | list | view <id> | update <id> | edit <id> | delete <id>
  bookmark  add | list | view <id> | update <id> | edit <id> | delete <id>
  task      add | list | view <id> | update <id> | edit <id> | delete <id>
            item add <task-id> | check <task-id> <n> | uncheck | update | remove
  search    rip <pattern> [rg-flags]
            query "<expression>"
  graph     show | dot | json | mermaid [--tag <t>] [--entity-type <y>]
  serve     [-p <port>] [-H <host>]
  validate  frontmatter
  config    show
  completions <shell>
```

## Wiki Links & Embeds

Inside note/bookmark/task content, you can use Obsidian-compatible syntax:

```markdown
[[target-id]]                  # link to entity
[[target-id|Display Name]]     # link with alias
![[target-id]]                 # embed full content
![[target-id#Section Header]]  # embed a specific section
```

Targets are resolved by UUID prefix — the first 8 characters are sufficient.
Embeds recurse one level deep.

For bookmarks, embedding without a section shows only the title and URL.

## Documentation

| Doc | Description |
|---|---|
| [docs/search.md](docs/search.md) | Full search reference with examples |
| [docs/hooks.md](docs/hooks.md) | Lifecycle hooks: events, env vars, and examples |
| [docs/git_commit_message_format.md](docs/git_commit_message_format.md) | Commit message conventions |

## Development

```bash
make build    # build debug binary + UI assets
make test     # run all tests (Rust + API)
make lint     # cargo clippy + ui lint
make release  # build release binary + UI assets
make dev      # auto-reload backend (requires cargo-watch)
make ui-watch # auto-rebuild UI assets
```

## Credits

This project stands on the shoulders of giants:

- [steveyegge/beads](https://github.com/steveyegge/beads) - For the inspiration and conceptual foundation for the agent-centric tooling.
- [dolthub/dolt](https://github.com/dolthub/dolt) - The SQL database that provides Git-like versioning for our data storage backend.
- [perstarkse/minne](https://github.com/perstarkse/minne) - The graph visualization aesthetics and physics-inspired navigation that influenced the znote Knowledge Map.
