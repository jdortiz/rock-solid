use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    repository::Repository,
    token::{get_encoding_key, produce_session_token},
    user::User,
};

/// Return an HTTP router with the handlers bound to the endpoints.
pub fn app(repo: Arc<dyn Repository<User>>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/session", post(create_session))
        .with_state(repo)
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

#[derive(Deserialize)]
struct SessionRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct SessionResponse {
    /// Session JWT.
    jwt: String,
}

/// HTTP handler to create a session for a correspondent in the application.
async fn create_session(
    State(repo): State<Arc<dyn Repository<User>>>,
    Json(payload): Json<SessionRequest>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let Some(user) = repo.find_by_id(&payload.username) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if !user.verify_password(&payload.password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let encoding_key = get_encoding_key().map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = produce_session_token(&encoding_key, &payload.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SessionResponse { jwt: token }))
}
