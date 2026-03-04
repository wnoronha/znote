#![allow(dead_code, unused_variables, unused_imports)]
use std::sync::OnceLock;
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use std::path::{Path, PathBuf};
use std::process::Command;
use mysql::prelude::*;
use mysql::{OptsBuilder, Pool};

use crate::models::bookmark::Bookmark;
use crate::models::note::Note;
use crate::models::task::{Task, TaskItem};

/// Abstract wrapper over Dolt CLI.
static DOLT_POOL: OnceLock<Pool> = OnceLock::new();

pub struct DoltStorage {
    pub data_dir: PathBuf,
}

impl DoltStorage {
    fn get_db_name(&self) -> String {
        std::env::var("ZNOTE_DOLT_DB").unwrap_or_else(|_| {
            self.data_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("znote")
                .to_string()
        })
    }

    pub fn start_server(&self) -> Result<()> {
        let host = std::env::var("ZNOTE_DOLT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("ZNOTE_DOLT_PORT").unwrap_or_else(|_| "3306".to_string());
        
        // Check if already running
        if self.get_pool().and_then(|p| p.get_conn().context("Check failed")).is_ok() {
            tracing::debug!("Dolt SQL server already running on {}:{}", host, port);
            return Ok(());
        }

        tracing::info!("Starting Dolt SQL server on {}:{}", host, port);
        let _child = Command::new("dolt")
            .current_dir(&self.data_dir)
            .args(&["sql-server", "--host", &host, "--port", &port])
            .spawn()
            .context("Failed to start dolt sql-server")?;
            
        // Wait a bit for it to start
        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    }

    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
        }
    }

    /// Run a quick Dolt CLI command and expect it to succeed
    fn run_dolt(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("dolt")
            .current_dir(&self.data_dir)
            .args(args)
            .output()
            .with_context(|| format!("Failed to execute dolt {:?}", args))?;

        if !output.status.success() {
            anyhow::bail!(
                "Dolt command failed {:?}: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8(output.stdout)?)
    }

        fn get_pool(&self) -> Result<Pool> {
        if let Some(pool) = DOLT_POOL.get() {
            return Ok(pool.clone());
        }

        tracing::info!("Initializing new MySQL connection pool");
        let host = std::env::var("ZNOTE_DOLT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port: u16 = std::env::var("ZNOTE_DOLT_PORT")
            .unwrap_or_else(|_| "3306".to_string())
            .parse()
            .unwrap_or(3306);
        let user = std::env::var("ZNOTE_DOLT_USER").unwrap_or_else(|_| "root".to_string());
        let pass = std::env::var("ZNOTE_DOLT_PASS").unwrap_or_default();

        let mut builder = OptsBuilder::new()
            .ip_or_hostname(Some(host))
            .tcp_port(port)
            .user(Some(user));

        if !pass.is_empty() {
            builder = builder.pass(Some(pass));
        }
        
        // Use a persistent pool with 1-10 connections
        let opts = builder.pool_opts(mysql::PoolOpts::default()
            .with_constraints(mysql::PoolConstraints::new(1, 10).unwrap()));

        let pool = Pool::new(opts).context("Failed to configure MySQL pool")?;
        let _ = DOLT_POOL.set(pool.clone());
        Ok(pool)
    }



    /// Convert mysql::Value to serde_json::Value
    fn mysql_val_to_json(&self, val: mysql::Value) -> serde_json::Value {
        match val {
            mysql::Value::NULL => serde_json::Value::Null,
            mysql::Value::Bytes(b) => {
                if let Ok(s) = String::from_utf8(b.clone()) {
                    // Dolt returns JSON strings or arrays occasionally
                    if (s.starts_with('[') && s.ends_with(']')) || (s.starts_with('{') && s.ends_with('}')) {
                        if let Ok(parsed) = serde_json::from_str(&s) {
                            return parsed;
                        }
                    }
                    serde_json::Value::String(s)
                } else {
                    serde_json::Value::String(format!("{:?}", b))
                }
            },
            mysql::Value::Int(i) => serde_json::json!(i),
            mysql::Value::UInt(u) => serde_json::json!(u),
            mysql::Value::Float(f) => serde_json::json!(f),
            mysql::Value::Double(d) => serde_json::json!(d),
            mysql::Value::Date(y, m, d, h, min, s, _u) => {
                serde_json::Value::String(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, min, s))
            },
            mysql::Value::Time(neg, d, h, m, s, _u) => {
                let sign = if neg { "-" } else { "" };
                serde_json::Value::String(format!("{}{:02}:{:02}:{:02}", sign, h + (d as u8 * 24), m, s))
            }
        }
    }

    fn row_to_json(&self, mut row: mysql::Row) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for i in 0..row.len() {
            let col = row.columns_ref()[i].name_str().into_owned();
            let val: mysql::Value = row.take(i).unwrap_or(mysql::Value::NULL);
            map.insert(col, self.mysql_val_to_json(val));
        }
        serde_json::Value::Object(map)
    }

    /// Read JSON output from a `dolt sql` command
    pub fn run_sql(&self, query: &str) -> Result<serde_json::Value> {
        let is_select = query.trim_start().to_lowercase().starts_with("select");
        
        match self.get_pool().and_then(|pool| pool.get_conn().context("Connection failed")) {
            Ok(mut conn) => {
                let dbname = self.get_db_name();
                let _ = conn.query_drop(format!("USE `{}`", dbname)); 
                
                tracing::debug!("Executing SQL via MySQL: {}", query);
                let start = std::time::Instant::now();
                if is_select {
                    let result: std::result::Result<Vec<mysql::Row>, _> = conn.query(query);
                    tracing::debug!("SQL took {:?}", start.elapsed());
                    match result {
                        Ok(rows) => {
                            let json_rows: Vec<serde_json::Value> = rows.into_iter().map(|r| self.row_to_json(r)).collect();
                            return Ok(serde_json::json!({"rows": json_rows}));
                        }
                        Err(e) => {
                            tracing::warn!("MySQL query error: {}. Falling back to CLI.", e);
                        }
                    }
                } else {
                    let result = conn.query_drop(query);
                    tracing::debug!("SQL took {:?}", start.elapsed());
                    match result {
                        Ok(_) => return Ok(serde_json::json!({"rows": []})),
                        Err(e) => {
                            tracing::warn!("MySQL exec error: {}. Falling back to CLI.", e);
                        }
                    }
                }
            }
            Err(_) => {
                // Connection failed
            }
        }

        // Fallback to Dolt CLI 
        tracing::debug!("Executing SQL via Dolt CLI: {}", query);
        let start = std::time::Instant::now();
        let output = self.run_dolt(&["sql", "-q", query, "-r", "json"])?;
        tracing::debug!("SQL took {:?}", start.elapsed());
        if output.trim().is_empty() {
            return Ok(serde_json::json!({"rows": []}));
        }
        let mut val: serde_json::Value = serde_json::from_str(&output).context("Failed to parse dolt json output")?;
        if val.get("rows").is_none() && let Some(obj) = val.as_object_mut() {
            obj.insert("rows".to_string(), serde_json::json!([]));
        }
        Ok(val)
    }

    /// Initializes a new Dolt repository if one does not exist,
    /// and ensures the schema for notes, bookmarks, and tasks is defined.
    pub fn add_remote(&self, name: &str, url: &str) -> Result<()> {
        self.run_dolt(&["remote", "add", name, url])?;
        Ok(())
    }

    pub fn pull(&self, remote: &str) -> Result<()> {
        self.run_dolt(&["pull", remote])?;
        Ok(())
    }

    pub fn push(&self, remote: &str) -> Result<()> {
        self.run_dolt(&["add", "-A"])?;
        self.run_dolt(&["commit", "-m", "znote sync push", "--allow-empty"])?;
        self.run_dolt(&["push", remote, "main"])?;
        Ok(())
    }

    pub fn init_db(&self) -> Result<()> {
        let mut newly_created = false;
        if !self.data_dir.join(".dolt").exists() {
            self.run_dolt(&["init"])?;
            newly_created = true;
        }

        // Setup notes
                // Ensure database exists if using MySQL
        let dbname = self.get_db_name();
        let _ = self.run_sql(&format!("CREATE DATABASE IF NOT EXISTS `{}`", dbname));

        self.run_sql(
            "CREATE TABLE IF NOT EXISTS notes (
                id VARCHAR(255) PRIMARY KEY,
                title VARCHAR(1024),
                content TEXT,
                created_at DATETIME,
                updated_at DATETIME
            )",
        )?;

        // Setup bookmarks
        self.run_sql(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id VARCHAR(255) PRIMARY KEY,
                url TEXT,
                title VARCHAR(1024),
                description TEXT,
                created_at DATETIME,
                updated_at DATETIME
            )",
        )?;

        // Setup tasks
        self.run_sql(
            "CREATE TABLE IF NOT EXISTS tasks (
                id VARCHAR(255) PRIMARY KEY,
                title VARCHAR(1024),
                description TEXT,
                items JSON,
                created_at DATETIME,
                updated_at DATETIME
            )",
        )?;

        // Setup tags
        self.run_sql(
            "CREATE TABLE IF NOT EXISTS tags (
                entity_id VARCHAR(255) NOT NULL,
                tag VARCHAR(255) NOT NULL,
                PRIMARY KEY (entity_id, tag),
                INDEX idx_tags_tag (tag)
            )",
        )?;

        // Setup links
        self.run_sql(
            "CREATE TABLE IF NOT EXISTS links (
                source_id VARCHAR(255) NOT NULL,
                target_id VARCHAR(255) NOT NULL,
                rel_type VARCHAR(64) NOT NULL DEFAULT 'rel',
                PRIMARY KEY (source_id, target_id, rel_type),
                INDEX idx_links_target (target_id)
            )",
        )?;

        if newly_created {
            self.import_from_fs()?;
        }
        Ok(())
    }

    pub fn import_from_fs(&self) -> Result<()> {
        if let Ok(notes) = crate::storage::list_notes_fs(&self.data_dir) {
            for note in notes {
                self.save_note(&note)?;
            }
        }
        if let Ok(bms) = crate::storage::list_bookmarks_fs(&self.data_dir) {
            for bm in bms {
                self.save_bookmark(&bm)?;
            }
        }
        if let Ok(tasks) = crate::storage::list_tasks_fs(&self.data_dir) {
            for task in tasks {
                self.save_task(&task)?;
            }
        }
        Ok(())
    }

    // A helper to escape SQL string
    fn escape_sql(val: &str) -> String {
        val.replace('\'', "''")
    }

    // -----------------------------------------------------------------------
    // Relations storage (tags & links)
    // -----------------------------------------------------------------------

    fn replace_tags(&self, entity_id: &str, tags: &[String]) -> Result<()> {
        self.run_sql(&format!(
            "DELETE FROM tags WHERE entity_id = '{}'",
            Self::escape_sql(entity_id)
        ))?;
        if tags.is_empty() {
            return Ok(());
        }
        let values = tags
            .iter()
            .map(|t| {
                format!(
                    "('{}', '{}')",
                    Self::escape_sql(entity_id),
                    Self::escape_sql(t)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.run_sql(&format!(
            "INSERT INTO tags (entity_id, tag) VALUES {}",
            values
        ))?;
        Ok(())
    }

    fn replace_links(&self, source_id: &str, links: &[String]) -> Result<()> {
        self.run_sql(&format!(
            "DELETE FROM links WHERE source_id = '{}'",
            Self::escape_sql(source_id)
        ))?;
        if links.is_empty() {
            return Ok(());
        }
        let values = links
            .iter()
            .map(|l| {
                let (rel, target) = match l.split_once(':') {
                    Some((r, t)) => (r, t),
                    None => ("", l.as_str()),
                };
                format!(
                    "('{}', '{}', '{}')",
                    Self::escape_sql(source_id),
                    Self::escape_sql(target),
                    Self::escape_sql(rel)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.run_sql(&format!(
            "INSERT INTO links (source_id, target_id, rel_type) VALUES {}",
            values
        ))?;
        Ok(())
    }

    fn load_tags(&self, entity_id: &str) -> Result<Vec<String>> {
        let sql = format!(
            "SELECT tag FROM tags WHERE entity_id = '{}'",
            Self::escape_sql(entity_id)
        );
        let res = self.run_sql(&sql)?;
        let rows_val = res
            .get("rows")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([]));
        let rows = rows_val.as_array().context("Rows is not an array")?;
        let mut tags = Vec::new();
        for r in rows {
            if let Some(t) = r["tag"].as_str() {
                tags.push(t.to_string());
            }
        }
        tags.sort();
        Ok(tags)
    }

    fn load_links(&self, source_id: &str) -> Result<Vec<String>> {
        let sql = format!(
            "SELECT rel_type, target_id FROM links WHERE source_id = '{}'",
            Self::escape_sql(source_id)
        );
        let res = self.run_sql(&sql)?;
        let rows_val = res
            .get("rows")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([]));
        let rows = rows_val.as_array().context("Rows is not an array")?;
        let mut links = Vec::new();
        for r in rows {
            if let (Some(rel), Some(target)) = (r["rel_type"].as_str(), r["target_id"].as_str()) {
                links.push(format!("{}:{}", rel, target));
            }
        }
        links.sort();
        Ok(links)
    }

    fn delete_tags_and_links(&self, id: &str) -> Result<()> {
        self.run_sql(&format!(
            "DELETE FROM tags WHERE entity_id = '{}'",
            Self::escape_sql(id)
        ))?;
        self.run_sql(&format!(
            "DELETE FROM links WHERE source_id = '{}'",
            Self::escape_sql(id)
        ))?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    
    fn fetch_all_tags(&self) -> Result<std::collections::HashMap<String, Vec<String>>> {
        let res = self.run_sql("SELECT entity_id, tag FROM tags ORDER BY entity_id, tag")?;
        let rows = res.get("rows").and_then(|r| r.as_array()).context("Missing rows")?;
        let mut map = std::collections::HashMap::new();
        for r in rows {
            let eid = r["entity_id"].as_str().unwrap_or("").to_string();
            let tag = r["tag"].as_str().unwrap_or("").to_string();
            map.entry(eid).or_insert_with(Vec::new).push(tag);
        }
        Ok(map)
    }

    fn fetch_all_links(&self) -> Result<std::collections::HashMap<String, Vec<String>>> {
        let res = self.run_sql("SELECT source_id, rel_type, target_id FROM links ORDER BY source_id")?;
        let rows = res.get("rows").and_then(|r| r.as_array()).context("Missing rows")?;
        let mut map = std::collections::HashMap::new();
        for r in rows {
            let sid = r["source_id"].as_str().unwrap_or("").to_string();
            let rel = r["rel_type"].as_str().unwrap_or("").to_string();
            let tid = r["target_id"].as_str().unwrap_or("").to_string();
            map.entry(sid).or_insert_with(Vec::new).push(format!("{}:{}", rel, tid));
        }
        Ok(map)
    }

    // Note Storage
    // -----------------------------------------------------------------------
    pub fn save_note(&self, note: &Note) -> Result<()> {
        let sql = format!(
            "REPLACE INTO notes (id, title, content, created_at, updated_at) VALUES ('{}', '{}', '{}', '{}', '{}')",
            Self::escape_sql(&note.id),
            Self::escape_sql(&note.title),
            Self::escape_sql(&note.content),
            note.created_at.format("%Y-%m-%d %H:%M:%S"),
            note.updated_at.format("%Y-%m-%d %H:%M:%S")
        );
        self.run_sql(&sql)?;
        self.replace_tags(&note.id, &note.tags)?;
        self.replace_links(&note.id, &note.links)?;

        // Sync markdown to disk
        crate::storage::save_note_fs(&self.data_dir, note)?;
        Ok(())
    }

    pub fn load_note(&self, id: &str) -> Result<Note> {
        let sql = format!(
            "SELECT * FROM notes WHERE id = '{}' LIMIT 1",
            Self::escape_sql(id)
        );
        let res = self.run_sql(&sql)?;
        let rows = res
            .get("rows")
            .and_then(|r| r.as_array())
            .context("Missing rows")?;
        if rows.is_empty() {
            anyhow::bail!("Note not found: {}", id);
        }

        let row = &rows[0];
        let created_at = self.parse_datetime_str(row["created_at"].as_str().unwrap_or(""))?;
        let updated_at = self.parse_datetime_str(row["updated_at"].as_str().unwrap_or(""))?;
        let tags = self.load_tags(id)?;
        let links = self.load_links(id)?;

        Ok(Note {
            id: row["id"].as_str().unwrap_or("").to_string(),
            title: row["title"].as_str().unwrap_or("").to_string(),
            content: row["content"].as_str().unwrap_or("").to_string(),
            tags,
            links,
            created_at,
            updated_at,
        })
    }

    pub fn delete_note(&self, id: &str) -> Result<()> {
        let sql = format!("DELETE FROM notes WHERE id = '{}'", Self::escape_sql(id));
        self.run_sql(&sql)?;
        self.delete_tags_and_links(id)?;

        // Ensure markdown is also deleted
        crate::storage::delete_note_fs(&self.data_dir, id).ok(); // ignore if not present physically
        Ok(())
    }

pub fn list_notes(&self) -> Result<Vec<Note>> {
        let mut all_tags = self.fetch_all_tags()?;
        let mut all_links = self.fetch_all_links()?;

        let res = self.run_sql("SELECT * FROM notes")?;
        let rows = res.get("rows").and_then(|r| r.as_array()).context("Missing rows")?;
        let mut notes = Vec::new();
        for row in rows {
            let id = row["id"].as_str().unwrap_or("").to_string();
            let created_at = self.parse_datetime_str(row["created_at"].as_str().unwrap_or(""))?;
            let updated_at = self.parse_datetime_str(row["updated_at"].as_str().unwrap_or(""))?;
            let tags = all_tags.remove(&id).unwrap_or_default();
            let links = all_links.remove(&id).unwrap_or_default();

            notes.push(Note {
                id,
                title: row["title"].as_str().unwrap_or("").to_string(),
                content: row["content"].as_str().unwrap_or("").to_string(),
                tags,
                links,
                created_at,
                updated_at,
            });
        }
        Ok(notes)
    }

    // -----------------------------------------------------------------------
    // Bookmark Storage
    // -----------------------------------------------------------------------
    pub fn save_bookmark(&self, bookmark: &Bookmark) -> Result<()> {
        let sql = format!(
            "REPLACE INTO bookmarks (id, url, title, description, created_at, updated_at) VALUES ('{}', '{}', '{}', '{}', '{}', '{}')",
            Self::escape_sql(&bookmark.id),
            Self::escape_sql(&bookmark.url),
            Self::escape_sql(&bookmark.title),
            Self::escape_sql(&bookmark.description.clone().unwrap_or_default()),
            bookmark.created_at.format("%Y-%m-%d %H:%M:%S"),
            bookmark.updated_at.format("%Y-%m-%d %H:%M:%S")
        );
        self.run_sql(&sql)?;
        self.replace_tags(&bookmark.id, &bookmark.tags)?;
        self.replace_links(&bookmark.id, &bookmark.links)?;

        crate::storage::save_bookmark_fs(&self.data_dir, bookmark)?;
        Ok(())
    }

    pub fn load_bookmark(&self, id: &str) -> Result<Bookmark> {
        let sql = format!(
            "SELECT * FROM bookmarks WHERE id = '{}' LIMIT 1",
            Self::escape_sql(id)
        );
        let res = self.run_sql(&sql)?;
        let rows = res
            .get("rows")
            .and_then(|r| r.as_array())
            .context("Missing rows")?;
        if rows.is_empty() {
            anyhow::bail!("Bookmark not found: {}", id);
        }

        let row = &rows[0];
        let desc_str = row["description"].as_str().unwrap_or("");

        let created_at = self.parse_datetime_str(row["created_at"].as_str().unwrap_or(""))?;
        let updated_at = self.parse_datetime_str(row["updated_at"].as_str().unwrap_or(""))?;
        let tags = self.load_tags(id)?;
        let links = self.load_links(id)?;

        Ok(Bookmark {
            id: row["id"].as_str().unwrap_or("").to_string(),
            url: row["url"].as_str().unwrap_or("").to_string(),
            title: row["title"].as_str().unwrap_or("").to_string(),
            description: if desc_str.is_empty() {
                None
            } else {
                Some(desc_str.to_string())
            },
            tags,
            links,
            created_at,
            updated_at,
        })
    }

    pub fn delete_bookmark(&self, id: &str) -> Result<()> {
        let sql = format!(
            "DELETE FROM bookmarks WHERE id = '{}'",
            Self::escape_sql(id)
        );
        self.run_sql(&sql)?;
        self.delete_tags_and_links(id)?;

        crate::storage::delete_bookmark_fs(&self.data_dir, id).ok();
        Ok(())
    }

pub fn list_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let mut all_tags = self.fetch_all_tags()?;
        let mut all_links = self.fetch_all_links()?;

        let res = self.run_sql("SELECT * FROM bookmarks")?;
        let rows = res.get("rows").and_then(|r| r.as_array()).context("Missing rows")?;
        let mut bms = Vec::new();
        for row in rows {
            let id = row["id"].as_str().unwrap_or("").to_string();
            let desc_str = row["description"].as_str().unwrap_or("");
            let created_at = self.parse_datetime_str(row["created_at"].as_str().unwrap_or(""))?;
            let updated_at = self.parse_datetime_str(row["updated_at"].as_str().unwrap_or(""))?;
            let tags = all_tags.remove(&id).unwrap_or_default();
            let links = all_links.remove(&id).unwrap_or_default();

            bms.push(Bookmark {
                id,
                url: row["url"].as_str().unwrap_or("").to_string(),
                title: row["title"].as_str().unwrap_or("").to_string(),
                description: if desc_str.is_empty() { None } else { Some(desc_str.to_string()) },
                tags,
                links,
                created_at,
                updated_at,
            });
        }
        Ok(bms)
    }

    // -----------------------------------------------------------------------
    // Task Storage
    // -----------------------------------------------------------------------
    pub fn save_task(&self, task: &Task) -> Result<()> {
        let items_json = serde_json::to_string(&task.items)?;
        let sql = format!(
            "REPLACE INTO tasks (id, title, description, items, created_at, updated_at) VALUES ('{}', '{}', '{}', '{}', '{}', '{}')",
            Self::escape_sql(&task.id),
            Self::escape_sql(&task.title),
            Self::escape_sql(&task.description.clone().unwrap_or_default()),
            Self::escape_sql(&items_json),
            task.created_at.format("%Y-%m-%d %H:%M:%S"),
            task.updated_at.format("%Y-%m-%d %H:%M:%S")
        );
        self.run_sql(&sql)?;

        // Harvest all tags for the task, including from its items
        let mut all_tags = task.tags.clone();
        for item in &task.items {
            all_tags.extend(item.tags.clone());
        }
        all_tags.sort();
        all_tags.dedup();

        self.replace_tags(&task.id, &all_tags)?;
        self.replace_links(&task.id, &task.links)?;

        crate::storage::save_task_fs(&self.data_dir, task)?;
        Ok(())
    }

    pub fn load_task(&self, id: &str) -> Result<Task> {
        let sql = format!(
            "SELECT * FROM tasks WHERE id = '{}' LIMIT 1",
            Self::escape_sql(id)
        );
        let res = self.run_sql(&sql)?;
        let rows = res
            .get("rows")
            .and_then(|r| r.as_array())
            .context("Missing rows")?;
        if rows.is_empty() {
            anyhow::bail!("Task not found: {}", id);
        }

        let row = &rows[0];
        let desc_str = row["description"].as_str().unwrap_or("");

        let items: Vec<TaskItem> = if let Some(items_val) = row.get("items") {
            if let Some(items_str) = items_val.as_str() {
                serde_json::from_str(items_str).unwrap_or_default()
            } else {
                serde_json::from_value(items_val.clone()).unwrap_or_default()
            }
        } else {
            vec![]
        };

        let created_at = self.parse_datetime_str(row["created_at"].as_str().unwrap_or(""))?;
        let updated_at = self.parse_datetime_str(row["updated_at"].as_str().unwrap_or(""))?;
        let tags = self.load_tags(id)?;
        let links = self.load_links(id)?;

        Ok(Task {
            id: row["id"].as_str().unwrap_or("").to_string(),
            title: row["title"].as_str().unwrap_or("").to_string(),
            description: if desc_str.is_empty() {
                None
            } else {
                Some(desc_str.to_string())
            },
            items,
            tags,
            links,
            created_at,
            updated_at,
        })
    }

    pub fn delete_task(&self, id: &str) -> Result<()> {
        let sql = format!("DELETE FROM tasks WHERE id = '{}'", Self::escape_sql(id));
        self.run_sql(&sql)?;
        self.delete_tags_and_links(id)?;

        crate::storage::delete_task_fs(&self.data_dir, id).ok();
        Ok(())
    }

pub fn list_tasks(&self) -> Result<Vec<Task>> {
        let mut all_tags = self.fetch_all_tags()?;
        let mut all_links = self.fetch_all_links()?;

        let res = self.run_sql("SELECT * FROM tasks")?;
        let rows = res.get("rows").and_then(|r| r.as_array()).context("Missing rows")?;
        let mut tasks = Vec::new();
        for row in rows {
            let id = row["id"].as_str().unwrap_or("").to_string();
            let desc_str = row["description"].as_str().unwrap_or("");
            let created_at = self.parse_datetime_str(row["created_at"].as_str().unwrap_or(""))?;
            let updated_at = self.parse_datetime_str(row["updated_at"].as_str().unwrap_or(""))?;
            let tags = all_tags.remove(&id).unwrap_or_default();
            let links = all_links.remove(&id).unwrap_or_default();
            
            let items: Vec<TaskItem> = if let Some(items_val) = row.get("items") {
                if let Some(items_str) = items_val.as_str() {
                    serde_json::from_str(items_str).unwrap_or_default()
                } else {
                    serde_json::from_value(items_val.clone()).unwrap_or_default()
                }
            } else {
                Vec::new()
            };

            tasks.push(Task {
                id,
                title: row["title"].as_str().unwrap_or("").to_string(),
                description: if desc_str.is_empty() { None } else { Some(desc_str.to_string()) },
                items,
                tags,
                links,
                created_at,
                updated_at,
            });
        }
        Ok(tasks)
    }

    /// Safely parse the datetime string returned by Dolt
    fn parse_datetime_str(&self, raw: &str) -> Result<DateTime<Utc>> {
        let cleaned = if let Some(idx) = raw.find('.') {
            &raw[..idx]
        } else {
            raw
        };

        chrono::NaiveDateTime::parse_from_str(cleaned, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc())
            .context("Failed to parse datetime from dolt")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::fs;
    use tempfile::tempdir;
    use uuid::Uuid;

    fn setup_env() -> Result<(tempfile::TempDir, DoltStorage)> {
        let dir = tempdir()?;
        let storage = DoltStorage::new(dir.path());
        storage.init_db()?;
        Ok((dir, storage))
    }

    #[test]
    fn test_dolt_init_creates_repo() -> Result<()> {
        let (dir, _storage) = setup_env()?;
        assert!(
            dir.path().join(".dolt").exists(),
            "Dolt repo should be initialized"
        );
        Ok(())
    }

    #[test]
    fn test_dolt_save_delete_list_note() -> Result<()> {
        let (dir, storage) = setup_env()?;

        let mut note = Note {
            id: Uuid::new_v4().to_string(),
            title: "Test Dolt Note".into(),
            content: "This is a note stored in dolt".into(),
            tags: vec!["test".into(), "dolt".into()],
            links: vec!["rel:12345".into()],
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
        };

        // Save
        storage.save_note(&note)?;

        // Load
        let loaded = storage.load_note(&note.id)?;
        assert_eq!(loaded.id, note.id);
        assert_eq!(loaded.title, "Test Dolt Note");
        assert_eq!(loaded.content, "This is a note stored in dolt");
        let mut loaded_tags = loaded.tags.clone();
        loaded_tags.sort();
        let mut expected_tags = vec!["test", "dolt"];
        expected_tags.sort();
        assert_eq!(loaded_tags, expected_tags);
        let expected_links = vec!["rel:12345"];
        assert_eq!(loaded.links, expected_links);

        // Verify markdown fallback
        let md_path = dir.path().join("notes").join(format!("{}.md", note.id));
        assert!(
            md_path.exists(),
            "Markdown fallback must be created to maintain markdown support"
        );

        let content = fs::read_to_string(md_path)?;
        assert!(content.contains("Test Dolt Note"));

        // List
        let listed = storage.list_notes()?;
        assert_eq!(listed.len(), 1);

        // Update
        note.title = "Updated Title".into();
        storage.save_note(&note)?;
        let updated = storage.load_note(&note.id)?;
        assert_eq!(updated.title, "Updated Title");

        // Delete
        storage.delete_note(&note.id)?;
        assert!(storage.load_note(&note.id).is_err());
        assert_eq!(storage.list_notes()?.len(), 0);

        // Ensure markdown deleted
        assert!(
            !dir.path()
                .join("notes")
                .join(format!("{}.md", note.id))
                .exists()
        );

        Ok(())
    }

    #[test]
    fn test_dolt_save_load_bookmark() -> Result<()> {
        let (_dir, storage) = setup_env()?;

        let bm = Bookmark {
            id: Uuid::new_v4().to_string(),
            title: "Dolt Docs".into(),
            url: "https://docs.dolthub.com/".into(),
            description: Some("Database docs".into()),
            tags: vec!["db".into()],
            links: vec![],
            created_at: Utc.with_ymd_and_hms(2025, 2, 2, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2025, 2, 2, 12, 0, 0).unwrap(),
        };

        storage.save_bookmark(&bm)?;
        let loaded = storage.load_bookmark(&bm.id)?;
        assert_eq!(loaded.title, "Dolt Docs");
        assert_eq!(loaded.url, "https://docs.dolthub.com/");
        assert_eq!(loaded.description, Some("Database docs".to_string()));

        storage.delete_bookmark(&bm.id)?;
        assert!(storage.load_bookmark(&bm.id).is_err());

        Ok(())
    }

    #[test]
    fn test_dolt_save_load_task() -> Result<()> {
        let (_dir, storage) = setup_env()?;

        let task = Task {
            id: Uuid::new_v4().to_string(),
            title: "Implement Dolt".into(),
            description: None,
            items: vec![
                TaskItem {
                    text: "Init".into(),
                    completed: true,
                    tags: vec!["db".into()],
                },
                TaskItem {
                    text: "Save Note".into(),
                    completed: false,
                    tags: vec!["critical".into()],
                },
            ],
            tags: vec!["sql".into()],
            links: vec![],
            created_at: Utc.with_ymd_and_hms(2025, 3, 3, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2025, 3, 3, 10, 0, 0).unwrap(),
        };

        storage.save_task(&task)?;
        let loaded = storage.load_task(&task.id)?;
        assert_eq!(loaded.title, "Implement Dolt");
        assert_eq!(loaded.items.len(), 2);
        assert_eq!(loaded.items[0].text, "Init");
        assert_eq!(loaded.items[0].completed, true);
        assert_eq!(loaded.items[1].text, "Save Note");
        assert_eq!(loaded.tags, vec!["critical", "db", "sql"]);

        storage.delete_task(&task.id)?;
        assert!(storage.load_task(&task.id).is_err());

        Ok(())
    }
}
