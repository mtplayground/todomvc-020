use serde::{Deserialize, Serialize};

/// A todo item as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub display_order: i64,
}

/// Request body for creating a new todo.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

/// Request body for updating an existing todo.
/// All fields are optional — only provided fields are updated.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub display_order: Option<i64>,
}
