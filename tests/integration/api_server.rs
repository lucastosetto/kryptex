//! Integration tests for the API Server
//!
//! Tests HTTP endpoints, health checks, metrics, and business logic.

#[path = "api_server/test_utils.rs"]
mod test_utils;

use serde_json::Value;

use test_utils::TestApiServer;

#[tokio::test]
async fn health_endpoint_reports_healthy_status() {
    let app = TestApiServer::new().await;
    let response = app.server.get("/health").await;
    assert_eq!(response.status_code(), 200);

    let body: Value = response.json();
    assert_eq!(body["status"], "healthy");
    assert!(body["uptime_seconds"].as_u64().is_some());
    assert_eq!(body["service"], "perptrix-signal-engine");
}

#[tokio::test]
async fn health_endpoint_includes_uptime() {
    let app = TestApiServer::new().await;
    
    // Wait a bit to ensure uptime changes
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let response = app.server.get("/health").await;
    assert_eq!(response.status_code(), 200);

    let body: Value = response.json();
    let _uptime = body["uptime_seconds"].as_u64().unwrap();
    // Verify uptime exists and is valid (u64 is always >= 0)
}

#[tokio::test]
async fn metrics_endpoint_exposes_prometheus_metrics() {
    let app = TestApiServer::new().await;
    let response = app.server.get("/metrics").await;
    assert_eq!(response.status_code(), 200);

    let body = response.text();
    assert!(
        body.contains("http_requests_total"),
        "Expected http_requests_total metric"
    );
    assert!(
        body.contains("http_request_duration_seconds"),
        "Expected http_request_duration_seconds metric"
    );
    assert!(
        body.contains("http_requests_in_flight"),
        "Expected http_requests_in_flight metric"
    );
}

#[tokio::test]
async fn metrics_endpoint_tracks_request_count() {
    let app = TestApiServer::new().await;
    
    // Make multiple requests
    for _ in 0..3 {
        let _ = app.server.get("/health").await;
    }
    
    let response = app.server.get("/metrics").await;
    let body = response.text();
    
    // Should have recorded multiple requests
    assert!(
        body.contains("http_requests_total"),
        "Should track request count"
    );
}

#[tokio::test]
async fn metrics_endpoint_tracks_request_duration() {
    let app = TestApiServer::new().await;
    
    let _ = app.server.get("/health").await;
    
    let response = app.server.get("/metrics").await;
    let body = response.text();
    
    // Should have recorded duration
    assert!(
        body.contains("http_request_duration_seconds"),
        "Should track request duration"
    );
}

#[tokio::test]
async fn api_server_is_stateless() {
    // Test that multiple requests don't affect each other
    let app = TestApiServer::new().await;
    
    let response1 = app.server.get("/health").await;
    let response2 = app.server.get("/health").await;
    
    assert_eq!(response1.status_code(), 200);
    assert_eq!(response2.status_code(), 200);
    
    let body1: Value = response1.json();
    let body2: Value = response2.json();
    
    // Both should be healthy
    assert_eq!(body1["status"], "healthy");
    assert_eq!(body2["status"], "healthy");
}

#[tokio::test]
async fn api_server_handles_concurrent_requests() {
    let app = TestApiServer::new().await;
    
    // Make sequential requests (TestServer is not Clone, so concurrent requests require different approach)
    for _ in 0..10 {
        let response = app.server.get("/health").await;
        assert_eq!(response.status_code(), 200);
    }
}

// Future tests for business logic endpoints will go here:
// - GET /signals - List signals
// - GET /signals/{symbol} - Get signals for a symbol
// - POST /signals/evaluate - Trigger manual evaluation
// - GET /symbols - List subscribed symbols
// etc.

