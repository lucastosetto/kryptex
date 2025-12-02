//! Strategy builder system data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

/// Main strategy entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: Option<i64>,
    pub name: String,
    pub symbol: String,
    pub config: StrategyConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Main strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub rules: Vec<Rule>,
    pub aggregation: AggregationConfig,
}

/// Individual condition or group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    #[serde(rename = "type")]
    pub rule_type: RuleType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<LogicalOperator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Rule>>,
}

/// Rule type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RuleType {
    Condition,
    Group,
    WeightedGroup,
}

/// Indicator comparison condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub indicator: IndicatorType,
    #[serde(default)]
    pub indicator_params: HashMap<String, Value>,
    pub comparison: Comparison,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_state: Option<String>, // Indicator-specific signal state (e.g., "Oversold", "BullishCross")
}

/// Available indicator types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IndicatorType {
    MACD,
    RSI,
    EMA,
    SuperTrend,
    Bollinger,
    ATR,
    OBV,
    VolumeProfile,
    FundingRate,
    OpenInterest,
}

/// Comparison operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Comparison {
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Equal,
    NotEqual,
    InRange,
    SignalState,
}

/// Logical operators for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogicalOperator {
    AND,
    OR,
}

/// How to combine rule results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    pub method: AggregationMethod,
    pub thresholds: SignalThresholds,
}

/// Aggregation methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AggregationMethod {
    Sum,
    WeightedSum,
    Majority,
    All,
    Any,
}

/// Score thresholds for signal generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalThresholds {
    pub long_min: i32,
    pub short_max: i32,
}

/// Result of evaluating a rule
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub rule_id: String,
    pub passed: bool,
    pub score: i32,
    pub weight: f64,
}

impl RuleResult {
    pub fn new(rule_id: String, passed: bool, score: i32, weight: f64) -> Self {
        Self {
            rule_id,
            passed,
            score,
            weight,
        }
    }
}

