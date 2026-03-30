use axum::{Json, Router, routing::get};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use types::HealthResponse;

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_routes = Router::new().route("/api/health", get(health));

    let app = api_routes.fallback_service(ServeDir::new("frontend/dist"));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
