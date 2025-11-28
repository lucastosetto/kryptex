//! Unit tests - organized by module structure

#[path = "unit/common/math.rs"]
mod common_math;

#[path = "unit/indicators/parser.rs"]
mod indicators_parser;

#[path = "unit/indicators/registry.rs"]
mod indicators_registry;

#[path = "unit/indicators/validation.rs"]
mod indicators_validation;

#[path = "unit/indicators/momentum/macd.rs"]
mod indicators_momentum_macd;

#[path = "unit/indicators/momentum/rsi.rs"]
mod indicators_momentum_rsi;

#[path = "unit/indicators/trend/ema.rs"]
mod indicators_trend_ema;

#[path = "unit/indicators/trend/supertrend.rs"]
mod indicators_trend_supertrend;

#[path = "unit/indicators/volatility/bollinger.rs"]
mod indicators_volatility_bollinger;

#[path = "unit/indicators/volatility/atr.rs"]
mod indicators_volatility_atr;

#[path = "unit/indicators/volume/obv.rs"]
mod indicators_volume_obv;

#[path = "unit/indicators/volume/volume_profile.rs"]
mod indicators_volume_volume_profile;

#[path = "unit/indicators/perp/open_interest.rs"]
mod indicators_perp_open_interest;

#[path = "unit/indicators/perp/funding_rate.rs"]
mod indicators_perp_funding_rate;

#[path = "unit/signals/decision.rs"]
mod signals_decision;

#[path = "unit/signals/engine.rs"]
mod signals_engine;

#[path = "unit/signals/scenarios.rs"]
mod signals_scenarios;

#[path = "unit/engine/aggregator.rs"]
mod engine_aggregator;

#[path = "unit/services/market_data.rs"]
mod services_market_data;

#[path = "unit/core/http.rs"]
mod core_http;

#[path = "unit/core/runtime.rs"]
mod core_runtime;
