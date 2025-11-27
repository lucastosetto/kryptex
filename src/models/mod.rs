//! Shared data models spanning the engine layers.

pub mod indicators;
pub mod signal;

pub use signal::{SignalDirection, SignalEvaluation, SignalOutput, SignalReason};
pub use indicators::{
    EmaIndicator, IndicatorSet, MacdIndicator, RsiIndicator, SmaIndicator, VolumeIndicator,
};
