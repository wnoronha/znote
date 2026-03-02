# Search

znote provides two complementary search commands under `znote search`:

| Command | Purpose |
|---|---|
| `search rip` | Full-text search using ripgrep |
| `search query` | Structured filter using a boolean expression language |

Both commands are scoped to your data directory (`-d` / `ZNOTE_DIR`).

---

## `search rip` — Full-Text Search

Runs [ripgrep](https://github.com/BurntSushi/ripgrep) (`rg`) directly inside your data directory. All arguments are passed through to `rg` unchanged, giving you the full power of ripgrep.

```
znote search rip <PATTERN> [RG_FLAGS...]
```

**Examples:**

```bash
# Find any file containing "ownership"
znote search rip "ownership"

# Case-insensitive search
znote search rip -i "rust"

# List only filenames (no line content)
znote search rip "#rust" -l

# Search with a regex
znote search rip "url:.*github\.com"

# Restrict to notes only
znote search rip "learning" notes/
```

Ripgrep must be installed and available in `$PATH`. On Ubuntu/Debian:

```bash
sudo apt-get install ripgrep
```

Or using cargo:

```bash
cargo install ripgrep
```

---

## `search query` — Structured Filter

Filter entities by their metadata using a composable boolean expression. Results are displayed in the same grouped list format as `note list`, `bookmark list`, and `task list`, with tags and outgoing links shown.

```
znote search query "<EXPRESSION>"
```

### Filters

| Filter | Matches |
|---|---|
| `tag:<value>` | Entity has the tag `#<value>` |
| `link:<relationship>` | Entity has an outgoing link with this relationship label |
| `type:<kind>` | Entity is of type `note`, `bookmark`, or `task` |

Tag values are matched without the leading `#` — `tag:rust` matches `#rust`.

### Operators

Operators in precedence order (lowest to highest):

| Operator | Meaning | Example |
|---|---|---|
| `OR` | Union — either condition | `tag:rust OR tag:docs` |
| `AND` | Intersection — both conditions | `tag:rust AND tag:learning` |
| `NOT` | Complement — negation (prefix) | `NOT tag:docs` |
| `( )` | Grouping — overrides precedence | `(tag:a OR tag:b) AND tag:c` |

`AND` binds more tightly than `OR`, matching standard boolean precedence.

### Examples

```bash
# All entities tagged #rust
znote search query "tag:rust"

# Notes tagged both #rust and #learning
znote search query "tag:rust AND tag:learning AND type:note"

# Bookmarks tagged #rust but not #docs
znote search query "type:bookmark AND tag:rust AND NOT tag:docs"

# Notes or bookmarks (not tasks) related to rust
znote search query "(type:note OR type:bookmark) AND tag:rust"

# Entities with a 'website' outgoing link that are also tagged #rust
znote search query "link:website AND tag:rust"

# Everything *without* a docs link
znote search query "NOT link:docs"

# Complex grouping
znote search query "tag:rust AND (tag:critical OR tag:new)"
```

### Output Format

Results are grouped by entity type and displayed in list format:

```
Notes
• 886a5bea Understanding Rust Ownership  #rust #learning  ↗ website:c1a2b3d4, blocked_by:e5f6g7h8
Bookmarks
• c1a2b3d4 Rust Documentation  https://doc.rust-lang.org/  #rust #docs
Tasks
• e5f6g7h8 Project: znote Core [1/3]  #coding #rust  ↗ docs:c1a2b3d4
```

Each row shows:
- Short ID (first 8 chars)
- Title
- Task progress `[done/total]` (tasks only)
- Tags
- Outgoing link relationships (`↗`)

### Error Messages

| Error | Meaning |
|---|---|
| `Unexpected token 'foo'. Did you mean 'tag:foo'?` | Bare words are not valid — use `tag:`, `link:`, or `type:` |
| `Unknown filter 'xyz'. Use: tag:, link:, type:` | Unrecognised filter kind |
| `Expected closing ')'` | Missing closing parenthesis |
| `ripgrep ('rg') is not installed` | Install ripgrep to use search |
