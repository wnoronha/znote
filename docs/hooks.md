# Hooks

znote supports lifecycle hooks — executable shell scripts that run before or after key operations. Use them to sync to git, post to webhooks, lint content, send notifications, or anything else.

## Hook Directory

Place scripts in `{data_dir}/hooks/`. The default data directory is `~/.local/share/znote/`, so the hooks directory is:

```
~/.local/share/znote/hooks/
```

You can override the data directory with `znote -d /path/to/dir` or `ZNOTE_DIR=/path/to/dir`.

## Naming Convention

Each script is named `<event>.sh` and must be executable (`chmod +x`):

```
hooks/
├── before_add.sh
├── after_add.sh
├── before_edit.sh
├── after_edit.sh
├── before_save.sh
├── after_save.sh
├── before_delete.sh
├── after_delete.sh
├── before_view.sh
├── after_view.sh
├── before_validate.sh
└── after_validate.sh
```

Missing scripts are silently skipped — hooks are fully opt-in.

## Hook Events

| Event | Trigger | Aborts on non-zero? |
|---|---|---|
| `before_add` | Before a new entity is created | ✅ |
| `after_add` | After entity written to disk | ❌ |
| `before_edit` | Before `$EDITOR` opens | ✅ |
| `after_edit` | After editor closes | ❌ |
| `before_save` | Before writing updated entity to disk (update command) | ✅ |
| `after_save` | After entity written to disk (update command) | ❌ |
| `before_delete` | Before entity file is removed | ✅ |
| `after_delete` | After entity file is removed | ❌ |
| `before_view` | Before entity is rendered to stdout | ✅ |
| `after_view` | After entity is rendered | ❌ |
| `before_validate` | Before validate command runs | ✅ |
| `after_validate` | After validate command completes | ❌ |

**`before_*` hooks** can abort the operation by exiting non-zero. znote will print an error and stop.

**`after_*` hooks** are fire-and-forget — their exit code is ignored.

## Environment Variables

Every hook receives the following environment variables:

| Variable | Description |
|---|---|
| `ZNOTE_EVENT` | Name of the hook event (e.g. `"before_save"`) |
| `ZNOTE_TYPE` | Entity type: `"note"`, `"bookmark"`, or `"task"` |
| `ZNOTE_ID` | Full UUID of the entity |
| `ZNOTE_TITLE` | Title of the entity |
| `ZNOTE_PATH` | Absolute path to the entity's `.md` file on disk |
| `ZNOTE_DATA_DIR` | Absolute path to the data directory |
| `ZNOTE_OLD_PATH` | Path to a temp file containing content **before** the change |
| `ZNOTE_NEW_PATH` | Path to a temp file containing content **after** the change |

`ZNOTE_OLD_PATH` and `ZNOTE_NEW_PATH` are only set for hooks where content changes — e.g. `before_save`, `after_save`, `before_delete`, `after_edit`. Do not delete these temp files; znote cleans them up automatically after the hook exits.

## Examples

### Auto-commit to git after every save

```bash
#!/bin/sh
# ~/.local/share/znote/hooks/after_save.sh
cd "$ZNOTE_DATA_DIR"
git add "$ZNOTE_PATH"
git commit -m "znote: update $ZNOTE_TYPE '$ZNOTE_TITLE'"
```

### Abort delete without confirmation

```bash
#!/bin/sh
# ~/.local/share/znote/hooks/before_delete.sh
printf "Delete %s '%s'? [y/N] " "$ZNOTE_TYPE" "$ZNOTE_TITLE"
read -r answer
[ "$answer" = "y" ] || [ "$answer" = "Y" ] || exit 1
```

### Lint markdown content before saving

```bash
#!/bin/sh
# ~/.local/share/znote/hooks/before_save.sh
command -v markdownlint >/dev/null || exit 0
markdownlint "$ZNOTE_NEW_PATH" || exit 1
```

### Diff changes after edit

```bash
#!/bin/sh
# ~/.local/share/znote/hooks/after_edit.sh
if [ -f "$ZNOTE_OLD_PATH" ] && [ -f "$ZNOTE_NEW_PATH" ]; then
    echo "--- Changes to $ZNOTE_TITLE ---"
    diff "$ZNOTE_OLD_PATH" "$ZNOTE_NEW_PATH"
fi
```

### Post to a webhook after adding a bookmark

```bash
#!/bin/sh
# ~/.local/share/znote/hooks/after_add.sh
[ "$ZNOTE_TYPE" = "bookmark" ] || exit 0
curl -s -X POST "https://your-webhook.example.com/znote" \
    -H "Content-Type: application/json" \
    -d "{\"event\": \"$ZNOTE_EVENT\", \"id\": \"$ZNOTE_ID\", \"title\": \"$ZNOTE_TITLE\"}"
```

### Back up entity before deletion

```bash
#!/bin/sh
# ~/.local/share/znote/hooks/before_delete.sh
ARCHIVE="$HOME/.znote-archive"
mkdir -p "$ARCHIVE"
cp "$ZNOTE_OLD_PATH" "$ARCHIVE/${ZNOTE_ID}.md"
echo "Archived to $ARCHIVE/${ZNOTE_ID}.md"
```
