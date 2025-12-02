//! Unit tests for signal engine

use chrono::Utc;
use perptrix::models::indicators::Candle;
use perptrix::models::strategy::{
    AggregationConfig, AggregationMethod, Condition, Comparison, IndicatorType, Rule, RuleType,
    SignalThresholds, Strategy, StrategyConfig,
};
use perptrix::signals::engine::SignalEngine;

fn create_test_strategy(symbol: &str) -> Strategy {
    // Create a simple strategy with a rule that will always pass
    // This allows tests to verify the evaluation pipeline works
    Strategy {
        id: None,
        name: "Test Strategy".to_string(),
        symbol: symbol.to_string(),
        config: StrategyConfig {
            rules: vec![Rule {
                id: "test_rule".to_string(),
                rule_type: RuleType::Condition,
                weight: Some(1.0),
                operator: None,
                condition: Some(Condition {
                    indicator: IndicatorType::RSI,
                    indicator_params: std::collections::HashMap::new(),
                    comparison: Comparison::GreaterThan,
                    threshold: Some(-100.0), // Always true (RSI is 0-100)
                    signal_state: None,
                }),
                children: None,
            }],
            aggregation: AggregationConfig {
                method: AggregationMethod::Sum,
                thresholds: SignalThresholds {
                    long_min: 1, // Lower threshold so tests can pass
                    short_max: -1,
                },
            },
        },
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn create_uptrend_candles(count: usize) -> Vec<Candle> {
    let mut candles = Vec::new();
    for i in 0..count {
        let price = 100.0 + (i as f64 * 0.5);
        let candle = Candle::new(
            price,
            price + 0.3,
            price - 0.2,
            price + 0.1,
            1000.0,
            Utc::now(),
        )
        .with_open_interest(10_000.0 + (i as f64 * 20.0))
        .with_funding_rate(0.0001);
        candles.push(candle);
    }
    candles
}

#[test]
fn test_evaluate_insufficient_data() {
    let candles = create_uptrend_candles(10);
    let strategy = create_test_strategy("BTC");
    assert!(SignalEngine::evaluate(&candles, &strategy).is_none());
}

#[test]
fn test_evaluate_sufficient_data() {
    let candles = create_uptrend_candles(250);
    let strategy = create_test_strategy("BTC");
    let result = SignalEngine::evaluate(&candles, &strategy);
    assert!(result.is_some());
    let signal = result.unwrap();
    assert!(signal.confidence >= 0.0);
    assert!(signal.confidence <= 1.0);
}



