use axum::{Json, Router, routing::get};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use types::HealthResponse;

mod db;
mod errors;
mod routes;

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".parse().expect("valid filter")),
        )
        .init();

    let pool = db::init_pool().await?;

    let api_routes = Router::new()
        .route("/api/health", get(health))
        .merge(routes::authors::router())
        .with_state(pool);

    let app = api_routes
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
