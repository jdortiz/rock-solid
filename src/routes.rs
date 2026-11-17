use std::sync::Arc;

use axum::{Json, Router, http::StatusCode, routing::get};
use serde::Serialize;

use crate::{repository::Repository, user::User};

/// Return an HTTP router with the handlers bound to the endpoints.
pub fn app(repo: Arc<dyn Repository<User>>) -> Router {
    Router::new().route("/health", get(health)).with_state(repo)
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
