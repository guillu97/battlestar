mod constants;
mod game;
mod handlers;
mod state;

use axum::{routing::get, Router};
use state::AppState;

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Battlestar server...");
    let app_state = AppState::new();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(handlers::ws_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
