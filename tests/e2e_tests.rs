//! End-to-end tests verifying full user flows through the API.
//! Each test simulates a realistic user scenario from start to finish.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{delete, get, patch, post};
use axum::Router;
use http_body_util::BodyExt;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tower::ServiceExt;

use todomvc::models::Todo;

async fn setup() -> (SqlitePool, Router) {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let app = Router::new()
        .route("/api/todos", get(todomvc::api::list_todos))
        .route("/api/todos", post(todomvc::api::create_todo))
        .route("/api/todos/{id}", patch(todomvc::api::update_todo))
        .route("/api/todos/{id}", delete(todomvc::api::delete_todo))
        .route("/api/todos/toggle-all", patch(todomvc::api::toggle_all))
        .route(
            "/api/todos/completed",
            delete(todomvc::api::clear_completed),
        )
        .with_state(pool.clone());

    (pool, app)
}

fn app(pool: SqlitePool) -> Router {
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

async fn json<T: serde::de::DeserializeOwned>(body: Body) -> T {
    let bytes = body.collect().await.expect("collect body").to_bytes();
    serde_json::from_slice(&bytes).expect("parse json")
}

async fn post_todo(pool: &SqlitePool, title: &str) -> Todo {
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"title":"{title}"}}"#)))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::CREATED);
    json(resp.into_body()).await
}

async fn get_todos(pool: &SqlitePool) -> Vec<Todo> {
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .uri("/api/todos")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    json(resp.into_body()).await
}

// ── Flow: Add multiple todos, verify ordering ─────────────────

#[tokio::test]
async fn flow_add_multiple_todos() {
    let (pool, _) = setup().await;

    let t1 = post_todo(&pool, "Buy groceries").await;
    let t2 = post_todo(&pool, "Walk the dog").await;
    let t3 = post_todo(&pool, "Read a book").await;

    assert!(!t1.completed);
    assert!(!t2.completed);
    assert!(!t3.completed);

    let todos = get_todos(&pool).await;
    assert_eq!(todos.len(), 3);
    assert_eq!(todos[0].title, "Buy groceries");
    assert_eq!(todos[1].title, "Walk the dog");
    assert_eq!(todos[2].title, "Read a book");
    // display_order should be sequential
    assert!(todos[0].display_order < todos[1].display_order);
    assert!(todos[1].display_order < todos[2].display_order);
}

// ── Flow: Add, complete, filter, clear ────────────────────────

#[tokio::test]
async fn flow_complete_filter_clear() {
    let (pool, _) = setup().await;

    // Add three todos
    let t1 = post_todo(&pool, "Task A").await;
    let _t2 = post_todo(&pool, "Task B").await;
    let t3 = post_todo(&pool, "Task C").await;

    // Complete Task A and Task C
    for id in [t1.id, t3.id] {
        let resp = app(pool.clone())
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/todos/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"completed":true}"#))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // Verify: 2 completed, 1 active
    let todos = get_todos(&pool).await;
    let active: Vec<_> = todos.iter().filter(|t| !t.completed).collect();
    let completed: Vec<_> = todos.iter().filter(|t| t.completed).collect();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].title, "Task B");
    assert_eq!(completed.len(), 2);

    // Clear completed
    let resp = app(pool.clone())
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

    // Only Task B should remain
    let todos = get_todos(&pool).await;
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "Task B");
    assert!(!todos[0].completed);
}

// ── Flow: Add, edit title, verify persistence ─────────────────

#[tokio::test]
async fn flow_add_and_edit() {
    let (pool, _) = setup().await;

    let todo = post_todo(&pool, "Orignal title").await;

    // Edit the title (fix typo)
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/todos/{}", todo.id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Original title"}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);
    let updated: Todo = json(resp.into_body()).await;
    assert_eq!(updated.title, "Original title");
    assert_eq!(updated.id, todo.id);
    assert!(!updated.completed);

    // Verify persisted
    let todos = get_todos(&pool).await;
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "Original title");
}

// ── Flow: Toggle-all complete then uncomplete ─────────────────

#[tokio::test]
async fn flow_toggle_all_round_trip() {
    let (pool, _) = setup().await;

    post_todo(&pool, "One").await;
    post_todo(&pool, "Two").await;
    post_todo(&pool, "Three").await;

    // Toggle all to completed
    let resp = app(pool.clone())
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
    let todos: Vec<Todo> = json(resp.into_body()).await;
    assert!(todos.iter().all(|t| t.completed));

    // Toggle all back to active
    let resp = app(pool.clone())
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
    let todos: Vec<Todo> = json(resp.into_body()).await;
    assert!(todos.iter().all(|t| !t.completed));
}

// ── Flow: Add, delete, verify gone ────────────────────────────

#[tokio::test]
async fn flow_add_and_delete() {
    let (pool, _) = setup().await;

    let t1 = post_todo(&pool, "Keep").await;
    let t2 = post_todo(&pool, "Delete me").await;

    // Delete t2
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/todos/{}", t2.id))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify only t1 remains
    let todos = get_todos(&pool).await;
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].id, t1.id);
    assert_eq!(todos[0].title, "Keep");

    // Deleting again should 404
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/todos/{}", t2.id))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── Flow: Full lifecycle — add, complete some, edit, toggle-all, clear ──

#[tokio::test]
async fn flow_full_lifecycle() {
    let (pool, _) = setup().await;

    // 1. Start empty
    let todos = get_todos(&pool).await;
    assert!(todos.is_empty());

    // 2. Add todos
    let t1 = post_todo(&pool, "Learn Rust").await;
    let t2 = post_todo(&pool, "Build TodoMVC").await;
    let t3 = post_todo(&pool, "Write tests").await;

    // 3. Complete "Build TodoMVC"
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/todos/{}", t2.id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"completed":true}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Edit "Learn Rust" to "Learn Leptos"
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/todos/{}", t1.id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Learn Leptos"}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::OK);

    // 5. Verify state: 2 active, 1 completed
    let todos = get_todos(&pool).await;
    assert_eq!(todos.len(), 3);
    assert_eq!(todos[0].title, "Learn Leptos");
    assert!(!todos[0].completed);
    assert_eq!(todos[1].title, "Build TodoMVC");
    assert!(todos[1].completed);
    assert_eq!(todos[2].title, "Write tests");
    assert!(!todos[2].completed);

    // 6. Toggle all to completed
    let resp = app(pool.clone())
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
    let todos: Vec<Todo> = json(resp.into_body()).await;
    assert!(todos.iter().all(|t| t.completed));

    // 7. Delete "Write tests"
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/todos/{}", t3.id))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // 8. Clear completed — removes remaining two
    let resp = app(pool.clone())
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

    // 9. Verify empty
    let todos = get_todos(&pool).await;
    assert!(todos.is_empty());
}

// ── Flow: Edge cases in a realistic sequence ──────────────────

#[tokio::test]
async fn flow_edge_cases() {
    let (pool, _) = setup().await;

    // Cannot create with blank title
    let resp = app(pool.clone())
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

    // Create a real todo
    let todo = post_todo(&pool, "  Trimmed  ").await;
    assert_eq!(todo.title, "Trimmed");

    // Cannot update to blank title
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/todos/{}", todo.id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"  "}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Update nonexistent todo
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/todos/99999")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"Nope"}"#))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Delete nonexistent todo
    let resp = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/todos/99999")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Clear completed when none are completed — should succeed
    let resp = app(pool.clone())
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

    // Original todo still exists
    let todos = get_todos(&pool).await;
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "Trimmed");
}
