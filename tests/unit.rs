//! Unit tests - organized by module structure

#[path = "common/math.rs"]
mod common_math;

#[path = "indicators/momentum/macd.rs"]
mod indicators_momentum_macd;

#[path = "indicators/momentum/rsi.rs"]
mod indicators_momentum_rsi;

#[path = "indicators/trend/ema.rs"]
mod indicators_trend_ema;

#[path = "indicators/trend/adx.rs"]
mod indicators_trend_adx;

#[path = "indicators/volatility/bollinger.rs"]
mod indicators_volatility_bollinger;

#[path = "indicators/volatility/atr.rs"]
mod indicators_volatility_atr;

#[path = "indicators/structure/supertrend.rs"]
mod indicators_structure_supertrend;

#[path = "indicators/structure/support_resistance.rs"]
mod indicators_structure_support_resistance;

#[path = "indicators/parser.rs"]
mod indicators_parser;

#[path = "indicators/registry.rs"]
mod indicators_registry;

#[path = "signals/aggregation.rs"]
mod signals_aggregation;

#[path = "signals/categories.rs"]
mod signals_categories;

#[path = "signals/scoring.rs"]
mod signals_scoring;

#[path = "signals/decision.rs"]
mod signals_decision;

#[path = "signals/engine.rs"]
mod signals_engine;

#[path = "services/market_data.rs"]
mod services_market_data;

#[path = "core/http.rs"]
mod core_http;

#[path = "core/runtime.rs"]
mod core_runtime;

