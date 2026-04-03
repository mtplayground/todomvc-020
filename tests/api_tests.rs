use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{delete, get, patch, post};
use axum::Router;
use http_body_util::BodyExt;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tower::ServiceExt;

use todomvc::models::Todo;

async fn setup_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    pool
}

fn build_app(pool: SqlitePool) -> Router {
    Router::new()
        .route("/api/todos", get(todomvc::api::list_todos))
        .route("/api/todos", post(todomvc::api::create_todo))
        .route("/api/todos/{id}", patch(todomvc::api::update_todo))
        .route("/api/todos/{id}", delete(todomvc::api::delete_todo))
        .route("/api/todos/toggle-all", patch(todomvc::api::toggle_all))
        .route(
            "/api/todos/completed",
            delete(todomvc::api::clear_completed),
        )
        .with_state(pool)
}

async fn body_to_string(body: Body) -> String {
    let bytes = body
        .collect()
        .await
        .expect("failed to read body")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("invalid utf8")
}

async fn body_to_json<T: serde::de::DeserializeOwned>(body: Body) -> T {
    let s = body_to_string(body).await;
    serde_json::from_str(&s).expect("failed to parse JSON")
}

// ── List (empty) ──────────────────────────────────────────────

#[tokio::test]
async fn list_todos_empty() {
    let pool = setup_pool().await;
    let app = build_app(pool);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/todos")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let todos: Vec<Todo> = body_to_json(resp.into_body()).await;
    assert!(todos.is_empty());
}

// ── Create ────────────────────────────────────────────────────

#[tokio::test]
async fn create_todo_success() {
    let pool = setup_pool().await;
    let app = build_app(pool);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Buy milk"}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::CREATED);
    let todo: Todo = body_to_json(resp.into_body()).await;
    assert_eq!(todo.title, "Buy milk");
    assert!(!todo.completed);
    assert_eq!(todo.display_order, 1);
}

#[tokio::test]
async fn create_todo_trims_whitespace() {
    let pool = setup_pool().await;
    let app = build_app(pool);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"  Buy milk  "}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::CREATED);
    let todo: Todo = body_to_json(resp.into_body()).await;
    assert_eq!(todo.title, "Buy milk");
}

#[tokio::test]
async fn create_todo_empty_title_returns_400() {
    let pool = setup_pool().await;
    let app = build_app(pool);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"   "}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Update ────────────────────────────────────────────────────

#[tokio::test]
async fn update_todo_title() {
    let pool = setup_pool().await;
    let app = build_app(pool.clone());

    // Create a todo first
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Original"}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    let created: Todo = body_to_json(resp.into_body()).await;

    // Update title
    let app = build_app(pool);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/todos/{}", created.id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Updated"}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let updated: Todo = body_to_json(resp.into_body()).await;
    assert_eq!(updated.title, "Updated");
    assert_eq!(updated.id, created.id);
}

#[tokio::test]
async fn update_todo_completed() {
    let pool = setup_pool().await;
    let app = build_app(pool.clone());

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Test"}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    let created: Todo = body_to_json(resp.into_body()).await;

    let app = build_app(pool);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/todos/{}", created.id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"completed":true}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let updated: Todo = body_to_json(resp.into_body()).await;
    assert!(updated.completed);
}

#[tokio::test]
async fn update_nonexistent_todo_returns_404() {
    let pool = setup_pool().await;
    let app = build_app(pool);

    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/todos/9999")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Nope"}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_todo_empty_title_returns_400() {
    let pool = setup_pool().await;
    let app = build_app(pool.clone());

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Test"}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    let created: Todo = body_to_json(resp.into_body()).await;

    let app = build_app(pool);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/todos/{}", created.id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"   "}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Delete ────────────────────────────────────────────────────

#[tokio::test]
async fn delete_todo_success() {
    let pool = setup_pool().await;
    let app = build_app(pool.clone());

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Delete me"}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    let created: Todo = body_to_json(resp.into_body()).await;

    let app = build_app(pool.clone());
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/todos/{}", created.id))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let app = build_app(pool);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/todos")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let todos: Vec<Todo> = body_to_json(resp.into_body()).await;
    assert!(todos.is_empty());
}

#[tokio::test]
async fn delete_nonexistent_todo_returns_404() {
    let pool = setup_pool().await;
    let app = build_app(pool);

    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/todos/9999")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── Toggle All ────────────────────────────────────────────────

#[tokio::test]
async fn toggle_all_sets_completed() {
    let pool = setup_pool().await;

    // Create two todos
    for title in &["Todo 1", "Todo 2"] {
        let app = build_app(pool.clone());
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"title":"{title}"}}"#)))
                .expect("request"),
        )
        .await
        .expect("response");
    }

    // Toggle all to completed
    let app = build_app(pool.clone());
    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/todos/toggle-all")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"completed":true}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let todos: Vec<Todo> = body_to_json(resp.into_body()).await;
    assert_eq!(todos.len(), 2);
    assert!(todos.iter().all(|t| t.completed));

    // Toggle all back to incomplete
    let app = build_app(pool);
    let resp = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/todos/toggle-all")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"completed":false}"#))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::OK);
    let todos: Vec<Todo> = body_to_json(resp.into_body()).await;
    assert!(todos.iter().all(|t| !t.completed));
}

// ── Clear Completed ───────────────────────────────────────────

#[tokio::test]
async fn clear_completed_removes_only_completed() {
    let pool = setup_pool().await;

    // Create two todos
    let app = build_app(pool.clone());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Keep me"}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    let _active: Todo = body_to_json(resp.into_body()).await;

    let app = build_app(pool.clone());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Complete me"}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    let to_complete: Todo = body_to_json(resp.into_body()).await;

    // Mark second as completed
    let app = build_app(pool.clone());
    app.oneshot(
        Request::builder()
            .method("PATCH")
            .uri(format!("/api/todos/{}", to_complete.id))
            .header("content-type", "application/json")
            .body(Body::from(r#"{"completed":true}"#))
            .expect("request"),
    )
    .await
    .expect("response");

    // Clear completed
    let app = build_app(pool.clone());
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/todos/completed")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify only active todo remains
    let app = build_app(pool);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/todos")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let todos: Vec<Todo> = body_to_json(resp.into_body()).await;
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "Keep me");
}

// ── List ordering ─────────────────────────────────────────────

#[tokio::test]
async fn list_todos_ordered_by_display_order() {
    let pool = setup_pool().await;

    for title in &["First", "Second", "Third"] {
        let app = build_app(pool.clone());
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"title":"{title}"}}"#)))
                .expect("request"),
        )
        .await
        .expect("response");
    }

    let app = build_app(pool);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/todos")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    let todos: Vec<Todo> = body_to_json(resp.into_body()).await;
    assert_eq!(todos.len(), 3);
    assert_eq!(todos[0].title, "First");
    assert_eq!(todos[1].title, "Second");
    assert_eq!(todos[2].title, "Third");
}
