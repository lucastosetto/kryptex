//! Perptrix API Server
//!
//! HTTP API server with health check, metrics, and business logic endpoints.
//! This service is stateless and can be horizontally scaled.
//! WebSocket service runs as a separate process.

use dotenvy::dotenv;
use perptrix::core::http::start_server;
use perptrix::logging;
use std::env;
use tokio::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env if present
    dotenv().ok();

    // Initialize logging based on environment
    logging::init_logging();
    
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let env = perptrix::config::get_environment();
    info!("Starting Perptrix API Server");
    info!(environment = %env, "Environment");
    info!(port = port, "HTTP Server: http://0.0.0.0:{}", port);
    info!("This service is stateless and can be horizontally scaled");

    // Start HTTP server
    let server_handle = tokio::spawn(async move {
        if let Err(e) = start_server(port).await {
            error!(error = %e, "HTTP server error");
        }
    });

    // Graceful shutdown
    info!("API server started, waiting for shutdown signal...");
    info!("Note: WebSocket service runs as separate process. Use 'cargo run --bin websocket-service' to start it.");
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Shutting down API server...");
            info!("API server stopped");
        }
        _ = server_handle => {
            error!("HTTP server stopped");
        }
    }

    Ok(())
}




