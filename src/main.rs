use tokio::net::TcpListener;

use crate::routes::app;

mod routes;

#[tokio::main]
async fn main() {
    const SERVER_ADDR: &str = "127.0.0.1:8080";

    println!("Launching server: http://{SERVER_ADDR}/");
    let listener = TcpListener::bind(SERVER_ADDR)
        .await
        .expect("Unable to create listener");
    axum::serve(listener, app()).await.unwrap();
}
