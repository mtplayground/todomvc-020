use serde::{Deserialize, Serialize};

/// A todo item as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub display_order: i64,
}

/// Request body for creating a new todo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

/// Request body for toggling all todos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleAll {
    pub completed: bool,
}

/// Request body for updating an existing todo.
/// All fields are optional — only provided fields are updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub display_order: Option<i64>,
}
