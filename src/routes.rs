use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{Level, debug, warn};

use crate::{
    app_error::AppError,
    repository::Repository,
    token::{get_encoding_key, produce_session_token},
    user::User,
};

/// Return an HTTP router with the handlers bound to the endpoints.
pub fn app(repo: Arc<dyn Repository<User>>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/session", post(create_session))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        // .include_headers(true)
                        .level(Level::INFO),
                )
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(repo)
}

#[derive(Serialize)]
struct HealthData {
    live: bool,
    ready: bool,
}

async fn health() -> Result<Json<HealthData>, AppError> {
    debug!("Health requested");
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
    payload: Result<Json<SessionRequest>, JsonRejection>,
) -> Result<Json<SessionResponse>, AppError> {
    let Json(payload) = payload?;
    let Some(user) = repo.find_by_id(&payload.username) else {
        warn!(username = payload.username, "User not found");
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    };
    debug!(username = payload.username, "Authenticating user");

    if !user.verify_password(&payload.password) {
        warn!(username = payload.username, "Wrong password for user");
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let encoding_key = get_encoding_key()?;
    let token = produce_session_token(&encoding_key, &payload.username)?;

    Ok(Json(SessionResponse { jwt: token }))
}
