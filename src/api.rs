use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sqlx::SqlitePool;

use crate::models::{CreateTodo, Todo, UpdateTodo};

pub async fn list_todos(State(pool): State<SqlitePool>) -> impl IntoResponse {
    match sqlx::query_as::<_, Todo>(
        "SELECT id, title, completed, display_order FROM todos ORDER BY display_order, id",
    )
    .fetch_all(&pool)
    .await
    {
        Ok(todos) => Json(todos).into_response(),
        Err(e) => {
            tracing::error!("failed to list todos: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn create_todo(
    State(pool): State<SqlitePool>,
    Json(input): Json<CreateTodo>,
) -> impl IntoResponse {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return (StatusCode::BAD_REQUEST, "title must not be empty").into_response();
    }

    match sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, completed, display_order) \
         VALUES (?, FALSE, (SELECT COALESCE(MAX(display_order), 0) + 1 FROM todos)) \
         RETURNING id, title, completed, display_order",
    )
    .bind(&title)
    .fetch_one(&pool)
    .await
    {
        Ok(todo) => (StatusCode::CREATED, Json(todo)).into_response(),
        Err(e) => {
            tracing::error!("failed to create todo: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn update_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateTodo>,
) -> impl IntoResponse {
    if input.title.is_none() && input.completed.is_none() && input.display_order.is_none() {
        return (StatusCode::BAD_REQUEST, "no fields to update").into_response();
    }

    // Fetch current todo first
    let current = match sqlx::query_as::<_, Todo>(
        "SELECT id, title, completed, display_order FROM todos WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(todo)) => todo,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("failed to fetch todo {id}: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let title = match input.title {
        Some(ref t) => {
            let trimmed = t.trim().to_string();
            if trimmed.is_empty() {
                return (StatusCode::BAD_REQUEST, "title must not be empty").into_response();
            }
            trimmed
        }
        None => current.title,
    };
    let completed = input.completed.unwrap_or(current.completed);
    let display_order = input.display_order.unwrap_or(current.display_order);

    match sqlx::query_as::<_, Todo>(
        "UPDATE todos SET title = ?, completed = ?, display_order = ? WHERE id = ? \
         RETURNING id, title, completed, display_order",
    )
    .bind(&title)
    .bind(completed)
    .bind(display_order)
    .bind(id)
    .fetch_one(&pool)
    .await
    {
        Ok(todo) => Json(todo).into_response(),
        Err(e) => {
            tracing::error!("failed to update todo {id}: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn delete_todo(State(pool): State<SqlitePool>, Path(id): Path<i64>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("failed to delete todo {id}: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
