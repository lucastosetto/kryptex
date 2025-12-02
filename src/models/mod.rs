//! Shared data models spanning the engine layers.

pub mod indicators;
pub mod signal;
pub mod strategy;

pub use indicators::{
    EmaIndicator, IndicatorSet, MacdIndicator, RsiIndicator, SmaIndicator, VolumeIndicator,
};
pub use signal::{SignalDirection, SignalEvaluation, SignalOutput, SignalReason};
pub use strategy::{
    AggregationConfig, AggregationMethod, Condition, Comparison, IndicatorType, LogicalOperator,
    Rule, RuleResult, RuleType, SignalThresholds, Strategy, StrategyConfig,
};
