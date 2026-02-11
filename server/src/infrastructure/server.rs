use axum::{routing::get, Router};
use std::sync::Arc;

use crate::app::AppState;
use super::websocket::ws_handler;

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Create and configure the Axum application router
pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(ws_handler))
        .with_state(state)
}

/// Start the server on the specified address
pub async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Battlestar server...");

    // Create application state
    let app_state = AppState::new();

    // Create router
    let app = create_app(app_state);

    // Bind and serve
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
