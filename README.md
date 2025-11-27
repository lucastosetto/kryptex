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

Perptrix implements a signal engine based on the [RFC](https://github.com/lucastosetto/perptrix/wiki/1.-RFC-%E2%80%90-Perptrix:-Crypto-Perps-Signal-&-Execution-Engine), with a complete indicator set that includes RFC Phase 2 indicators plus additional categories. The core signal evaluation pipeline (indicator computation, aggregation, decisioning, SL/TP logic) is functional, while runtime integration (live data, HTTP signal APIs, metrics, exchange execution) is still pending.

### Implemented

**Indicator Categories:**
- **Momentum**: MACD (12/26/9), RSI (14)
- **Trend**: EMA (20/50 cross), SuperTrend (10, 3.0)
- **Volatility**: Bollinger Bands (20 SMA, 2σ), ATR (14)
- **Volume**: OBV, Volume Profile (POC-based support/resistance)
- **Perp**: Funding Rate, Open Interest

**Core Engine:**
- Signal aggregation with category-based scoring (`src/engine/aggregator.rs`)
- Direction thresholds and ATR-driven SL/TP logic (`src/signals/decision.rs`)
- Signal evaluation orchestrator (`src/signals/engine.rs`)
- SQLite persistence layer (`src/db/sqlite.rs`)
- Unit + integration tests covering indicators and multiple market regimes (`tests/**`)

**Cloud Runtime (Partial):**
- HTTP server with health check endpoint (`/health`)
- Periodic signal evaluation runtime (requires real market data provider)
- Placeholder market data provider interface

### Missing / In Progress

**Phase 3 Requirements:**
- Live market data ingestion: `SignalRuntime` currently uses `PlaceholderMarketDataProvider`
- HTTP API for retrieving latest signal/indicator breakdown (server only has `/health`)
- Structured logging/metrics suitable for cloud monitoring (only `println!` statements)
- Exchange adapters (Hyperliquid WebSocket, funding rate fetching)
- OHLC reconstruction from real-time data

**Future Phases:**
- Execution engine (order placement, trade management)
- Dashboard & backtester

## RFC Alignment

| RFC Item | Status | Notes |
| --- | --- | --- |
| **Indicators** | | |
| Momentum: MACD, RSI | ✅ | Fully implemented (12/26/9, 14) |
| Trend: EMA cross, SuperTrend | ✅ | EMA 20/50 cross, SuperTrend (10, 3.0) |
| Volatility: Bollinger Bands, ATR | ✅ | Fully implemented (20 SMA, 2σ; 14 period) |
| Volume: OBV, Volume Profile | ✅ | Implemented (beyond RFC Phase 2) |
| Perp: Funding Rate, Open Interest | ✅ | Implemented (beyond RFC Phase 2) |
| **Signal Engine** | | |
| Category-based aggregation | ✅ | Integer scoring system (-3 to +3 per category) |
| Direction thresholds (>60% Long, <40% Short) | ✅ | Implemented in `signals::decision` |
| SL/TP logic (ATR × 1.2/2.0) | ✅ | Correctly implemented |
| Explainability (per-indicator contributions) | ✅ | `Aggregator` returns reasons for each signal |
| **Infrastructure** | | |
| Persistence (SQLite) | ✅ | Schema and helpers ready but not wired into runtime |
| Cloud runtime | ⚠️ Partial | `SignalRuntime` + Axum server exist; server only has `/health` |
| **Phase 3 - Runtime** | | |
| HTTP signal endpoint | ❌ | Needs endpoint(s) to fetch latest signal, indicator set, history |
| Market data ingestion | ❌ | Only `PlaceholderMarketDataProvider`; no exchange adapters |
| Logging + metrics | ❌ | No structured logging or metrics |
| **Future Phases** | | |
| Execution engine | ❌ | Not started |
| Dashboard & backtester | ❌ | Not started |

## Architecture

```
┌─────────────────┐
│ Hyperliquid WS  │─────┐
└───────────┬─────┘     │ Future adapters
            │           │
            ▼           │
    ┌───────────────┐
    │ Market Data   │
    │   Pipeline    │
    └───────┬───────┘
            │ Candles / Indicators (POC)
            ▼
   ┌──────────────────┐
   │ Indicator Engine │
   └────────┬─────────┘
            │ Signals
            ▼
  ┌─────────────────────────┐
  │ Signal Interpreter      │
  │ + SL/TP Recommendations │
  └──────────┬──────────────┘
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
  engine/               # Signal aggregation and scoring
    ├── aggregator.rs   # Category-based signal aggregation (integer scoring)
    └── signal.rs       # Trading signal types and market bias
  indicators/           # Indicator implementations organized by category
    ├── momentum/       # MACD, RSI
    ├── trend/          # EMA, SuperTrend (ADX missing)
    ├── volatility/     # Bollinger Bands, ATR
    ├── volume/         # OBV, Volume Profile (beyond RFC Phase 2)
    ├── perp/           # Funding Rate, Open Interest (beyond RFC Phase 2)
    └── registry.rs     # Indicator registry and category system
  models/               # Shared DTOs (Candle, IndicatorSet, SignalOutput)
  services/             # Market data provider interface
  signals/              # Signal evaluation engine
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

The signal engine uses stateful indicators that update incrementally. For standalone calculations, use the indicator structs directly:

```rust
use perptrix::indicators::momentum::{rsi, macd};
use perptrix::indicators::trend::ema;
use perptrix::indicators::volatility::atr;

// RSI
let mut rsi = rsi::RSI::new(14);
for candle in &candles {
    if let Some(rsi_value) = rsi.update(candle.close) {
        // Use rsi_value
    }
}

// MACD
let mut macd = macd::MACD::new(12, 26, 9);
for candle in &candles {
    let (macd_line, signal_line, histogram, signal) = macd.update(candle.close);
    // Use values
}

// EMA
let mut ema_cross = ema::EMACrossover::new(20, 50);
for candle in &candles {
    let signal = ema_cross.update(candle.close);
    // Use signal
}

// ATR
let mut atr = atr::ATR::new(14);
for candle in &candles {
    let atr_value = atr.update(candle.high, candle.low, candle.close);
    // Use atr_value
}
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
- **Indicators**: Unit tests for MACD, RSI, EMA, Bollinger Bands, ATR, SuperTrend, OBV, Volume Profile, Funding Rate, Open Interest (see `tests/indicators/**`)
- **Signal scenarios**: Integration tests exercising strong up/down trends, ranging markets, high volatility, and major reversals using deterministic synthetic candles (`tests/signal_scenarios.rs`)
- **Signal pipeline**: Aggregation, decision thresholds, and SL/TP calculations (`tests/signals/**` and `tests/engine/aggregator.rs`)
- **Core components**: HTTP server, runtime, market data provider interface (`tests/core/**` and `tests/services/**`)

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

### Category Weights

The aggregator uses integer scoring (-3 to +3 per category). The registry defines percentage weights:
- **Momentum**: 25% (MACD, RSI)
- **Trend**: 30% (EMA, SuperTrend)
- **Volatility**: 15% (Bollinger Bands, ATR)
- **Volume**: 15% (OBV, Volume Profile)
- **Perp**: 15% (Funding Rate, Open Interest)

### Direction Thresholds
- **Long**: Global score > 60% (implemented via `DirectionThresholds::LONG_THRESHOLD`)
- **Short**: Global score < 40% (implemented via `DirectionThresholds::SHORT_THRESHOLD`)
- **Neutral**: Global score 40-60%

### SL/TP Calculation
- **Stop Loss**: ATR × 1.2 (as percentage of price)
- **Take Profit**: ATR × 2.0 (as percentage of price)
- Only calculated for Long/Short signals (not Neutral)

### Indicator Parameters

- **MACD**: 12/26 EMA, 9 signal period
- **RSI**: 14 period
- **EMA**: 20/50 cross
- **SuperTrend**: 10 period, 3.0 multiplier
- **Bollinger Bands**: 20 SMA, 2 standard deviations
- **ATR**: 14 period
- **OBV**: On-Balance Volume
- **Volume Profile**: POC-based support/resistance detection
- **Funding Rate**: 24-hour rolling average
- **Open Interest**: Change-based signals

## Implementation Roadmap

### ✅ Phase 1 — POC (Completed)
- Receive external indicators
- Generate LONG/SHORT signal + SL/TP + reasons
- SQLite persistence

### ✅ Phase 2 — Signal Engine (Completed)
- **Momentum Indicators**: MACD (12/26/9), RSI (14)
- **Trend Indicators**: EMA (20/50 cross), SuperTrend (10, 3)
- **Volatility Indicators**: Bollinger Bands (20 SMA, 2σ), ATR (14)
- **Volume Indicators**: OBV, Volume Profile
- **Perp Indicators**: Funding Rate, Open Interest
- Category-based aggregation with integer scoring
- Signal decision engine (Long/Short/Neutral thresholds)
- SL/TP calculation from ATR
- Cloud runtime with HTTP health check (partial)

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

