# Git Commit Message Format

We follow the Conventional Commits specification. This leads to more readable messages that are easy to follow when looking through the project history.

## Format

```text
<type>(<scope>): <subject>

<body>

<footer>
```

- `<type>`: the kind of change being made (e.g. `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`).
- `<scope>`: (optional) what part of the code the commit is modifying (e.g. `cli`, `models`, `ui`, `search`).
- `<subject>`: a short summary of the change, written in the imperative mood (e.g. "add feature" instead of "added feature" or "adds feature").
- `<body>`: (optional) detailed explanation of the change and its motivation.
- `<footer>`: (optional) references to related issues or breaking changes.

## Examples

```text
feat(cli): implement config show subcommand

Created `src/handlers/config.rs` with `show` handler to display the current active configuration.
```

```text
fix(ui): resolve deep linking issue on note view

Ensure the react router can handle direct URL access for single notes without redirecting to the list view.
```

```text
docs: update GEMINI.md to reflect recent architecture changes
```
