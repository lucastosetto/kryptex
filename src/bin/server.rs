//! Perptrix Signal Engine Server
//!
//! Starts the HTTP server with health check endpoint and optionally
//! runs periodic signal evaluation.

use perptrix::core::http::start_server;
use perptrix::core::runtime::{RuntimeConfig, SignalRuntime};
use std::env;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let eval_interval: u64 = env::var("EVAL_INTERVAL_SECONDS")
        .ok()
        .and_then(|i| i.parse().ok())
        .unwrap_or(0);

    let symbols: Option<Vec<String>> = env::var("SYMBOLS")
        .ok()
        .map(|s| {
            let v: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        })
        .flatten();

    println!("Starting Perptrix Signal Engine Server");
    println!("  HTTP Server: http://0.0.0.0:{}", port);
    
    if eval_interval > 0 {
        let symbols = symbols.ok_or("SYMBOLS environment variable is required when EVAL_INTERVAL_SECONDS > 0")?;
        println!("  Signal Evaluation: every {} seconds", eval_interval);
        println!("  Symbols: {}", symbols.join(", "));
        
        let server_handle = tokio::spawn(async move {
            if let Err(e) = start_server(port).await {
                eprintln!("HTTP server error: {}", e);
            }
        });

        let runtime_config = RuntimeConfig {
            evaluation_interval_seconds: eval_interval,
            symbols,
        };
        let runtime = SignalRuntime::new(runtime_config);

        let runtime_handle = tokio::spawn(async move {
            if let Err(e) = runtime.run().await {
                eprintln!("Signal runtime error: {}", e);
            }
        });

        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("\nShutting down...");
            }
            _ = server_handle => {
                eprintln!("HTTP server stopped");
            }
            _ = runtime_handle => {
                eprintln!("Signal runtime stopped");
            }
        }
    } else {
        println!("  Signal Evaluation: disabled (set EVAL_INTERVAL_SECONDS to enable)");
        
        let server_handle = tokio::spawn(async move {
            if let Err(e) = start_server(port).await {
                eprintln!("HTTP server error: {}", e);
            }
        });

        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("\nShutting down...");
            }
            _ = server_handle => {
                eprintln!("HTTP server stopped");
            }
        }
    }

    Ok(())
}
