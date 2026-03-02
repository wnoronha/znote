#[cfg(test)]
mod tests {
    use crate::models::bookmark::Bookmark;
    use crate::models::note::Note;
    use crate::models::task::Task;
    use crate::storage;

    // -----------------------------------------------------------------------
    // Model constructor tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_note() {
        let title = "Test Note".to_string();
        let content = "This is a test note content.".to_string();
        let tags = vec!["#test".to_string(), "#tdd".to_string()];

        let note = Note::new(title.clone(), content.clone(), tags.clone());

        assert_eq!(note.title, title);
        assert_eq!(note.content, content);
        assert_eq!(note.tags, tags);
        assert!(!note.id.is_empty());
    }

    #[test]
    fn test_create_bookmark() {
        let url = "https://rust-lang.org".to_string();
        let title = "Rust Programming Language".to_string();
        let description = "The official website for Rust.".to_string();
        let tags = vec!["#rust".to_string(), "#lang".to_string()];

        let bookmark = Bookmark::new(
            url.clone(),
            title.clone(),
            Some(description.clone()),
            tags.clone(),
        );

        assert_eq!(bookmark.url, url);
        assert_eq!(bookmark.title, title);
        assert_eq!(bookmark.description, Some(description));
        assert_eq!(bookmark.tags, tags);
        assert!(!bookmark.id.is_empty());
    }

    #[test]
    fn test_create_task() {
        let title = "Project Tasks".to_string();
        let tags = vec!["#work".to_string()];

        let mut task = Task::new(title.clone(), tags.clone());
        task.add_item("Finish TDD setup".to_string(), vec!["#setup".to_string()]);

        assert_eq!(task.title, title);
        assert_eq!(task.tags, tags);
        assert_eq!(task.items.len(), 1);
        assert_eq!(task.items[0].text, "Finish TDD setup");
        assert_eq!(task.items[0].tags, vec!["#setup".to_string()]);
        assert!(!task.items[0].completed);
    }

    // -----------------------------------------------------------------------
    // Storage round-trip tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_note_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let note = Note::new(
            "Round-trip Note".to_string(),
            "Some content here.".to_string(),
            vec!["#storage".to_string(), "#test".to_string()],
        );
        let id = note.id.clone();

        storage::save_note(dir.path(), &note).unwrap();

        let file_path = dir.path().join("notes").join(format!("{id}.md"));
        assert!(file_path.exists(), "expected file {}", file_path.display());
        let raw = std::fs::read_to_string(&file_path).unwrap();
        assert!(
            raw.contains("znote:"),
            "znote field must appear in frontmatter"
        );
        assert!(
            raw.starts_with("---\n"),
            "must start with frontmatter fence"
        );

        let loaded = storage::load_note(dir.path(), &id).unwrap();
        assert_eq!(loaded.id, note.id);
        assert_eq!(loaded.title, note.title);
        assert_eq!(loaded.content, note.content);
        assert_eq!(loaded.tags, note.tags);
        assert_eq!(loaded.created_at.timestamp(), note.created_at.timestamp());
    }

    #[test]
    fn test_note_delete() {
        let dir = tempfile::tempdir().unwrap();
        let note = Note::new("To Delete".to_string(), "bye".to_string(), vec![]);
        let id = note.id.clone();

        storage::save_note(dir.path(), &note).unwrap();
        storage::delete_note(dir.path(), &id).unwrap();

        let file_path = dir.path().join("notes").join(format!("{id}.md"));
        assert!(!file_path.exists(), "file should be gone after delete");
    }

    #[test]
    fn test_list_notes() {
        let dir = tempfile::tempdir().unwrap();
        let n1 = Note::new("Alpha".to_string(), "a".to_string(), vec![]);
        let n2 = Note::new("Beta".to_string(), "b".to_string(), vec![]);

        storage::save_note(dir.path(), &n1).unwrap();
        storage::save_note(dir.path(), &n2).unwrap();

        let mut notes = storage::list_notes(dir.path()).unwrap();
        notes.sort_by(|a, b| a.title.cmp(&b.title));
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].title, "Alpha");
        assert_eq!(notes[1].title, "Beta");
    }

    #[test]
    fn test_bookmark_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let bm = Bookmark::new(
            "https://example.com".to_string(),
            "Example".to_string(),
            Some("A description".to_string()),
            vec!["#web".to_string()],
        );
        let id = bm.id.clone();

        storage::save_bookmark(dir.path(), &bm).unwrap();

        let file_path = dir.path().join("bookmarks").join(format!("{id}.md"));
        assert!(file_path.exists());
        let raw = std::fs::read_to_string(&file_path).unwrap();
        assert!(
            raw.contains("znote:"),
            "znote field must appear in frontmatter"
        );

        let loaded = storage::load_bookmark(dir.path(), &id).unwrap();
        assert_eq!(loaded.id, bm.id);
        assert_eq!(loaded.url, bm.url);
        assert_eq!(loaded.title, bm.title);
        assert_eq!(loaded.description, bm.description);
        assert_eq!(loaded.tags, bm.tags);
    }

    #[test]
    fn test_bookmark_no_description() {
        let dir = tempfile::tempdir().unwrap();
        let bm = Bookmark::new(
            "https://example.com".to_string(),
            "No Desc".to_string(),
            None,
            vec![],
        );
        let id = bm.id.clone();

        storage::save_bookmark(dir.path(), &bm).unwrap();
        let loaded = storage::load_bookmark(dir.path(), &id).unwrap();
        assert_eq!(loaded.description, None);
    }

    #[test]
    fn test_task_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut task = Task::new("My Task".to_string(), vec!["#work".to_string()]);
        task.add_item("Item one".to_string(), vec!["#step1".to_string()]);
        task.add_item("Item two".to_string(), vec![]);
        task.items[0].completed = true;
        let id = task.id.clone();

        storage::save_task(dir.path(), &task).unwrap();

        let file_path = dir.path().join("tasks").join(format!("{id}.md"));
        assert!(file_path.exists());
        let raw = std::fs::read_to_string(&file_path).unwrap();
        assert!(
            raw.contains("znote:"),
            "znote field must appear in frontmatter"
        );
        assert!(
            raw.contains("- [x] Item one"),
            "completed item should be [x]"
        );
        assert!(
            raw.contains("- [ ] Item two"),
            "incomplete item should be [ ]"
        );

        let loaded = storage::load_task(dir.path(), &id).unwrap();
        assert_eq!(loaded.id, task.id);
        assert_eq!(loaded.title, task.title);
        assert_eq!(loaded.items.len(), 2);
        assert!(loaded.items[0].completed);
        assert_eq!(loaded.items[0].text, "Item one");
        assert_eq!(loaded.items[0].tags, vec!["#step1".to_string()]);
        assert!(!loaded.items[1].completed);
        assert_eq!(loaded.items[1].text, "Item two");
    }

    #[test]
    fn test_task_delete_and_list() {
        let dir = tempfile::tempdir().unwrap();
        let t1 = Task::new("Task A".to_string(), vec![]);
        let t2 = Task::new("Task B".to_string(), vec![]);

        storage::save_task(dir.path(), &t1).unwrap();
        storage::save_task(dir.path(), &t2).unwrap();
        storage::delete_task(dir.path(), &t1.id).unwrap();

        let tasks = storage::list_tasks(dir.path()).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Task B");
    }

    // -----------------------------------------------------------------------
    // Note handler tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_note_handler_add_and_list() {
        use crate::commands::NoteAddArgs;
        use crate::handlers::note as note_handler;

        let dir = tempfile::tempdir().unwrap();
        let args = NoteAddArgs {
            title: Some("Handler Test".to_string()),
            content: "Hello world".to_string(),
            tags: Some("#handler #test".to_string()),
            links: None,
        };

        note_handler::add(dir.path(), &args).unwrap();

        let notes = storage::list_notes(dir.path()).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Handler Test");
        assert_eq!(notes[0].content, "Hello world");
        assert_eq!(
            notes[0].tags,
            vec!["#handler".to_string(), "#test".to_string()]
        );
    }

    #[test]
    fn test_note_handler_view() {
        use crate::commands::NoteAddArgs;
        use crate::handlers::note as note_handler;

        let dir = tempfile::tempdir().unwrap();
        let args = NoteAddArgs {
            title: Some("Viewable Note".to_string()),
            content: "View me".to_string(),
            tags: None,
            links: None,
        };
        note_handler::add(dir.path(), &args).unwrap();

        let notes = storage::list_notes(dir.path()).unwrap();
        let id = notes[0].id.clone();

        note_handler::view(dir.path(), &id).unwrap();
    }

    #[test]
    fn test_note_handler_update() {
        use crate::commands::{NoteAddArgs, UpdateArgs};
        use crate::handlers::note as note_handler;

        let dir = tempfile::tempdir().unwrap();
        let add_args = NoteAddArgs {
            title: Some("Original".to_string()),
            content: "Old content".to_string(),
            tags: None,
            links: None,
        };
        note_handler::add(dir.path(), &add_args).unwrap();

        let notes = storage::list_notes(dir.path()).unwrap();
        let id = notes[0].id.clone();

        let update_args = UpdateArgs {
            id: id.clone(),
            title: Some("Updated Title".to_string()),
            content: Some("New content".to_string()),
            url: None,
            tags: Some("#updated".to_string()),
            links: None,
        };
        note_handler::update(dir.path(), &update_args).unwrap();

        let updated = storage::load_note(dir.path(), &id).unwrap();
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.content, "New content");
        assert_eq!(updated.tags, vec!["#updated".to_string()]);
    }

    #[test]
    fn test_note_handler_delete() {
        use crate::commands::NoteAddArgs;
        use crate::handlers::note as note_handler;

        let dir = tempfile::tempdir().unwrap();
        let args = NoteAddArgs {
            title: Some("To Delete".to_string()),
            content: "".to_string(),
            tags: None,
            links: None,
        };
        note_handler::add(dir.path(), &args).unwrap();

        let notes = storage::list_notes(dir.path()).unwrap();
        let id = notes[0].id.clone();

        note_handler::delete(dir.path(), &id).unwrap();

        let remaining = storage::list_notes(dir.path()).unwrap();
        assert!(remaining.is_empty());
    }

    // -----------------------------------------------------------------------
    // Bookmark handler tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_bookmark_handler_add_and_list() {
        use crate::commands::BookmarkAddArgs;
        use crate::handlers::bookmark as bm_handler;

        let dir = tempfile::tempdir().unwrap();
        let args = BookmarkAddArgs {
            title: Some("Rust Lang".to_string()),
            url: "https://rust-lang.org".to_string(),
            tags: Some("#rust #lang".to_string()),
            links: None,
        };

        bm_handler::add(dir.path(), &args).unwrap();

        let bookmarks = storage::list_bookmarks(dir.path()).unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].title, "Rust Lang");
        assert_eq!(bookmarks[0].url, "https://rust-lang.org");
        assert_eq!(
            bookmarks[0].tags,
            vec!["#rust".to_string(), "#lang".to_string()]
        );
    }

    #[test]
    fn test_bookmark_handler_view() {
        use crate::commands::BookmarkAddArgs;
        use crate::handlers::bookmark as bm_handler;

        let dir = tempfile::tempdir().unwrap();
        let args = BookmarkAddArgs {
            title: Some("Example".to_string()),
            url: "https://example.com".to_string(),
            tags: None,
            links: None,
        };
        bm_handler::add(dir.path(), &args).unwrap();

        let bookmarks = storage::list_bookmarks(dir.path()).unwrap();
        let id = bookmarks[0].id.clone();
        bm_handler::view(dir.path(), &id).unwrap();
    }

    #[test]
    fn test_bookmark_handler_update() {
        use crate::commands::{BookmarkAddArgs, UpdateArgs};
        use crate::handlers::bookmark as bm_handler;

        let dir = tempfile::tempdir().unwrap();
        let add_args = BookmarkAddArgs {
            title: Some("Old Title".to_string()),
            url: "https://old.com".to_string(),
            tags: None,
            links: None,
        };
        bm_handler::add(dir.path(), &add_args).unwrap();

        let bookmarks = storage::list_bookmarks(dir.path()).unwrap();
        let id = bookmarks[0].id.clone();

        let update_args = UpdateArgs {
            id: id.clone(),
            title: Some("New Title".to_string()),
            content: None,
            url: Some("https://new.com".to_string()),
            tags: Some("#updated".to_string()),
            links: None,
        };
        bm_handler::update(dir.path(), &update_args).unwrap();

        let updated = storage::load_bookmark(dir.path(), &id).unwrap();
        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.url, "https://new.com");
        assert_eq!(updated.tags, vec!["#updated".to_string()]);
    }

    #[test]
    fn test_bookmark_handler_delete() {
        use crate::commands::BookmarkAddArgs;
        use crate::handlers::bookmark as bm_handler;

        let dir = tempfile::tempdir().unwrap();
        let args = BookmarkAddArgs {
            title: Some("To Delete".to_string()),
            url: "https://delete.me".to_string(),
            tags: None,
            links: None,
        };
        bm_handler::add(dir.path(), &args).unwrap();

        let bookmarks = storage::list_bookmarks(dir.path()).unwrap();
        let id = bookmarks[0].id.clone();

        bm_handler::delete(dir.path(), &id).unwrap();

        let remaining = storage::list_bookmarks(dir.path()).unwrap();
        assert!(remaining.is_empty());
    }

    // -----------------------------------------------------------------------
    // Task handler tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_task_handler_add_and_list() {
        use crate::commands::TaskAddArgs;
        use crate::handlers::task as task_handler;

        let dir = tempfile::tempdir().unwrap();
        let args = TaskAddArgs {
            title: Some("My Sprint".to_string()),
            content: "".to_string(),
            tags: Some("#work #sprint".to_string()),
            links: None,
        };

        task_handler::add(dir.path(), &args).unwrap();

        let tasks = storage::list_tasks(dir.path()).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "My Sprint");
        assert_eq!(
            tasks[0].tags,
            vec!["#work".to_string(), "#sprint".to_string()]
        );
        assert!(tasks[0].items.is_empty());
    }

    #[test]
    fn test_task_handler_view() {
        use crate::commands::TaskAddArgs;
        use crate::handlers::task as task_handler;

        let dir = tempfile::tempdir().unwrap();
        let args = TaskAddArgs {
            title: Some("View Task".to_string()),
            content: "".to_string(),
            tags: None,
            links: None,
        };
        task_handler::add(dir.path(), &args).unwrap();

        let tasks = storage::list_tasks(dir.path()).unwrap();
        let id = tasks[0].id.clone();
        task_handler::view(dir.path(), &id).unwrap();
    }

    #[test]
    fn test_task_handler_update() {
        use crate::commands::{TaskAddArgs, UpdateArgs};
        use crate::handlers::task as task_handler;

        let dir = tempfile::tempdir().unwrap();
        let add_args = TaskAddArgs {
            title: Some("Old Task".to_string()),
            content: "".to_string(),
            tags: None,
            links: None,
        };
        task_handler::add(dir.path(), &add_args).unwrap();

        let tasks = storage::list_tasks(dir.path()).unwrap();
        let id = tasks[0].id.clone();

        let update_args = UpdateArgs {
            id: id.clone(),
            title: Some("Renamed Task".to_string()),
            content: None,
            url: None,
            tags: Some("#renamed".to_string()),
            links: None,
        };
        task_handler::update(dir.path(), &update_args).unwrap();

        let updated = storage::load_task(dir.path(), &id).unwrap();
        assert_eq!(updated.title, "Renamed Task");
        assert_eq!(updated.tags, vec!["#renamed".to_string()]);
    }

    #[test]
    fn test_task_handler_delete() {
        use crate::commands::TaskAddArgs;
        use crate::handlers::task as task_handler;

        let dir = tempfile::tempdir().unwrap();
        let args = TaskAddArgs {
            title: Some("Disposable".to_string()),
            content: "".to_string(),
            tags: None,
            links: None,
        };
        task_handler::add(dir.path(), &args).unwrap();

        let tasks = storage::list_tasks(dir.path()).unwrap();
        let id = tasks[0].id.clone();

        task_handler::delete(dir.path(), &id).unwrap();

        let remaining = storage::list_tasks(dir.path()).unwrap();
        assert!(remaining.is_empty());
    }

    // -----------------------------------------------------------------------
    // Task item handler tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_task_item_lifecycle() {
        use crate::commands::{ItemAddArgs, ItemUpdateArgs, TaskAddArgs};
        use crate::handlers::task as task_handler;

        let dir = tempfile::tempdir().unwrap();

        // 1. Create a task
        let add_args = TaskAddArgs {
            title: Some("Sprint".to_string()),
            content: "".to_string(),
            tags: None,
            links: None,
        };
        task_handler::add(dir.path(), &add_args).unwrap();
        let task_id = storage::list_tasks(dir.path()).unwrap()[0].id.clone();

        // 2. Add an item
        let item_add = ItemAddArgs {
            text: "Initial item".to_string(),
            tags: Some("#initial".to_string()),
        };
        task_handler::item_add(dir.path(), &task_id, &item_add).unwrap();

        let task = storage::load_task(dir.path(), &task_id).unwrap();
        assert_eq!(task.items.len(), 1);
        assert_eq!(task.items[0].text, "Initial item");

        // 3. Check item
        task_handler::item_check(dir.path(), &task_id, 1, true).unwrap();
        let task = storage::load_task(dir.path(), &task_id).unwrap();
        assert!(task.items[0].completed);

        // 4. Update item
        let item_update = ItemUpdateArgs {
            index: 1,
            text: Some("Updated item".to_string()),
            tags: None,
        };
        task_handler::item_update(dir.path(), &task_id, &item_update).unwrap();
        let task = storage::load_task(dir.path(), &task_id).unwrap();
        assert_eq!(task.items[0].text, "Updated item");

        // 5. Remove item
        task_handler::item_remove(dir.path(), &task_id, 1).unwrap();
        let task = storage::load_task(dir.path(), &task_id).unwrap();
        assert!(task.items.is_empty());
    }

    // -----------------------------------------------------------------------
    // Validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_frontmatter() {
        use crate::handlers::validate;
        let dir = tempfile::tempdir().unwrap();

        // Should return ok if empty
        assert!(validate::frontmatter(dir.path()).is_ok());

        // Add some good files
        let args = crate::commands::NoteAddArgs {
            title: Some("Valid".to_string()),
            content: "Content".to_string(),
            tags: None,
            links: None,
        };
        crate::handlers::note::add(dir.path(), &args).unwrap();

        // Still Valid
        assert!(validate::frontmatter(dir.path()).is_ok());

        // Introduce a bad file
        let notes_dir = dir.path().join("notes");
        std::fs::write(notes_dir.join("bad.md"), "---\nbad: frontmatter\n---\nbody").unwrap();

        // Now Invalid
        assert!(validate::frontmatter(dir.path()).is_err());
    }

    #[test]
    fn test_validate_timestamps() {
        use crate::handlers::validate;
        let dir = tempfile::tempdir().unwrap();

        let notes_dir = dir.path().join("notes");
        std::fs::create_dir_all(&notes_dir).unwrap();

        let path = notes_dir.join("time.md");
        std::fs::write(&path, "---\ntitle: Time\ntags: []\ncreated_at: 2024-02-27T10:30:00Z\nupdated_at: 2024-02-27T10:00:00Z\n---\nbody").unwrap();

        // Run validation, should repair the file
        assert!(validate::frontmatter(dir.path()).is_ok());

        let note = crate::storage::load_note(dir.path(), "time").unwrap();
        // updated_at should now equal created_at
        assert_eq!(note.updated_at, note.created_at);
    }

    #[test]
    fn test_validate_links() {
        use crate::handlers::validate;
        let dir = tempfile::tempdir().unwrap();

        let notes_dir = dir.path().join("notes");
        std::fs::create_dir_all(&notes_dir).unwrap();

        let path = notes_dir.join("link_err.md");
        std::fs::write(&path, "---\ntitle: Link Test\ntags: []\nlinks: [\"badformat\"]\ncreated_at: 2024-02-27T10:30:00Z\nupdated_at: 2024-02-27T10:30:00Z\n---\nbody").unwrap();

        // Run validation, should fail because link format is missing a colon
        assert!(validate::frontmatter(dir.path()).is_err());

        // Fix the link and reload
        std::fs::write(&path, "---\ntitle: Link Test\ntags: []\nlinks: [\"rel:valid-id\"]\ncreated_at: 2024-02-27T10:30:00Z\nupdated_at: 2024-02-27T10:30:00Z\n---\nbody").unwrap();

        // This should now be OK
        assert!(validate::frontmatter(dir.path()).is_ok());

        let note = crate::storage::load_note(dir.path(), "link_err").unwrap();
        assert_eq!(note.links, vec!["rel:valid-id".to_string()]);
    }

    // -----------------------------------------------------------------------
    // Agent handler tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_agent_skill_handler() {
        use crate::handlers::agent;
        // This just verifies it doesn't panic
        assert!(agent::skill().is_ok());
    }

    // -----------------------------------------------------------------------
    // Serve handler tests (Web UI API)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_clean_url_fallback() {
        use crate::handlers::serve;
        use axum::{
            http::{Request, StatusCode},
            response::Response,
        };
        use std::sync::Arc;
        use tower::util::ServiceExt;

        let dir = tempfile::tempdir().unwrap();
        let state = serve::AppState {
            data_dir: Arc::new(dir.path().to_path_buf()),
            token: None,
        };

        let app = serve::test_router(state);
        let response: Response = app
            .oneshot(
                Request::builder()
                    .uri("/note/886a5bea-1234-4567-8901-23456789abcd")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(headers.get("content-type").unwrap(), "text/html");
    }

    #[tokio::test]
    async fn test_api_authentication() {
        use crate::handlers::serve;
        use axum::{
            http::{Request, StatusCode},
            response::Response,
        };
        use std::sync::Arc;
        use tower::util::ServiceExt;

        let dir = tempfile::tempdir().unwrap();
        let token = "test-token";
        let state = serve::AppState {
            data_dir: Arc::new(dir.path().to_path_buf()),
            token: Some(token.to_string()),
        };

        let app = serve::test_router(state);

        // 1. API request without token should fail
        let response: Response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/notes")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // 2. API request with token should succeed
        let response: Response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/notes?token=test-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // 3. Frontend route should REMAIN PUBLIC even if token is required
        // (This allows the UI to load and show the login dialog)
        let response: Response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/note/some-uuid")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // 4. Test query API
        let response: Response = app
            .oneshot(
                Request::builder()
                    .uri("/api/query?expr=tag:test&token=test-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_graph() {
        use crate::commands::NoteAddArgs;
        use crate::handlers::note as note_handler;
        use crate::handlers::serve;
        use axum::http::Request;
        use std::sync::Arc;
        use tower::util::ServiceExt;

        let dir = tempfile::tempdir().unwrap();

        // 1. Create a note (the target)
        let note1_args = NoteAddArgs {
            title: Some("Target Note".to_string()),
            content: "I am the target".to_string(),
            tags: None,
            links: None,
        };
        note_handler::add(dir.path(), &note1_args).unwrap();
        let target_note = storage::list_notes(dir.path()).unwrap()[0].clone();
        let target_id = target_note.id;
        let target_prefix = if target_id.len() <= 8 {
            target_id.clone()
        } else {
            target_id[..8].to_string()
        };

        // 2. Create another note that links to the first note using a prefix
        let note2_args = NoteAddArgs {
            title: Some("Source Note".to_string()),
            content: "I link to the target".to_string(),
            tags: None,
            links: Some(format!("rel:{}", target_prefix)),
        };
        note_handler::add(dir.path(), &note2_args).unwrap();
        let source_note = storage::list_notes(dir.path())
            .unwrap()
            .into_iter()
            .find(|n| n.title == "Source Note")
            .unwrap();
        let source_id = source_note.id;

        let token = "test-graph-token";
        let state = serve::AppState {
            data_dir: Arc::new(dir.path().to_path_buf()),
            token: Some(token.to_string()),
        };

        let app = serve::test_router(state);

        // 3. Request the graph
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/graph?token={}", token))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 100000)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let nodes = json["nodes"].as_array().unwrap();
        let links = json["links"].as_array().unwrap();

        assert_eq!(nodes.len(), 2);
        assert_eq!(links.len(), 1);

        let link = &links[0];
        assert_eq!(link["source"].as_str().unwrap(), source_id);
        assert_eq!(link["target"].as_str().unwrap(), target_id); // This is the crucial check!
        assert_eq!(link["label"].as_str().unwrap(), "rel");
    }
}
