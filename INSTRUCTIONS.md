- [Pre-workshop setup](#org39ed6c3)
- [Introduction and context](#org73e3a8c)
- [Build a basic HTTP server](#org7214df7)
  - [Basic server structure, handlers, and serialization](#org94f4353)
  - [Add user data to a repository](#org4f60fd7)
- [Add basic security features](#orgc0f3608)
  - [Create a session and issue a JWT](#org5125c9b)
  - [Add logging and tracing](#org624e53e)
  - [Implement unified error handling](#org62158f5)
- [Add OTP with Vonage Verify v2](#orgd5a6f82)
  - [Create a Vonage application](#orgf6a27af)
  - [Replace the first authentication step](#orgd25ec05)
  - [Verify the OTP](#org1db9b7f)
  - [Reduce risk with Identity Insights](#org6fe986a)
- [Implement authorization with middleware](#org6f90610)
  - [Validate an individual endpoint](#org0e81cda)
  - [Protect the private router with middleware](#org8f28067)
  - [Hardening and production concerns](#org672f368)
    - [Timeouts](#orge25c482)
    - [Cross-Origin Resource Sharing (CORS)](#orgb85931b)
    - [Rate limiting](#orgf4fe0eb)



<a id="org39ed6c3"></a>

# Pre-workshop setup

-   Start the Codespaces container and share its URL. Use a supported browser; do not use Safari.
-   Participants should create a Vonage account, collect their API credentials, and confirm access to Verify v2 and Identity Insights.
-   Verify the toolchain with `cargo --version` and `cargo run` in the Hello World starter.


<a id="org73e3a8c"></a>

# Introduction and context


<a id="org7214df7"></a>

# Build a basic HTTP server


<a id="org94f4353"></a>

## Basic server structure, handlers, and serialization

1.  Add Tokio from the command line.

    ```shell
    cargo add -F rt-multi-thread,macros tokio
    ```
2.  Make `main` asynchronous and annotate it.

    ```rust
    #[tokio::main]
    async fn main() {
    ```
3.  Add Axum.

    ```shell
    cargo add axum
    ```
4.  Define the service address.

    ```rust
    const SERVER_ADDR: &str = "127.0.0.1:8080";
    ```
5.  Replace the startup message.

    ```rust
    println!("Launching server: http://{SERVER_ADDR}/");
    ```
6.  Create a Tokio TCP listener with (`use tokio::net::TcpListener;`).

    ```rust
    let listener = TcpListener::bind(SERVER_ADDR)
        .await
        .expect("Unable to create listener");
    ```
7.  Add the routes module to `main.rs`.

    ```rust
    mod routes;
    ```
8.  Define a function that returns the router.

    ```rust
    use axum::Router;

    /// Return an HTTP router with the handlers bound to the endpoints.
    pub fn app() -> Router {
        Router::new()
    }
    ```
9.  Add a handler for the health endpoint.

    ```rust
    Router::new().route("/health", get(health))
    //...

    async fn health() -> Result<String, StatusCode> {
        Ok("Tot-be".to_string())
    }
    ```
10. Call the router function from `main.rs` and serve it at the end of `main`.

    ```rust
    axum::serve(listener, app()).await.unwrap();
    ```
11. Define the `HealthData` structure.

    ```rust
    struct HealthData {
        live: bool,
        ready: bool,
    }
    ```
12. Add the dependencies needed to serialize this data as JSON.

    ```shell
    cargo add -F derive serde
    cargo add serde_json
    ```
13. Derive `Serialize` for `HealthData`.

    ```rust
    #[derive(Serialize)]
    struct HealthData {
    ```
14. Change the handler signature and return the data as JSON.

    ```rust
    async fn health() -> Result<Json<HealthData>, StatusCode> {
        Ok(Json(HealthData {
            live: true,
            ready: true,
        }))
    }
    ```
15. Run the service.

    ```shell
    cargo run
    ```
16. Test `/health`.

    ```shell
    curl_tests/get_health.sh
    ```


<a id="org4f60fd7"></a>

## Add user data to a repository

A repository separates data access from the HTTP handlers. Start with an in-memory implementation for the workshop.

1.  The `Repository` trait defines the data-access abstraction, and `InMemoryRepository` implements it. Add the repository module to `main.rs`.

    ```rust
    mod repository;
    ```
2.  Then copy the code into `repository.rs`.

    ```rust
    #![allow(dead_code)]
    use std::collections::HashMap;
    use std::hash::Hash;

    /// Identifiable types have a method to return its ID.
    pub trait Identifiable {
        type Id: Clone + Eq + Hash + Send + Sync;

        fn id(&self) -> &Self::Id;
    }

    /// Abstraction to store and manage [`Identifiable`] objects.
    pub trait Repository<T: Identifiable>: Send + Sync {
        /// Add a new [`Identifiable`] item to the repository.
        fn add(&mut self, item: T);
        /// Find an [`Identifiable`] item using its ID.
        fn find_by_id(&self, id: &T::Id) -> Option<&T>;
    }

    /// Simple implementation of [`Repository`] that holds the data in memory.
    pub struct InMemoryRepository<T>
    where
        T: Identifiable,
    {
        entities: HashMap<T::Id, T>,
    }

    impl<T> InMemoryRepository<T>
    where
        T: Identifiable,
    {
        /// Create a new [`InMemoryRepository`].
        pub fn new() -> Self {
            Self {
                entities: HashMap::new(),
            }
        }
    }

    impl<T> Repository<T> for InMemoryRepository<T>
    where
        T: Identifiable + Send + Sync,
    {
        fn add(&mut self, item: T) {
            let id = item.id().clone();
            self.entities.insert(id, item);
        }

        fn find_by_id(&self, id: &T::Id) -> Option<&T> {
            self.entities.get(id)
        }
    }
    ```
3.  Add the bcrypt crate for password hashing.

    ```shell
    cargo add bcrypt
    ```
4.  Add the user module to `main.rs`.

    ```rust
    mod user;
    ```
5.  Then copy the code into `user.rs`.

    ```rust
    #![allow(dead_code)]
    use crate::repository::Identifiable;

    /// Application users
    pub struct User {
        pub username: String,
        hashed_password: String,
    }

    impl User {
        /// Create a new user.
        pub fn new(username: impl Into<String>, password: &str) -> Self {
            Self {
                username: username.into(),
                hashed_password: bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap(),
            }
        }

        /// Check if the provided password matches the one of this user.
        pub fn verify_password(&self, password: &str) -> bool {
            bcrypt::verify(password, &self.hashed_password).unwrap_or(false)
        }
    }

    impl Identifiable for User {
        type Id = String;

        fn id(&self) -> &Self::Id {
            &self.username
        }
    }
    ```
6.  The `User` implementation stores a salted password hash instead of the plaintext password. Create a helper in `main.rs` that seeds a demo user.

    ```rust
    /// Create a user repository for demo purposes.
    fn seed_users() -> InMemoryRepository<User> {
        let mut users = InMemoryRepository::new();
        users.add(User::new("alice", "secret123"));

        users
    }
    ```
7.  Add an instance of the repository with that data that can be used in shared application state.

    ```rust
    let repo: Arc<dyn Repository<User>> = Arc::new(seed_users());
    ```
8.  Pass the repository to the router in `routes.rs` so handlers can use it.

    ```rust
    pub fn app(repo: Arc<dyn Repository<User>>) -> Router {
        Router::new().route("/health", get(health)).with_state(repo)
    }
    ```
9.  Pass the repository from `main.rs` and use the router to construct the service.

    ```rust
    axum::serve(listener, app(repo)).await.unwrap();
    ```
10. Run the service.

    ```shell
    cargo r
    ```
11. Test it.

    ```shell
    curl_tests/get_health.sh
    ```


<a id="orgc0f3608"></a>

# Add basic security features

In this section of the workshop we will implement authentication, accounting. We will tackle authorization and administration/accounting later.


<a id="org5125c9b"></a>

## Create a session and issue a JWT

HTTP is stateless by default, so the service needs a way to represent an authenticated session. First compare JWTs with cookies, then implement a password-authenticated session endpoint.

1.  In `routes.rs`, define the request body for creating a session.

    ```rust
    #[derive(Deserialize)]
    struct SessionRequest {
        username: String,
        password: String,
    }
    ```
2.  Define the session-creation handler.

    ```rust
    /// HTTP handler to create a session for a correspondent in the application.
    async fn create_session(
        State(repo): State<Arc<dyn Repository<User>>>,
        Json(payload): Json<SessionRequest>,
    ) -> Result<Json<SessionResponse>, StatusCode> {
    }
    ```
3.  Define the response structure that carries the session data.

    ```rust
    #[derive(Serialize)]
    struct SessionResponse {
        /// Session JWT.
        jwt: String,
    }
    ```
4.  In the handler, find the user and verify the submitted password.

    ```rust
    let Some(user) = repo.find_by_id(&payload.username) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if !user.verify_password(&payload.password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Produce and return a token
    Err(StatusCode::IM_A_TEAPOT)
    ```
5.  Add JWT and general-purpose error crates.

    ```shell
    cargo add -F rust_crypto jsonwebtoken
    cargo add anyhow
    ```
6.  Add the token module to `main.rs`.

    ```rust
    mod token;
    ```
7.  Then add functions to create and verify session JWTs.

    ```rust
    #![allow(dead_code)]
    use std::time::{SystemTime, UNIX_EPOCH};

    use axum::http::{HeaderMap, StatusCode};
    use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
    use serde::{Deserialize, Serialize};

    /// JWT claims for the session.
    #[derive(Serialize, Deserialize)]
    struct SessionClaims {
        /// Subject, i.e. username.
        sub: String,
        /// The time at which the token should expire
        exp: u64,
    }

    /// Create a JWT for the application session.
    pub fn produce_session_token(encoding_key: &EncodingKey, username: &str) -> anyhow::Result<String> {
        const SESSION_EXPIRY_SECS: u64 = 3600;

        let now = get_now_in_secs();
        let claims = SessionClaims {
            sub: username.to_string(),
            exp: now + SESSION_EXPIRY_SECS,
        };
        Ok(encode(
            &Header::new(Algorithm::RS256),
            &claims,
            encoding_key,
        )?)
    }

    /// Verify the signature of the session JWT.
    pub fn verify_session_token(
        decoding_key: &DecodingKey,
        headers: &HeaderMap,
    ) -> Result<(), StatusCode> {
        let token = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNAUTHORIZED)?;

        decode::<SessionClaims>(token, &decoding_key, &Validation::new(Algorithm::RS256))
            .map(|_| ())
            .map_err(|_| StatusCode::UNAUTHORIZED)
    }

    fn get_now_in_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    ```
8.  Read the RSA keys used to encode and decode the tokens.

    ```rust
    /// Auxiliary function to obtain the encoding key from the environment.
    pub fn get_encoding_key() -> anyhow::Result<EncodingKey> {
        let secret = std::env::var("JWT_PRIVATE_KEY")?;
        Ok(EncodingKey::from_rsa_pem(secret.as_bytes())?)
    }

    /// Auxiliary function to obtain the decoding key from the environment.
    pub fn get_decoding_key() -> anyhow::Result<DecodingKey> {
        let public_key = std::env::var("JWT_PUBLIC_KEY")?;
        Ok(DecodingKey::from_rsa_pem(public_key.as_bytes())?)
    }
    ```
9.  Add dotenvy so local development can load values from an `.env` file.

    ```shell
    cargo add dotenvy
    ```
10. Initialize it in `main.rs`.

    ```rust
    dotenvy::dotenv().expect("Unable to load configuration from .env");
    ```
11. In `router.rs`, after the password is valid, use the encoding key to issue the session token. Replace the temporary teapot response with it.

    ```rust
    let encoding_key = get_encoding_key().map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = produce_session_token(&encoding_key, &payload.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SessionResponse { jwt: token }))
    ```
12. Add the handler to the router.

    ```rust
    .route("/session", post(create_session))
    ```
13. Before running this version, create the JWT key pair with `generate-env.sh`.

    ```shell
    ./generate-env.sh
    ```
14. Run the service

    ```shell
    cargo r
    ```
15. From another terminal, test session creation.

    ```rust
    curl_tests/new_session.sh
    ```


<a id="org624e53e"></a>

## Add logging and tracing

A tracing span describes the lifetime and context of an operation; events record what happened inside it.

1.  Add the tracing dependencies:

    ```shell
    cargo add tracing
    cargo add -F json,env-filter tracing-subscriber
    ```
2.  Initialize and configure the subscriber.

    ```rust
    tracing_subscriber::fmt()
        // .json()
        .init();
    ```
3.  Replace the startup `println!` with an `info!` event.

    ```rust
    info!(addr = SERVER_ADDR, "Launching server");
    ```
4.  Add a span and an event to the health handler.

    ```rust
    #[tracing::instrument]
    async fn health() -> Result<Json<HealthData>, StatusCode> {
        debug!("Health requested");
        Ok(Json(HealthData {
            live: true,
            ready: true,
        }))
    }
    ```
5.  Run the service.

    ```shell
    cargo r
    ```
6.  And test it.

    ```shell
    curl_tests/get_health.sh
    ```
7.  No event appears at the default level, so configure the subscriber.

    ```rust
    .with_env_filter(EnvFilter::from_default_env())
    ```
8.  Set the log level and run the service.

    ```shell
    RUST_LOG=info cargo r
    ```
9.  Request `/health`.

    ```shell
    curl_tests/get_health.sh
    ```
10. The subscriber can also report span creation, closure, and timing, although this workshop does not enable it:

    ```rust
    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    ```
11. Use middleware for request-level tracing. Add tower-http.

    ```shell
    cargo add -F trace tower-http
    ```
12. Add the tracing middleware to the router. The example uses INFO to make the output visible; DEBUG is usually a better production default.

    ```rust
    Router::new().route("/health", get(health)).layer(
        TraceLayer::new_for_http()
            .make_span_with(
                DefaultMakeSpan::new()
                    // .include_headers(true)
                    .level(Level::INFO),
            )
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)),
    )
    ```
13. Add tracing events to `create_session()`.

    ```rust
    let Some(user) = repo.find_by_id(&payload.username) else {
        warn!(username = payload.username, "User not found");
        return Err(StatusCode::UNAUTHORIZED);
    };

    debug!(username = payload.username, "Authenticating user");

    if !user.verify_password(&payload.password) {
        warn!(username = payload.username, "Wrong password for user");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let encoding_key = get_encoding_key().map_err(|err| {
        error!("Encoding key not found: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let token = produce_session_token(&encoding_key, &payload.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SessionResponse { jwt: token }))
    ```
14. Run the service with log level set to info.

    ```shell
    RUST_LOG=info cargo r
    ```
15. Make requests and then inspect the logs. `TraceLayer` creates a request span containing the method, URI, and HTTP version. Handler instrumentation creates a nested span; remove that extra annotation once request middleware provides the context.

    ```shell
    curl_tests/get_health.sh
    curl_tests/new_session.sh
    ```


<a id="org62158f5"></a>

## Implement unified error handling

Replace scattered `unwrap()` and `expect()` calls in request paths with one application error type. Return stable, generic messages to clients while logging diagnostic details internally. Do not expose database schemas, stack traces, or provider details in responses.

1.  Add a new module to `main.rs` that will contain the custom errors.

    ```rust
    mod app_error;
    ```
2.  Add the most commonly used error crates. `thiserror` to derive the standard error traits and `anyhow` to wrap unexpected failures.

    ```shell
    cargo add thiserror anyhow
    ```
3.  In `app_error.rs`, define an `AppError` enum for the failure cases exposed by the application, including `Unauthorized` and `Forbidden`. Keep unexpected implementation failures in the `Internal` catch-all.

    ```rust
    pub enum AppError {
        NotFound(String),
        BadRequest(String),
        Unauthorized(String),
        Forbidden(String),
        ExternalApiError(String),
        Internal(anyhow::Error),
    }
    ```
4.  Add stable display messages for the application errors.

    ```rust
    use thiserror::Error;

    #[derive(Error,Debug)]
    enum AppError {
        #[error("Resource not found: {0}")]
        NotFound(String),
        #[error("Bad request: {0}")]
        BadRequest(String),
        #[error("Unauthorized: {0}")]
        Unauthorized(String),
        #[error("Forbidden: {0}")]
        Forbidden(String),
        #[error("External API error: {0}")]
        ExternalApiError(String),
        #[error("Internal server error: {0}")]
    ```
5.  Derive or implement the conversion from `anyhow` so `?` can propagate internal failures and handlers can focus on the success path.

    ```rust
    Internal(#[from] anyhow::Error),
    ```
6.  Implement `IntoResponse` once. Map each variant to an HTTP status and a stable JSON error body. Log internal details but return only a generic message for internal failures.

    ```rust
    impl IntoResponse for AppError {
        fn into_response(self) -> Response {
            let (status, error_message) = match &self {
                AppError::Internal(e) => {
                    // Log the actual error internally, but hide details from the client
                    error!("Internal error: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An unexpected error occurred".to_string(),
                    )
                }
                AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
                AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
                AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
                AppError::ExternalApiError(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
                AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            };

            let body = Json(json!({
                "error": {
                    "message": error_message,
                    "type": format!("{self:?}").split('(').next().unwrap_or("Unknown"),
                }
            }));

            (status, body).into_response()
        }
    }
    ```
7.  In `router.rs`, use `AppError` as the health handler's error type.

    ```rust
    async fn health() -> Result<Json<HealthData>, AppError> {
    ```
8.  Use it in the session-creation handler as well.

    ```rust
    async fn create_session(
        State(repo): State<Arc<dyn Repository<User>>>,
        Json(payload): Json<SessionRequest>,
    ) -> Result<Json<SessionResponse>, AppError> {
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
    ```
9.  Use `AppError` in `verify_session_token()` in `token.rs` because an invalid token is an application-level unauthorized response. Keep lower-level token and key-loading functions focused on their internal errors.

    ```rust
    pub fn verify_session_token(
        decoding_key: &DecodingKey,
        headers: &HeaderMap,
    ) -> Result<(), AppError> {
        let token = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| {
                AppError::Unauthorized("Missing or invalid authorization header".to_string())
            })?;

        decode::<SessionClaims>(token, decoding_key, &Validation::new(Algorithm::RS256))
            .map(|_| ())
            .map_err(|_| AppError::Unauthorized("Invalid session token".to_string()))
    }
    ```
10. Invalid JSON is rejected by Axum's `Json<SessionRequest>` extractor before `create_session()` runs. Axum reports a `JsonRejection`, not a plain `serde_json::Error`, so the failure would bypass `AppError`. Convert the rejection into `AppError` instead.

    ```rust
    impl From<JsonRejection> for AppError {
        fn from(rejection: JsonRejection) -> Self {
            debug!(error = %rejection, "JSON request rejected");
            Self::BadRequest("Invalid JSON request body".to_string())
        }
    }
    ```
11. Change the extractor parameter to carry the possible rejection:

    ```rust
    async fn create_session(
        State(repo): State<Arc<dyn Repository<User>>>,
        payload: Result<Json<SessionRequest>, JsonRejection>,
    ) -> Result<Json<SessionResponse>, AppError> {
        let Json(payload) = payload?;
    ```
12. Send malformed JSON to the session endpoint and confirm that the response is a generic 400 error.

    ```shell
    curl_tests/malformed_json.sh
    ```


<a id="orgd5a6f82"></a>

# Add OTP with Vonage Verify v2

Add a second authentication factor. The service starts an SMS verification and later exchanges the submitted code for a session JWT.


<a id="orgf6a27af"></a>

## Create a Vonage application

1.  Open <https://dashboard.vonage.com/> and select "Applications" on the left panel.
2.  Create an application:
    -   Enter a name.
    -   Generate a public/private key pair and download `private.key`.
    -   Enable `Verify v2` (no Status URL) and `Network Registry` (with "Test Access") under "Capabilities".
    -   Create the application.
3.  Copy the application ID and add it to the `.env` file using this format.

    ```
    VONAGE_APP_ID="<your-application-id-here>"
    ```
4.  Add the contents of `private.key` to `.env`:

    ```
    VONAGE_SECRET_KEY="<contents-of-private.key-here>"
    ```
5.  In `Network Registry`, add your phone number under `Your numbers` in the `Test numbers` section.
6.  Add the number to `.env` in the required international format. Use the format expected by the API and the workshop configuration, with country code and no spaces, plus sign, or hyphens:

    ```
    VERIFY_PHONE="<your-phone-number-here>"
    ```


<a id="orgd25ec05"></a>

## Replace the first authentication step

1.  Add a module named `vonage.rs` to `main.rs`:

    ```rust
    mod vonage;
    ```
2.  Use `reqwest` for calls to the Vonage APIs:

    ```shell
    cargo add -F json reqwest
    ```
3.  In that module, define a type that owns the shared HTTP client and Vonage operations:

    ```rust
    /// Shared HTTP client and Vonage API operations.
    #[derive(Clone)]
    pub struct VonageClient {
        http_client: reqwest::Client,
    }
    ```
4.  Add a constructor that creates a client with a bounded request timeout:

    ```rust
    impl VonageClient {
        /// Create a Vonage client with a bounded request timeout.
        pub fn new() -> Self {
            let http_client = reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("Unable to create Vonage HTTP client");
            Self { http_client }
        }
    }
    ```
5.  The service authenticates to the Vonage application with a short-lived JWT. Add its claims structure to `token.rs`:

    ```rust
    /// JWT claims for interacting with the Vonage APIs.
    #[derive(Serialize)]
    struct VonageClaims {
        /// Corresponds to the ID of the application with Network Registry capabilities, created in the user's dashboard.
        application_id: String,
        /// The time at which the token was issued
        iat: u64,
        /// The time at which the token should expire
        exp: u64,
        /// A unique identifier for the JWT
        jti: String,
    }
    ```
6.  Add uuid for the unique `jti` claim:

    ```shell
    cargo add -F v4 uuid
    ```
7.  Add a function that creates the Vonage JWT:

    ```rust
    /// Create a JWT for using the Vonage APIs.
    pub fn produce_vonage_token() -> anyhow::Result<String> {
        const VONAGE_EXPIRY_SECS: u64 = 600;

        let secret = std::env::var("VONAGE_SECRET_KEY")?;
        let encoding_key = EncodingKey::from_rsa_pem(secret.as_bytes())?;
        let app_id = std::env::var("VONAGE_APP_ID")?;
        let now = get_now_in_secs();
        let vonage_claims = VonageClaims {
            iat: now,
            application_id: app_id,
            jti: Uuid::new_v4().to_string(),
            exp: now + VONAGE_EXPIRY_SECS,
        };
        Ok(encode(
            &Header::new(Algorithm::RS256),
            &vonage_claims,
            &encoding_key,
        )?)
    }
    ```
8.  Add the user's phone number to the `User` structure in `user.rs`.

    ```rust
    pub struct User {
        pub username: String,
        hashed_password: String,
        pub phone: String,
    }
    ```
9.  Update the `new()` constructor to accept the phone number.

    ```rust
    pub fn new(username: impl Into<String>, password: &str, phone: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            hashed_password: bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap(),
            phone: phone.into(),
        }
    }
    ```
10. Update the demo-user seed data in `main.rs`.

    ```rust
    users.add(User::new(
        "alice",
        "secret123",
        std::env::var("VERIFY_PHONE").unwrap_or("34600123456".to_string()),
    ));
    ```
11. Change the `create_session()` response in `routes.rs` to return the Verify request ID:

    ```rust
    #[derive(Serialize)]
    struct SessionResponse {
        /// Used to provide 2FA code.
        verify_request_id: String,
    }
    ```
12. Define the Verify workflow, request, and response structures in `vonage.rs`. The Verify API returns the request ID:

    ```rust
    // Verify V2
    #[derive(Serialize)]
    struct VerifyWorkflow {
        channel: &'static str,
        to: String,
    }

    #[derive(Serialize)]
    struct VerifyRequest {
        brand: &'static str,
        workflow: Vec<VerifyWorkflow>,
        channel_timeout: u16,
    }

    #[derive(Deserialize)]
    struct VerifyResponse {
        request_id: String,
    }
    ```
13. Add a `VonageClient` method that starts an OTP verification and returns its request ID:

    ```rust
    /// Start a Verify request and return its request ID.
    pub async fn start_verification(
        &self,
        vonage_token: &str,
        phone_number: &str,
    ) -> Result<String, AppError> {
    }
    ```
14. Build the Verify request body:

    ```rust
    let verify_body = VerifyRequest {
        brand: "Rock Solid",
        workflow: vec![VerifyWorkflow {
            channel: "sms",
            to: phone_number.to_owned(),
        }],
        channel_timeout: 600,
    };
    ```
15. Define the Verify v2 endpoint:

    ```rust
    const VERIFY_ENDPOINT: &str = "https://api.nexmo.com/v2/verify";
    ```
16. Send the authenticated request:

    ```rust
    let response = self
        .http_client
        .post(VERIFY_ENDPOINT)
        .bearer_auth(vonage_token)
        .json(&verify_body)
        .send()
        .await?;
    ```
17. Reject unsuccessful provider responses without exposing provider details to the client:

    ```rust
    if !response.status().is_success() {
        debug!("Verification provider rejected request with response: {response:?}");
        return Err(AppError::ExternalApiError(
            "Verification provider rejected the request".to_string(),
        ));
    }
    ```
18. For a successful response, deserialize the body and return the request ID:

    ```rust
    let verify_data: VerifyResponse = response.json().await?;
    debug!("REQUEST_ID=\"{}\"", verify_data.request_id);
    Ok(verify_data.request_id)
    ```
19. Implement `From<reqwest::Error>` in `app_error.rs` so request failures become application errors:

    ```rust
    impl From<reqwest::Error> for AppError {
        fn from(error: reqwest::Error) -> Self {
            error!("External verification API error: {error}");
            Self::ExternalApiError("API request failed".to_string())
        }
    }
    ```
20. Define application state in `routes.rs`. It contains the repository and the shared Vonage client:

    ```rust
    #[derive(Clone)]
    struct AppState {
        repo: Arc<dyn Repository<User>>,
        vonage: VonageClient,
    }
    ```
21. Construct that state in `app()`:

    ```rust
    let state = AppState {
        repo,
        vonage: VonageClient::new(),
    };
    ```
22. Attach the state to the router:

    ```rust
    .with_state(state)
    ```
23. Update the `create_session()` signature to extract `AppState`:

    ```rust
    State(state): State<AppState>,
    ```
24. Use the repository from the extracted state to find the user:

    ```rust
    let Some(user) = state.repo.find_by_id(&payload.username) else {
    ```
25. Use the same state to call Verify after the password has been checked:

    ```rust
    let vonage_token = produce_vonage_token()?;
    let verify_request_id = state
        .vonage
        .start_verification(&vonage_token, &user.phone)
        .await?;
    Ok(Json(SessionResponse { verify_request_id }))
    ```
26. Run the service with:

    ```shell
    RUST_LOG=rock_solid=debug cargo r
    ```
27. Run the session test. It should return a verification request ID. The next subsection adds the endpoint that sends the user-entered code back to Vonage.

    ```shell
    curl_tests/new_session.sh
    ```


<a id="org1db9b7f"></a>

## Verify the OTP

1.  Define the request body in `vonage.rs` for the code received by SMS:

    ```rust
    #[derive(Serialize)]
    struct CheckCodeRequest {
        code: String,
    }
    ```
2.  Add a `VonageClient` method that checks the received code:

    ```rust
    /// Check the verification code that a user has received.
    pub async fn validate_verification(
        &self,
        vonage_token: &str,
        request_id: &str,
        code: &str,
    ) -> Result<(), AppError> {
    }
    ```
3.  Include the request ID in the Verify endpoint path and send the code:

    ```rust
    let response = self
        .http_client
        .post(format!("{VERIFY_ENDPOINT}/{request_id}"))
        .bearer_auth(vonage_token)
        .json(&CheckCodeRequest {
            code: code.to_owned(),
        })
        .send()
        .await?;
    ```
4.  Return `Ok(())` only when the provider accepts the code:

    ```rust
    if !response.status().is_success() {
        debug!("Verification provider rejected validation request with response: {response:?}");
        return Err(AppError::ExternalApiError(
            "Verification provider rejected the validation".to_string(),
        ));
    }

    Ok(())
    ```
5.  Define the application request and response for the validation endpoint. The workshop example accepts the username from the client for simplicity. This is intentionally insecure; production code must bind the challenge to server-side state and never trust a client-supplied username:

    ```rust
    /// `username` should never be accepted from the client. The previous step should store pending challenges with some
    /// expiration deadline.
    #[derive(Deserialize)]
    struct ValidateCodeRequest {
        username: String, // WARN: This is not a secure implementation!!!
        request_id: String,
        code: String,
    }

    #[derive(Serialize)]
    struct ValidateCodeResponse {
        token: String,
    }
    ```
6.  Add the validation endpoint handler:

    ```rust
    /// Complete 2FA for the session.
    async fn validate_code(
        State(state): State<AppState>,
        payload: Result<Json<ValidateCodeRequest>, JsonRejection>,
    ) -> Result<(StatusCode, Json<ValidateCodeResponse>), AppError> {
    }
    ```
7.  Use `VonageClient` to validate the code, then issue a session JWT:

    ```rust
    let Json(payload) = payload?;

    let vonage_token = produce_vonage_token()?;
    state
        .vonage
        .validate_verification(&vonage_token, &payload.request_id, &payload.code)
        .await?;

    debug!(user = %payload.username, "Successful verification");
    let encoding_key = get_encoding_key()?;
    let token = produce_session_token(&encoding_key, &payload.username)?;

    Ok((StatusCode::CREATED, Json(ValidateCodeResponse { token })))
    ```
8.  Add the handler to the router:

    ```rust
    .route("/session/verification", post(validate_code))
    ```
9.  Run the service with:

    ```shell
    RUST_LOG=rock_solid=debug cargo r
    ```
10. Start a login to send the OTP:

    ```shell
    curl_tests/new_session.sh
    ```
11. Send the received code for validation:

    ```shell
    REQUEST_ID="xx" CODE="XX" curl_tests/validate_code.sh
    ```


<a id="org6fe986a"></a>

## Reduce risk with Identity Insights

Identity Insights provides carrier-derived signals that can help detect fraud. These signals are passive and harder to spoof than data supplied by the user. Request only the insights needed for the decision, such as number format, carrier, SIM swap, number recycling, device swap, and scam signals.

Use approved test numbers or the Virtual Operator test environment when available so attendees can run the exercise without live traffic. Discuss how the same check can protect login, onboarding, account recovery, and high-value transactions.

1.  In the `vonage` module, add a method that checks the phone-number risk:

    ```rust
    /// Check whether a phone number has had a recent SIM swap.
    ///
    /// This function fails closed: no OTP is sent when the insight reports a
    /// recent swap, an unsupported number, or an unavailable provider result.
    pub async fn check_sim_swap(
        &self,
        vonage_token: &str,
        phone_number: &str,
    ) -> Result<(), AppError> {
    }
    ```
2.  Define the Identity Insights endpoints for Europe and the United States:

    ```rust
    const EU_ENDPOINT: &str = "https://api-eu.vonage.com/identity-insights/v1/requests";
    const US_ENDPOINT: &str = "https://api-us.vonage.com/identity-insights/v1/requests";
    ```
3.  Define the request structures:

    ```rust
    #[derive(Serialize)]
    struct IdentityInsightsRequest {
        phone_number: String,
        purpose: &'static str,
        insights: InsightsRequest,
    }

    #[derive(Serialize)]
    struct InsightsRequest {
        sim_swap: SimSwapRequest,
    }

    #[derive(Serialize)]
    struct SimSwapRequest {
        period: u16,
    }
    ```
4.  Define the fraud-prevention purpose and the SIM-swap look-back period:

    ```rust
    const FRAUD_PREVENTION_PURPOSE: &str = "FraudPreventionAndDetection";
    /// The SIM Swap API's lookback period, in hours.
    const SIM_SWAP_PERIOD_HOURS: u16 = 24 * 7;
    ```
5.  Build the request with those structures:

    ```rust
    let request = IdentityInsightsRequest {
        // phone_number: "990123455".to_owned(),
        phone_number: phone_number.to_owned(),
        purpose: FRAUD_PREVENTION_PURPOSE,
        insights: InsightsRequest {
            sim_swap: SimSwapRequest {
                period: SIM_SWAP_PERIOD_HOURS,
            },
        },
    };
    ```
6.  Send the request with the shared HTTP client:

    ```rust
    let response = self
        .http_client
        .post(identity_insights_endpoint()?)
        .bearer_auth(vonage_token)
        .json(&request)
        .send()
        .await?;
    ```
7.  Select the regional endpoint from an environment variable:

    ```rust
    /// Returns the Identity Insights endpoint. By default the European endpoint is returned, but it can be changed using
    /// the environment variable `VONAGE_IDENTITY_INSIGHTS_REGION` to force the US one ("us") or the European one ("eu")
    fn identity_insights_endpoint() -> Result<&'static str, AppError> {
        match std::env::var("VONAGE_IDENTITY_INSIGHTS_REGION").as_deref() {
            Ok("eu") | Err(VarError::NotPresent) => Ok(EU_ENDPOINT),
            Ok("us") => Ok(US_ENDPOINT),
            Ok(region) => Err(AppError::Internal(anyhow::anyhow!(
                "Unsupported VONAGE_IDENTITY_INSIGHTS_REGION: {region}"
            ))),
            Err(error) => Err(AppError::Internal(anyhow::anyhow!(
                "Unable to read VONAGE_IDENTITY_INSIGHTS_REGION: {error}"
            ))),
        }
    }
    ```
8.  First check whether the provider accepted the request at the HTTP level:

    ```rust
    if !response.status().is_success() {
        warn!(status = %response.status(), "Identity Insights rejected SIM swap request");
        return Err(AppError::ExternalApiError(
            "Phone risk check was unavailable".to_string(),
        ));
    }
    ```
9.  The response also contains a status for each insight. Define response structures for those fields:

    ```rust
    #[derive(Deserialize)]
    struct IdentityInsightsResponse {
        insights: InsightsResponse,
    }

    #[derive(Deserialize)]
    struct InsightsResponse {
        sim_swap: SimSwapResponse,
    }

    #[derive(Deserialize)]
    struct SimSwapResponse {
        is_swapped: Option<bool>,
        status: InsightStatus,
    }

    #[derive(Deserialize)]
    struct InsightStatus {
        code: String,
    }
    ```
10. Deserialize the response and require a successful insight status:

    ```rust
    let data: IdentityInsightsResponse = response.json().await.map_err(|error| {
        warn!(error = %error, "Identity Insights returned an invalid response body");
        AppError::ExternalApiError("Phone risk check returned an invalid response".to_string())
    })?;
    let sim_swap = data.insights.sim_swap;

    if sim_swap.status.code != "OK" {
        warn!(
            status_code = %sim_swap.status.code,
            "Identity Insights returned an unsuccessful SIM swap result"

        );
        return Err(AppError::ExternalApiError(
            "Phone risk check was unavailable".to_string(),
        ));
    }
    ```
11. Fail closed. Continue only when the provider explicitly confirms that the SIM has not been swapped:

    ```rust
    match sim_swap.is_swapped {
        Some(true) => Err(AppError::Forbidden(
            "Unable to send a verification code to this phone number".to_string(),
        )),
        Some(false) => Ok(()),
        None => {
            warn!("Identity Insights returned OK without an is_swapped result");
            Err(AppError::ExternalApiError(
                "Phone risk check returned an incomplete response".to_string(),
            ))
        }
    }
    ```
12. Call this check from the `/session` endpoint before sending an OTP:

    ```rust
    // Check the phone risk before creating a Verify request. This keeps the
    // OTP from being sent when a recent SIM swap is reported.
    state
        .vonage
        .check_sim_swap(&vonage_token, &user.phone)
        .await?;
    ```
13. Run the service again:

    ```shell
    RUST_LOG=rock_solid=debug cargo r
    ```
14. Start a login and inspect the risk-check result:

    ```shell
    curl_tests/new_session.sh
    ```
15. If the check allows the flow, validate the received code:

    ```shell
    REQUEST_ID="xx" CODE="XX" curl_tests/validate_code.sh
    ```


<a id="org6f90610"></a>

# Implement authorization with middleware

Split the router into public routes for login and OTP initiation, and protected routes for application data. Apply middleware to the protected branch instead of repeating JWT checks in every handler. Use tower-http for standard concerns such as tracing, timeouts, and CORS rather than implementing them from scratch. Explain layering order: layers wrap one another, and the last layer added is the first to receive the request.


<a id="org0e81cda"></a>

## Validate an individual endpoint

1.  A session JWT now exists, but no endpoint uses it yet. Define the response type for a protected endpoint.

    ```rust
    #[derive(Serialize)]
    struct ApplicationData {
        message: &'static str,
    }
    ```
2.  Add the handler for the protected endpoint:

    ```rust
    /// Example endpoint protected by the session middleware.
    async fn application() -> Json<ApplicationData> {
        Json(ApplicationData {
            message: "Application data",
        })
    }
    ```
3.  Connect the `application` handler to `=/application=` in the router:

    ```rust
    .route("/application", get(application))
    ```
4.  Change the handler to return a `Result`:

    ```rust
    async fn application(headers: HeaderMap) -> Result<Json<ApplicationData>, AppError> {
    }
    ```
5.  Use the token helpers from `token.rs`:

    ```rust
    let decoding_key = get_decoding_key()?;
    verify_session_token(&decoding_key, &headers)?;
    Ok(Json(ApplicationData {
        message: "Application data",
    }))
    ```
6.  Confirm that the endpoint rejects requests without a valid bearer JWT:

    ```shell
    SESSION_JWT="none" curl_tests/access_protected.sh
    ```


<a id="org8f28067"></a>

## Protect the private router with middleware

1.  Checking every handler individually is repetitive and easy to get wrong. Create a second router for routes that require authentication:

    ```rust
    let protected_routes = Router::new()
        .route("/application", get(application));
    ```
2.  Merge the protected router with the public router:

    ```rust
    .merge(protected_routes)
    ```
3.  Define middleware that extracts the bearer token and verifies it. The example only needs a valid token; a later feature could extract claims for authorization decisions:

    ```rust
    /// Require a valid application session before running a protected handler.
    async fn require_session(
        headers: HeaderMap,
        request: Request,
        next: Next,
    ) -> Result<Response, AppError> {
        let decoding_key = get_decoding_key()?;
        verify_session_token(&decoding_key, &headers)?;
        Ok(next.run(request).await)
    }
    ```
4.  Add the middleware as a route layer on the protected router:

    ```rust
    .route_layer(middleware::from_fn(require_session));
    ```
5.  Run the service as before:

    ```shell
    RUST_LOG=rock_solid=debug cargo r
    ```
6.  Confirm that public endpoints such as `/health` remain accessible without a token:

    ```shell
    curl_tests/get_health.sh
    ```
7.  Confirm that the protected endpoint rejects unauthenticated requests:

    ```shell
    SESSION_JWT="none" curl_tests/access_protected.sh
    ```
8.  Obtain a session JWT through the two-step authentication flow:

    ```shell
    curl_tests/new_session.sh
    ```
9.  Validate the received code to obtain the JWT:

    ```shell
    REQUEST_ID="xx" CODE="XX" curl_tests/validate_code.sh
    ```
10. Send the JWT as a bearer token to access the protected endpoint:

    ```shell
    SESSION_JWT="xx" curl_tests/access_protected.sh
    ```


<a id="org672f368"></a>

## Hardening and production concerns


<a id="orge25c482"></a>

### Timeouts

1.  Add the timeout feature to tower-http.

    ```shell
    cargo add -F timeout tower-http
    ```
2.  In `routes.rs`, add a global request timeout near the other middleware layers. Return 408 when a request exceeds the limit:

    ```rust
    .layer(TimeoutLayer::with_status_code(
        StatusCode::REQUEST_TIMEOUT,
        Duration::from_secs(15),
    ))
    ```


<a id="orgb85931b"></a>

### Cross-Origin Resource Sharing (CORS)

Apply CORS globally in `app()` because browser access applies to both public and protected endpoints.

1.  Add the relevant feature to tower-http:

    ```shell
    cargo add -F cors tower-http
    ```
2.  Create a CORS middleware layer with an explicit allow-list:

    ```rust
    .layer(CorsLayer::new()
        .allow_origin(allowed_origin) // Allow only known frontend origins.
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]))
    ```
3.  Keep these rules in mind:
    -   Do not use `Any` in production.
    -   The workshop uses bearer tokens rather than cookies, so `allow_credentials(true)` is unnecessary.
    -   Ensure rejected responses such as 401 and 429 also receive the intended CORS headers.
    -   CORS is not authentication. Non-browser clients can ignore it.


<a id="orgf4fe0eb"></a>

### Rate limiting

Use middleware for coarse request-level limits:

-   Apply a global limit to protect the process.
-   Apply stricter route-specific limits to `/session` and `/session/verification`.
-   Limit expensive operations such as password hashing and Vonage API calls separately.
-   Add a concurrency limit or timeout so requests cannot hold resources indefinitely.

A possible router structure is:

```rust
let session_routes = Router::new()
    .route("/session", post(create_session))
    .route("/session/verification", post(validate_code))
    .layer(session_rate_limit);

Router::new()
    .route("/health", get(health))
    .merge(session_routes)
    .merge(protected_routes)
    .layer(global_rate_limit);
```
