use axum::routing::{delete, get, patch, post};
use axum::Router;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let pool = todomvc::db::init_pool()
        .await
        .expect("failed to initialize database");

    let api = Router::new()
        .route("/api/todos", get(todomvc::api::list_todos))
        .route("/api/todos", post(todomvc::api::create_todo))
        .route("/api/todos/{id}", patch(todomvc::api::update_todo))
        .route("/api/todos/{id}", delete(todomvc::api::delete_todo))
        .route("/api/todos/toggle-all", patch(todomvc::api::toggle_all))
        .route(
            "/api/todos/completed",
            delete(todomvc::api::clear_completed),
        );

    let app = api
        .with_state(pool)
        .fallback_service(ServeDir::new("target/site"));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind to address");

    axum::serve(listener, app).await.expect("server error");
}
