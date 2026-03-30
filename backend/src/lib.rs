pub mod db;
pub mod errors;
pub mod routes;

use axum::{Json, Router, routing::get};
use sqlx::SqlitePool;
use types::HealthResponse;

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

pub fn build_app(pool: SqlitePool) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .merge(routes::authors::router())
        .merge(routes::books::router())
        .with_state(pool)
}
