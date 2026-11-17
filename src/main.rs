use std::sync::Arc;

use tokio::net::TcpListener;

use crate::{
    repository::{InMemoryRepository, Repository},
    routes::app,
    user::User,
};

mod repository;
mod routes;
mod user;

#[tokio::main]
async fn main() {
    const SERVER_ADDR: &str = "127.0.0.1:8080";

    let repo: Arc<dyn Repository<User>> = Arc::new(seed_users());

    println!("Launching server: http://{SERVER_ADDR}/");
    let listener = TcpListener::bind(SERVER_ADDR)
        .await
        .expect("Unable to create listener");
    axum::serve(listener, app(repo)).await.unwrap();
}

/// Create a user repository for demo purposes.
fn seed_users() -> InMemoryRepository<User> {
    let mut users = InMemoryRepository::new();
    users.add(User::new("alice", "secret123"));

    users
}
