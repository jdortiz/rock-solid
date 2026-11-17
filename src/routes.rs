use axum::{Json, Router, http::StatusCode, routing::get};
use serde::Serialize;

/// Return an HTTP router with the handlers bound to the endpoints.
pub fn app() -> Router {
    Router::new().route("/health", get(health))
}

#[derive(Serialize)]
struct HealthData {
    live: bool,
    ready: bool,
}

async fn health() -> Result<Json<HealthData>, StatusCode> {
    Ok(Json(HealthData {
        live: true,
        ready: true,
    }))
}
