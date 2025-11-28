//! Unit tests for the SignalAggregator scoring engine.

use perptrix::engine::aggregator::{IndicatorSignals, SignalAggregator};
use perptrix::engine::signal::{MarketBias, RiskLevel};
use perptrix::indicators::momentum::{macd, rsi};
use perptrix::indicators::perp::{funding_rate, open_interest};
use perptrix::indicators::trend::{ema, supertrend};
use perptrix::indicators::volatility::{atr, bollinger};
use perptrix::indicators::volume::{obv, volume_profile};

fn bullish_signals() -> IndicatorSignals {
    IndicatorSignals {
        ema_signal: ema::EMATrendSignal::BullishCross,
        supertrend_signal: supertrend::SuperTrendSignal::BullishFlip,
        rsi_signal: rsi::RSISignal::BullishDivergence,
        macd_signal: macd::MACDSignal::BullishCross,
        bollinger_signal: bollinger::BollingerSignal::UpperBreakout,
        volatility_regime: atr::VolatilityRegime::Normal,
        obv_signal: obv::OBVSignal::Confirmation,
        volume_profile_signal: volume_profile::VolumeProfileSignal::POCSupport,
        oi_signal: open_interest::OpenInterestSignal::BullishExpansion,
        funding_signal: funding_rate::FundingSignal::NeutralNegative,
    }
}

#[test]
fn aggregator_builds_strong_bullish_bias() {
    let aggregator = SignalAggregator::new();
    let result = aggregator.aggregate(bullish_signals());
    assert!(matches!(
        result.bias,
        MarketBias::StrongBullish | MarketBias::Bullish
    ));
    assert!(result.confidence > 0.7);
    assert!(!result.reasons.is_empty());
}

#[test]
fn aggregator_flags_high_risk_when_volatility_high() {
    let aggregator = SignalAggregator::new();
    let mut signals = bullish_signals();
    signals.volatility_regime = atr::VolatilityRegime::High;
    signals.funding_signal = funding_rate::FundingSignal::ExtremeLongBias;
    signals.rsi_signal = rsi::RSISignal::Neutral;
    let result = aggregator.aggregate(signals);
    assert_eq!(result.risk_level, RiskLevel::High);
}

#[test]
fn funding_extremes_penalize_crowded_longs() {
    let aggregator = SignalAggregator::new();
    let mut signals = bullish_signals();
    let base = aggregator.aggregate(signals.clone());
    signals.funding_signal = funding_rate::FundingSignal::ExtremeLongBias;
    let result = aggregator.aggregate(signals);
    assert!(result.score_breakdown.perp_score < base.score_breakdown.perp_score);
    assert!(matches!(
        result.risk_level,
        RiskLevel::Medium | RiskLevel::High
    ));
}

#[test]
fn funding_extremes_support_contrarian_longs() {
    let aggregator = SignalAggregator::new();
    let mut signals = bullish_signals();
    signals.funding_signal = funding_rate::FundingSignal::ExtremShortBias;
    let result = aggregator.aggregate(signals);
    assert!(result.score_breakdown.perp_score >= 2);
    assert!(result.confidence > 0.5);
}
