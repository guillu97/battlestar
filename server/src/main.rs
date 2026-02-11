mod constants;
mod app;
mod domain;
mod simulation;
mod infrastructure;

use infrastructure::run_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_server("0.0.0.0:3000").await
}
