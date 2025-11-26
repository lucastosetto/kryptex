# Kryptex

A modular crypto perpetuals signal generation and execution engine built in Rust.

## Overview

Kryptex is designed to:
1. Receive market data from exchanges (initially Hyperliquid)
2. Calculate technical indicators
3. Generate trading signals with recommended stop loss (SL) and take profit (TP) percentages
4. Execute Long/Short orders in perpetual futures
5. Maintain modularity to allow changing exchanges without altering core logic

## Current Status: Phase 1 POC

The current implementation is a **Proof of Concept** that validates the core signal generation logic:

- ✅ Receives external indicator inputs (MACD, RSI, funding rate)
- ✅ Generates LONG/SHORT signals with confidence scores
- ✅ Provides recommended SL/TP percentages
- ✅ Stores signals in SQLite database
- ❌ Does not connect to any exchange yet
- ❌ Does not execute trades

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
  config/
    mod.rs              # Configuration management
  signals/
    types.rs            # Input/output types (IndicatorInput, SignalOutput)
    signal_generator.rs # Signal generation logic
    mod.rs
  db/
    sqlite.rs           # SQLite persistence layer
    mod.rs
  main.rs               # POC runner
  lib.rs
```

## Installation

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo

### Build

```bash
cargo build
```

### Run

Provide indicator values as keyword arguments:

```bash
cargo run -- --symbol <SYMBOL> --macd <VALUE> --signal <VALUE> --histogram <VALUE> --rsi <VALUE> --funding-rate <VALUE> --price <VALUE>
```

Example:
```bash
cargo run -- --symbol BTC --macd 0.5 --signal 0.3 --histogram 0.2 --rsi 25.0 --funding-rate -0.0002 --price 45000.0
```

Or after building:
```bash
./target/debug/kryptex --symbol BTC --macd 0.5 --signal 0.3 --histogram 0.2 --rsi 25.0 --funding-rate -0.0002 --price 45000.0
```

Show help:
```bash
cargo run -- --help
```

The system will:
1. Accept indicator inputs from command-line arguments
2. Perform calculations and analysis on the indicators
3. Generate trading signals with direction, confidence, SL/TP recommendations
4. Store signals in the SQLite database

## Usage

### Providing Indicators

Provide indicator values as keyword arguments:

```bash
kryptex --symbol <SYMBOL> --macd <VALUE> --signal <VALUE> --histogram <VALUE> --rsi <VALUE> --funding-rate <VALUE> --price <VALUE>
```

Options:
- `--symbol, -s`: Trading symbol (e.g., BTC, ETH) - **Required**
- `--macd, -m`: MACD line value - **Required**
- `--signal`: MACD signal line value - **Required**
- `--histogram`: MACD histogram value (MACD - Signal) - **Required**
- `--rsi, -r`: RSI value (0-100) - **Required**
- `--funding-rate, -f`: Funding rate as decimal (optional, e.g., -0.0002 for -0.02%)
- `--price, -p`: Current price - **Required**
- `--help, -h`: Show help message

Examples:
```bash
# Without funding rate
kryptex --symbol BTC --macd 0.5 --signal 0.3 --histogram 0.2 --rsi 25.0 --price 45000.0

# With funding rate
kryptex --symbol BTC --macd 0.5 --signal 0.3 --histogram 0.2 --rsi 25.0 --funding-rate -0.0002 --price 45000.0
```

The system will analyze these indicators and generate signals.

### Signal Generation

The signal generator takes `IndicatorInput` and produces `SignalOutput`:

```rust
use kryptex::signals::{IndicatorInput, MacdSignal, SignalGenerator};
use kryptex::config::Config;

let config = Config::default();
let generator = SignalGenerator::new(config);

let input = IndicatorInput {
    macd: MacdSignal {
        macd: 0.5,
        signal: 0.3,
        histogram: 0.2,
    },
    rsi: 25.0,
    funding_rate: -0.0002,
    price: 45000.0,
    symbol: Some("BTC".to_string()),
};

let signal = generator.generate_signal(&input);
```

### Signal Output

Each signal includes:
- **Direction**: `Long`, `Short`, or `None`
- **Confidence**: 0.0 to 1.0
- **Recommended SL/TP**: Dynamic percentages based on confidence
- **Reasons**: List of contributing factors with weights

### Persistence

Signals are automatically stored in `kryptex_signals.db`:

```rust
use kryptex::db::SignalDatabase;

let db = SignalDatabase::new("kryptex_signals.db")?;
db.store_signal(&signal)?;

let all_signals = db.get_all_signals()?;
let btc_signals = db.get_signals_by_symbol("BTC")?;
```

## Configuration

Default configuration values:

- Default SL: 2%
- Default TP: 4%
- RSI Overbought: 70
- RSI Oversold: 30
- Min Confidence: 0.5

Customize via `Config::new()` or load from JSON file.

## Implementation Roadmap

### ✅ Phase 1 — POC (Current)
- Receive external indicators
- Generate LONG/SHORT signal + SL/TP + reasons
- SQLite persistence

### 🔜 Phase 2 — Native Indicator Module
- Implement MACD, RSI, ATR, volatility
- Candle aggregation: 1m, 5m, 15m, 1h
- Divergences and candle patterns

### 🔜 Phase 3 — Hyperliquid Adapter
- WebSocket market data
- Funding rate fetching
- OHLC reconstruction
- Ed25519 authentication
- Order submission preparation

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

## Design Principles

- **Modularity**: Exchange adapters can be swapped without changing core logic
- **Precision**: Uses `f64` for all numeric values
- **Extensibility**: Clear separation between signal generation and execution
- **Self-documenting**: Minimal comments, code should be clear

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]

