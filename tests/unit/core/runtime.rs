//! Unit tests for signal runtime

use perptrix::core::runtime::RuntimeConfig;

#[test]
fn test_runtime_config_default() {
    let config = RuntimeConfig::default();
    assert_eq!(config.evaluation_interval_seconds, 60);
    assert_eq!(config.symbols.len(), 1);
}

// Note: SignalRuntime::new() now requires job_context and storage backends
// This test is skipped as it requires async setup and dependencies
// Integration tests cover runtime creation



