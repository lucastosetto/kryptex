//! Integration tests - test the system end-to-end
//!
//! Tests are organized by service:
//! - api_server: HTTP API endpoints and business logic
//! - websocket_service: WebSocket connection and data ingestion
//! - worker: Job processing and workflow execution

#[path = "integration/api_server.rs"]
mod api_server;

#[path = "integration/websocket_service.rs"]
mod websocket_service;

#[path = "integration/worker.rs"]
mod worker;

// Legacy integration tests (can be migrated to api_server)
#[path = "integration/hyperliquid.rs"]
mod hyperliquid;
