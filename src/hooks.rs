use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// All lifecycle hook events znote can fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    BeforeAdd,
    AfterAdd,
    BeforeEdit,
    AfterEdit,
    BeforeSave,
    AfterSave,
    BeforeDelete,
    AfterDelete,
    BeforeView,
    AfterView,
    BeforeValidate,
    AfterValidate,
}

impl HookEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            HookEvent::BeforeAdd => "before_add",
            HookEvent::AfterAdd => "after_add",
            HookEvent::BeforeEdit => "before_edit",
            HookEvent::AfterEdit => "after_edit",
            HookEvent::BeforeSave => "before_save",
            HookEvent::AfterSave => "after_save",
            HookEvent::BeforeDelete => "before_delete",
            HookEvent::AfterDelete => "after_delete",
            HookEvent::BeforeView => "before_view",
            HookEvent::AfterView => "after_view",
            HookEvent::BeforeValidate => "before_validate",
            HookEvent::AfterValidate => "after_validate",
        }
    }

    /// `before_*` hooks abort the operation on non-zero exit.
    pub fn is_blocking(self) -> bool {
        matches!(
            self,
            HookEvent::BeforeAdd
                | HookEvent::BeforeEdit
                | HookEvent::BeforeSave
                | HookEvent::BeforeDelete
                | HookEvent::BeforeView
                | HookEvent::BeforeValidate
        )
    }
}

/// Context passed to a hook script via environment variables.
pub struct HookContext<'a> {
    /// "note", "bookmark", "task"
    pub entity_type: &'a str,
    pub id: Option<&'a str>,
    pub title: Option<&'a str>,
    /// Canonical path to the entity file on disk (if it exists)
    pub path: Option<&'a Path>,
    /// Content of the entity *before* the operation
    pub old_content: Option<&'a str>,
    /// Content of the entity *after* the operation (or what will be written)
    pub new_content: Option<&'a str>,
}

/// Resolve hooks directory: `{data_dir}/hooks/`
pub fn hooks_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("hooks")
}

/// Write content to a named temp file and return its path.
fn write_temp(prefix: &str, content: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("znote_{}_{}.md", prefix, std::process::id()));
    fs::write(&path, content).context("writing hook temp file")?;
    Ok(path)
}

/// Run a hook script if it exists.
///
/// - `before_*` hooks:  if the script exits non-zero the error is propagated,
///   giving the hook the power to abort the operation.
/// - `after_*` hooks:   exit code is ignored (fire-and-forget).
///
/// If no script exists the call is a no-op.
pub fn run(data_dir: &Path, event: HookEvent, ctx: &HookContext<'_>) -> Result<()> {
    let script = hooks_dir(data_dir).join(format!("{}.sh", event.as_str()));
    if !script.exists() {
        return Ok(());
    }

    // Write old/new content to temp files so the hook can diff them.
    let old_path = ctx.old_content.map(|c| write_temp("old", c)).transpose()?;
    let new_path = ctx.new_content.map(|c| write_temp("new", c)).transpose()?;

    let mut cmd = Command::new("sh");
    cmd.arg(&script);

    // --- Core env vars ---
    cmd.env("ZNOTE_EVENT", event.as_str());
    cmd.env("ZNOTE_TYPE", ctx.entity_type);
    cmd.env("ZNOTE_ID", ctx.id.unwrap_or(""));
    cmd.env("ZNOTE_TITLE", ctx.title.unwrap_or(""));
    cmd.env(
        "ZNOTE_PATH",
        ctx.path
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
    );
    cmd.env("ZNOTE_DATA_DIR", data_dir.to_string_lossy().as_ref());

    // --- Content paths ---
    if let Some(ref p) = old_path {
        cmd.env("ZNOTE_OLD_PATH", p.to_string_lossy().as_ref());
    }
    if let Some(ref p) = new_path {
        cmd.env("ZNOTE_NEW_PATH", p.to_string_lossy().as_ref());
    }

    let status = cmd
        .status()
        .context(format!("Failed to run hook: {}", script.display()))?;

    // Clean up temp files
    if let Some(ref p) = old_path {
        let _ = fs::remove_file(p);
    }
    if let Some(ref p) = new_path {
        let _ = fs::remove_file(p);
    }

    if event.is_blocking() && !status.success() {
        anyhow::bail!(
            "Hook '{}' aborted the operation (exit code: {:?})",
            event.as_str(),
            status.code()
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_hooks_dir(tmp: &Path, hook_name: &str, script_body: &str) {
        let hooks = tmp.join("hooks");
        fs::create_dir_all(&hooks).unwrap();
        let script = hooks.join(format!("{}.sh", hook_name));
        fs::write(&script, format!("#!/bin/sh\n{}", script_body)).unwrap();
        // Make executable
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn ctx<'a>(entity_type: &'a str) -> HookContext<'a> {
        HookContext {
            entity_type,
            id: None,
            title: None,
            path: None,
            old_content: None,
            new_content: None,
        }
    }

    #[test]
    fn test_no_hook_script_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        // hooks dir doesn't exist — should silently succeed
        let c = ctx("note");
        assert!(run(tmp.path(), HookEvent::AfterSave, &c).is_ok());
    }

    #[test]
    fn test_after_hook_ignores_nonzero_exit() {
        let tmp = tempfile::tempdir().unwrap();
        make_hooks_dir(tmp.path(), "after_save", "exit 42");
        let c = ctx("note");
        assert!(run(tmp.path(), HookEvent::AfterSave, &c).is_ok());
    }

    #[test]
    fn test_before_hook_zero_exit_succeeds() {
        let tmp = tempfile::tempdir().unwrap();
        make_hooks_dir(tmp.path(), "before_save", "exit 0");
        let c = ctx("note");
        assert!(run(tmp.path(), HookEvent::BeforeSave, &c).is_ok());
    }

    #[test]
    fn test_before_hook_nonzero_exit_aborts() {
        let tmp = tempfile::tempdir().unwrap();
        make_hooks_dir(tmp.path(), "before_save", "exit 1");
        let c = ctx("note");
        let result = run(tmp.path(), HookEvent::BeforeSave, &c);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("aborted the operation")
        );
    }

    #[test]
    fn test_env_vars_are_set() {
        let tmp = tempfile::tempdir().unwrap();
        let marker = tmp.path().join("env_ok");
        // Script checks that ZNOTE_EVENT is "before_save" and writes a marker file
        make_hooks_dir(
            tmp.path(),
            "before_save",
            &format!(
                r#"[ "$ZNOTE_EVENT" = "before_save" ] && [ "$ZNOTE_TYPE" = "note" ] && touch "{}""#,
                marker.display()
            ),
        );
        let c = ctx("note");
        run(tmp.path(), HookEvent::BeforeSave, &c).unwrap();
        assert!(marker.exists(), "hook env vars were not set correctly");
    }

    #[test]
    fn test_old_new_temp_files_created_and_cleaned_up() {
        let tmp = tempfile::tempdir().unwrap();
        let marker = tmp.path().join("paths_ok");
        make_hooks_dir(
            tmp.path(),
            "after_save",
            &format!(
                r#"[ -f "$ZNOTE_OLD_PATH" ] && [ -f "$ZNOTE_NEW_PATH" ] && touch "{}""#,
                marker.display()
            ),
        );
        let c = HookContext {
            entity_type: "note",
            id: Some("abc"),
            title: Some("Test"),
            path: None,
            old_content: Some("old content"),
            new_content: Some("new content"),
        };
        run(tmp.path(), HookEvent::AfterSave, &c).unwrap();
        assert!(marker.exists(), "temp files were not passed to hook");
    }

    #[test]
    fn test_hook_event_is_blocking() {
        assert!(HookEvent::BeforeSave.is_blocking());
        assert!(HookEvent::BeforeDelete.is_blocking());
        assert!(!HookEvent::AfterSave.is_blocking());
        assert!(!HookEvent::AfterDelete.is_blocking());
    }
}
