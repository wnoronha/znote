use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskItem {
    pub text: String,
    pub completed: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub items: Vec<TaskItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(title: String, tags: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description: None,
            tags,
            links: Vec::new(),
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_item(&mut self, text: String, tags: Vec<String>) {
        self.items.push(TaskItem {
            text,
            completed: false,
            tags,
        });
        self.updated_at = Utc::now();
    }
}
