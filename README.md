# Kryptex

A modular crypto perpetuals signal generation and execution engine built in Rust.

## Overview

Kryptex is designed to:
1. Receive market data from exchanges (initially Hyperliquid)
2. Calculate technical indicators
3. Generate trading signals with recommended stop loss (SL) and take profit (TP) percentages
4. Execute Long/Short orders in perpetual futures
5. Maintain modularity to allow changing exchanges without altering core logic

## Current Status: Phase 2 & 3 Complete ✅

The signal engine has been fully implemented according to the [Kryptex RFC](https://github.com/lucastosetto/kryptex/wiki/1.-RFC-%E2%80%90-Kryptex:-Crypto-Perps-Signal-&-Execution-Engine):

- ✅ Complete indicator calculation engine (all categories implemented)
- ✅ Category-based aggregation system with RFC-defined weights
- ✅ Signal decision engine with Long/Short/Neutral thresholds
- ✅ Cloud runtime with HTTP health check endpoint
- ✅ Periodic task runner for continuous signal evaluation
- ✅ Comprehensive test suite (133 tests, all passing)
- 🔜 Exchange adapters and execution engine (Phase 3+)

## Architecture

```
┌─────────────────┐
│ Hyperliquid WS  │─────┐
└───────────┬─────┘     │ Future adapters
            │            │
            ▼            │
    ┌───────────────┐
    │ Market Data    │
    │   Pipeline     │
    └───────┬────────┘
            │ Candles / Indicators (POC)
            ▼
   ┌─────────────────┐
   │ Indicator Engine │
   └────────┬────────┘
            │ Signals
            ▼
  ┌────────────────────────┐
  │ Signal Interpreter      │
  │ + SL/TP Recommendations │
  └──────────┬─────────────┘
             ▼
      (Future) Trade Executor
             ▼
          Unified DB
```

## Project Structure

```
src/
  common/               # Shared helpers (math utilities: EMA, SMA, std dev)
  config/               # Configuration management
  core/                 # Cloud runtime (HTTP server, periodic task runner)
    ├── http.rs         # HTTP endpoints (health check)
    └── runtime.rs      # Periodic signal evaluation
  db/                   # Persistence adapters (SQLite)
  evaluation/           # Signal scoring and validation utilities
  indicators/           # Indicator implementations organized by category
    ├── momentum/       # MACD, RSI
    ├── trend/          # EMA, ADX
    ├── volatility/     # Bollinger Bands, ATR
    ├── structure/      # SuperTrend, Support/Resistance
    └── registry.rs     # Indicator registry and category system
  models/               # Shared DTOs (Candle, IndicatorSet, SignalOutput)
  services/             # Market data provider interface
  signals/              # Signal evaluation engine
    ├── aggregation.rs  # Category-based aggregation
    ├── categories.rs   # Category weights
    ├── scoring.rs      # Score normalization
    ├── decision.rs     # Direction thresholds and SL/TP logic
    └── engine.rs       # Main signal evaluation orchestrator
  strategies/           # Strategy definitions (placeholder)
  lib.rs                # Crate root exposing layered modules
```

## Installation

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Running the Server

Start the server with default settings:

```bash
cargo run --bin server
```

The server will:
- Start HTTP server on port 8080 (configurable via `PORT` env var)
- Optionally run periodic signal evaluation (disabled by default)

**Environment Variables:**
- `PORT` - HTTP server port (default: 8080)
- `EVAL_INTERVAL_SECONDS` - Signal evaluation interval in seconds (default: 0 = disabled)
- `SYMBOLS` - Comma-separated list of symbols to evaluate (default: "BTC")

**Examples:**

```bash
# Just HTTP server on default port
cargo run --bin server

# Custom port
PORT=3000 cargo run --bin server

# HTTP server + periodic evaluation every 60 seconds
EVAL_INTERVAL_SECONDS=60 cargo run --bin server

# Full configuration
PORT=8080 EVAL_INTERVAL_SECONDS=30 SYMBOLS=BTC,ETH cargo run --bin server
```

### Health Check

The HTTP server exposes a health check endpoint:

```bash
curl http://localhost:8080/health
```

Response:
```json
{
  "status": "healthy",
  "uptime_seconds": 0,
  "service": "kryptex-signal-engine"
}
```

**Note:** When periodic evaluation is enabled, it will use the placeholder data provider (returns empty data) until a real market data provider is implemented. Signals will only be generated when actual candle data is available.

## Usage

### Signal Evaluation

Evaluate signals from candle data:

```rust
use kryptex::signals::engine::SignalEngine;
use kryptex::models::indicators::Candle;
use chrono::Utc;

// Create candle data
let candles = vec![
    Candle::new(100.0, 101.0, 99.0, 100.5, 1000.0, Utc::now()),
    // ... more candles
];

// Evaluate signal
if let Some(signal) = SignalEngine::evaluate(&candles, "BTC") {
    println!("Direction: {:?}", signal.direction);
    println!("Confidence: {:.2}%", signal.confidence * 100.0);
    println!("SL: {:.2}%", signal.recommended_sl_pct);
    println!("TP: {:.2}%", signal.recommended_tp_pct);
}
```

### Individual Indicators

Calculate specific indicators:

```rust
use kryptex::indicators::momentum::{calculate_rsi_default, calculate_macd_default};
use kryptex::indicators::trend::calculate_ema;
use kryptex::indicators::volatility::calculate_atr_default;

// RSI
let rsi = calculate_rsi_default(&candles);

// MACD
let macd = calculate_macd_default(&candles);

// EMA
let ema_12 = calculate_ema(&candles, 12);

// ATR
let atr = calculate_atr_default(&candles);
```

### Cloud Runtime

Start the HTTP server and periodic task runner:

```rust
use kryptex::core::{start_server, SignalRuntime, RuntimeConfig};

// Start HTTP server (health check at /health)
tokio::spawn(async {
    start_server(8080).await.unwrap();
});

// Start periodic signal evaluation
let config = RuntimeConfig {
    evaluation_interval_seconds: 60,
    symbols: vec!["BTC".to_string(), "ETH".to_string()],
};
let runtime = SignalRuntime::new(config);
runtime.run().await?;
```

## Testing

Run all tests:

```bash
cargo test
```

The test suite includes:
- **Unit tests**: Each indicator and module has comprehensive unit tests (66 tests)
- **Integration tests**: Market scenario tests (uptrends, downtrends, ranging, volatility, reversals)
- **Total**: 133 tests, all passing ✅

Test coverage includes:
- Indicator calculations with fixed datasets
- Score normalization and aggregation logic
- Signal decision thresholds
- Category weight verification
- Edge cases (insufficient data, NaN handling)

### Persistence

Signals are automatically stored in `kryptex_signals.db`:

```rust
use kryptex::db::SignalDatabase;

let db = SignalDatabase::new("kryptex_signals.db")?;
db.store_signal(&signal)?;

let all_signals = db.get_all_signals()?;
let btc_signals = db.get_signals_by_symbol("BTC")?;
```

## Signal Engine Configuration

### Category Weights (RFC-defined)
- **Momentum**: 25% (MACD, RSI)
- **Trend**: 35% (EMA crosses, ADX)
- **Volatility**: 20% (Bollinger Bands, ATR)
- **Market Structure**: 20% (SuperTrend, Support/Resistance)

### Direction Thresholds
- **Long**: Global score > 60%
- **Short**: Global score < 40%
- **Neutral**: Global score 40-60%

### SL/TP Calculation
- **Stop Loss**: ATR × 1.2 (as percentage of price)
- **Take Profit**: ATR × 2.0 (as percentage of price)
- Only calculated for Long/Short signals (not Neutral)

### Indicator Parameters
- **MACD**: 12/26 EMA, 9 signal period
- **RSI**: 14 period
- **EMA**: 12, 26, 50, 200 periods
- **ADX**: 14 period
- **Bollinger Bands**: 20 SMA, 2 standard deviations
- **ATR**: 14 period
- **SuperTrend**: 10 period, 3.0 multiplier

## Implementation Roadmap

### ✅ Phase 1 — POC (Completed)
- Receive external indicators
- Generate LONG/SHORT signal + SL/TP + reasons
- SQLite persistence

### ✅ Phase 2 — Signal Engine (Completed)
- **Momentum Indicators**: MACD (12/26/9), RSI (14)
- **Trend Indicators**: EMA (12, 26, 50, 200), ADX (14)
- **Volatility Indicators**: Bollinger Bands (20 SMA, 2σ), ATR (14)
- **Market Structure**: SuperTrend (10, 3), Support/Resistance
- Category-based aggregation with RFC-defined weights
- Signal decision engine (Long/Short/Neutral thresholds)
- SL/TP calculation from ATR
- Cloud runtime with HTTP health check

### 🔜 Phase 3 — Exchange Adapter
- WebSocket market data integration
- Funding rate fetching
- OHLC reconstruction
- Exchange authentication
- Real-time data pipeline

### 🔜 Phase 4 — Execution Engine
- Order builder
- Trade manager
- Risk manager
- Automatic SL/TP placement
- Trade state machine

### 🔜 Phase 5 — Optional Future Exchanges
- Adapter structure allows easy integration

### 🔜 Phase 6 — Dashboard & Backtester
- Web dashboard (Leptos/Tauri)
- Backtesting engine with historical candles
- Signal performance visualization

## Dependencies

- `serde` / `serde_json` - Serialization
- `rusqlite` - SQLite database
- `chrono` - Timestamps
- `axum` - HTTP framework for cloud runtime
- `tokio` - Async runtime
- `tower` / `tower-http` - Middleware (CORS, logging)

## Design Principles

- **Modularity**: Exchange adapters can be swapped without changing core logic
- **Precision**: Uses `f64` for all numeric values
- **Extensibility**: Clear separation between signal generation and execution
- **Self-documenting**: Minimal comments, code should be clear

## License

This project is released under the MIT License. See [LICENSE.md](LICENSE.md)
for the full text and terms.

## Contributing

Contributions are welcome! Please read
[CONTRIBUTING.md](CONTRIBUTING.md) for the workflow and quality checklist
before opening a pull request.

