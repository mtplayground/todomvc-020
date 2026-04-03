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

    let app = Router::new().fallback_service(ServeDir::new("target/site"));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
