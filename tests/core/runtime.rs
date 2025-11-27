//! Unit tests for signal runtime

use perptrix::core::runtime::{RuntimeConfig, SignalRuntime};

#[test]
fn test_runtime_config_default() {
    let config = RuntimeConfig::default();
    assert_eq!(config.evaluation_interval_seconds, 60);
    assert_eq!(config.symbols.len(), 1);
}

#[test]
fn test_runtime_creation() {
    let config = RuntimeConfig::default();
    let _runtime = SignalRuntime::new(config);
    // Verify runtime was created successfully (no panic)
}

