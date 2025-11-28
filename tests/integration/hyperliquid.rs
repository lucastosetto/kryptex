//! Integration tests for the Hyperliquid-powered HTTP stack.
mod test_utils;

use perptrix::services::market_data::MarketDataProvider;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use test_utils::TestApp;

#[tokio::test]
async fn health_endpoint_reports_healthy_status() {
    let app = TestApp::new().await;
    let response = app.server.get("/health").await;
    assert_eq!(response.status_code(), 200);

    let body: Value = response.json();
    assert_eq!(body["status"], "healthy");
    assert!(body["uptime_seconds"].as_u64().is_some());
    assert_eq!(body["service"], "perptrix-signal-engine");
}

#[tokio::test]
async fn metrics_endpoint_exposes_prometheus_metrics() {
    let app = TestApp::new().await;
    let response = app.server.get("/metrics").await;
    assert_eq!(response.status_code(), 200);

    let body = response.text();
    assert!(
        body.contains("http_requests_total"),
        "Expected Prometheus metrics output"
    );
}

#[tokio::test]
async fn subscriptions_use_mocked_dependencies() {
    let app = TestApp::new().await;

    app.provider.subscribe("BTC").await.expect("subscribe succeeds");

    let requests = app
        .hyperliquid_rest
        .received_requests()
        .await
        .expect("wiremock requests");
    assert!(
        requests.iter().any(|req| {
            let body = String::from_utf8_lossy(&req.body);
            body.contains("candleSnapshot")
        }),
        "Expected candleSnapshot request routed through wiremock"
    );

    let sent_messages = app.websocket.sent_messages().await;
    assert!(
        sent_messages.iter().any(|message| match message {
            Message::Text(payload) => payload.contains("subscribe"),
            _ => false,
        }),
        "Expected subscription payload sent through mocked WebSocket"
    );
}

