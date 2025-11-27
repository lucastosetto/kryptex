# Perptrix

A modular crypto perpetuals signal generation and execution engine built in Rust.

## Overview

Perptrix is designed to:
1. Receive market data from exchanges (initially Hyperliquid)
2. Calculate technical indicators
3. Generate trading signals with recommended stop loss (SL) and take profit (TP) percentages
4. Execute Long/Short orders in perpetual futures
5. Maintain modularity to allow changing exchanges without altering core logic

## Current Status

Perptrix now ships the full Phase 2 signal engine defined in the [RFC](https://github.com/lucastosetto/perptrix/wiki/1.-RFC-%E2%80%90-Perptrix:-Crypto-Perps-Signal-&-Execution-Engine), plus scaffolding for the Phase 3 cloud runtime. Indicator computation, aggregation, decisioning, and SL/TP logic are implemented, while runtime integration (live data, HTTP signal APIs, metrics, exchange execution) is still pending.

### Implemented

- Multi-category indicator stack: MACD, RSI, 12/26 + 50/200 EMAs, ADX, Bollinger Bands, ATR, SuperTrend, Support/Resistance (`src/indicators/**`).
- Normalization, category-based weighting, aggregation, and explainability (`src/signals/scoring.rs`, `src/signals/aggregation.rs`, `src/signals/categories.rs`).
- Direction thresholds and ATR-driven SL/TP logic (`src/signals/decision.rs`).
- SQLite persistence layer ready for storing evaluated signals (`src/db/sqlite.rs`).
- Unit + integration tests covering indicators and multiple market regimes (`tests/**`).

### Missing / In Progress

- Live market data ingestion: `SignalRuntime` currently uses `PlaceholderMarketDataProvider`, so periodic jobs cannot emit real signals.
- HTTP API for retrieving the latest signal/indicator breakdown (server currently exposes only `/health`).
- Structured logging/metrics suitable for cloud monitoring (only `println!` statements exist today).
- Exchange adapters, funding-rate ingestion, execution engine, dashboard/backtester (future RFC phases).

## RFC Alignment

| RFC Item | Status | Notes |
| --- | --- | --- |
| Indicator categories (Momentum, Trend, Volatility, Market Structure) | ✅ | Implemented in `src/indicators/` with dedicated modules per category. |
| Normalization + helper utilities | ✅ | `src/signals/scoring.rs` provides shared normalization + confidence helpers. |
| Category weighting + aggregation | ✅ | `Aggregator` + `CategoryWeights` mirror the RFC weights. |
| Direction thresholds + SL/TP logic | ✅ | `signals::decision` matches the >60% / <40% thresholds and ATR × (1.2/2.0) rules. |
| Explainability (per-indicator contributions) | ✅ | `Aggregator::generate_reasons` returns category + indicator reasons. |
| Persistence | ✅ | `SignalDatabase` (SQLite) schema + helpers are ready but not wired into runtime yet. |
| Cloud runtime | ⚠️ Partial | `SignalRuntime` + Axum server exist, but server only has `/health` and runtime has no real data source. |
| HTTP signal endpoint | ❌ | Needs endpoint(s) to fetch latest signal, indicator set, and stored history. |
| Market data ingestion (Hyperliquid adapter, funding data) | ❌ | Only `PlaceholderMarketDataProvider` is present; no exchange adapters yet. |
| Logging + metrics | ❌ | No structured logging, telemetry, or Prometheus-style metrics are implemented. |
| Execution engine + downstream phases | ❌ | Order/execution/risk management modules have not been started. |

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
  "service": "perptrix-signal-engine"
}
```

**Note:** When periodic evaluation is enabled, it will use the placeholder data provider (returns empty data) until a real market data provider is implemented. Signals will only be generated when actual candle data is available.

## Usage

### Signal Evaluation

Evaluate signals from candle data:

```rust
use perptrix::signals::engine::SignalEngine;
use perptrix::models::indicators::Candle;
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
use perptrix::indicators::momentum::{calculate_rsi_default, calculate_macd_default};
use perptrix::indicators::trend::calculate_ema;
use perptrix::indicators::volatility::calculate_atr_default;

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
use perptrix::core::{start_server, SignalRuntime, RuntimeConfig};

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

What the suite currently covers:
- **Indicators & helpers**: Unit tests for MACD, RSI, EMA, ADX, Bollinger Bands, ATR, SuperTrend, Support/Resistance, math helpers, parsers, and validation logic (see `tests/indicators/**` and `tests/unit/**`).
- **Signal scenarios**: Integration tests exercising strong up/down trends, ranging markets, high volatility, and major reversals using deterministic synthetic candles (`tests/signal_scenarios.rs`).
- **Signal pipeline**: Aggregation, category weights, decision thresholds, and SL/TP calculations (`tests/signals/**`).

Add exchange-provided fixture datasets + performance benchmarks before promoting to 24/7 cloud execution.

### Persistence

Signals are automatically stored in `perptrix_signals.db`:

```rust
use perptrix::db::SignalDatabase;

let db = SignalDatabase::new("perptrix_signals.db")?;
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

