# Perptrix

A modular crypto perpetuals signal generation and execution engine built in Rust.

## 📖 Overview

Perptrix is designed to:
1. Receive market data from exchanges (initially Hyperliquid)
2. Calculate technical indicators
3. Generate trading signals with recommended stop loss (SL) and take profit (TP) percentages
4. Execute Long/Short orders in perpetual futures
5. Maintain modularity to allow changing exchanges without altering core logic

## 📊 Current Status

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
- QuestDB persistence layer for candles and signals (`src/db/questdb.rs`)
- Redis caching layer for fast signal evaluation (`src/cache/redis.rs`)
- Unit + integration tests covering indicators and multiple market regimes (`tests/**`)

**Market Data Integration:**
- Hyperliquid WebSocket client for real-time candle updates (`src/services/hyperliquid/client.rs`)
- Hyperliquid REST API client for historical candle fetching (`src/services/hyperliquid/rest.rs`)
- Historical data fetching on startup (configurable count, default: 200 candles)
- Automatic storage in QuestDB and caching in Redis
- Multi-interval support (1m, 5m, 15m, 1h)

**Cloud Runtime:**
- HTTP server with health check endpoint (`/health`)
- Periodic signal evaluation runtime with real market data
- Hyperliquid market data provider with WebSocket and REST integration
- Environment-based configuration (sandbox/production)

### Missing / In Progress

**Phase 3 Requirements:**
- Structured logging/metrics suitable for cloud monitoring (only `println!` statements)
- Funding rate and open interest real-time updates (historical data fetching implemented)

**Future Phases:**
- Execution engine (order placement, trade management)
- Dashboard & backtester

## 🏗️ Architecture

```
┌─────────────────────┐
│ Hyperliquid REST    │───┐
│ (Historical Data)   │   │
└─────────────────────┘   │
                          │
┌─────────────────────┐   │
│ Hyperliquid WS      │───┤
│ (Real-time Updates) │   │
└───────────┬─────────┘   │
            │             │
            ▼             │
    ┌───────────────┐     │
    │ Market Data   │     │
    │   Provider    │     │
    └───────┬───────┘     │
            │             │
            ├─────────────┘
            │ Candles
            ▼
    ┌───────────────┐
    │   QuestDB     │ (Persistent Storage)
    └───────────────┘
            │
            ├──────────────┐
            │              │
            ▼              ▼
    ┌───────────────┐  ┌───────────────┐
    │     Redis     │  │ In-Memory     │
    │    (Cache)    │  │   Buffer      │
    └───────┬───────┘  └───────┬───────┘
            │                  │
            └──────────┬───────┘
                       │
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
                       │
                       ▼
            ┌─────────────────────────┐
            │   QuestDB (Signals)     │
            └─────────────────────────┘
                       │
                       ▼
            (Future) Trade Executor
```

## 📁 Project Structure

```
perptrix/
  config.example.json   # Example configuration file with category weights
  src/
    common/             # Shared helpers (math utilities: EMA, SMA, std dev)
    config/             # Configuration management (JSON-based config)
    core/               # Cloud runtime (HTTP server, periodic task runner)
    ├── http.rs         # HTTP endpoints (health check)
    └── runtime.rs      # Periodic signal evaluation
  db/                   # Persistence adapters (QuestDB)
  cache/                # Caching layer (Redis)
  evaluation/           # Signal scoring and validation utilities
  engine/               # Signal aggregation and scoring
    ├── aggregator.rs   # Category-based signal aggregation (integer scoring)
    └── signal.rs       # Trading signal types and market bias
  indicators/           # Indicator implementations organized by category
    ├── momentum/       # MACD, RSI
    ├── trend/          # EMA, SuperTrend
    ├── volatility/     # Bollinger Bands, ATR
    ├── volume/         # OBV, Volume Profile (beyond RFC Phase 2)
    ├── perp/           # Funding Rate, Open Interest (beyond RFC Phase 2)
    └── registry.rs     # Indicator registry and category system
  models/               # Shared DTOs (Candle, IndicatorSet, SignalOutput)
  services/             # Market data provider interface
    hyperliquid/        # Hyperliquid WebSocket and REST clients
      client.rs         # WebSocket client with reconnection logic
      messages.rs       # WebSocket message types
      provider.rs       # Market data provider implementation
      rest.rs           # REST API client for historical data
      subscriptions.rs  # Subscription management
  signals/              # Signal evaluation engine
    ├── decision.rs     # Direction thresholds and SL/TP logic
    └── engine.rs       # Main signal evaluation orchestrator
  strategies/           # Strategy definitions (placeholder)
  lib.rs                # Crate root exposing layered modules
```

## 🔧 Installation

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo
- Docker and Docker Compose (for local development with QuestDB and Redis)

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Local Development Setup

Perptrix uses QuestDB for persistent storage and Redis for caching. Start the required services using Docker Compose:

```bash
docker-compose up -d
```

This will start:
- **QuestDB** on ports 9000 (HTTP) and 8812 (PostgreSQL wire protocol)
- **Redis** on port 6379

To stop the services:

```bash
docker-compose down
```

To view QuestDB's web console, visit: http://localhost:9000

## 🚀 Usage

### Running the Server

Start the server using:

```bash
cargo run --bin server
```

**Environment Variables:**
- `PORT` - HTTP server port (default: 8080)
- `EVAL_INTERVAL_SECONDS` - Signal evaluation interval in seconds (default: 0 = disabled)
- `SYMBOLS` - Comma-separated list of symbols to evaluate (required when `EVAL_INTERVAL_SECONDS > 0`)
- `PERPTRIX_ENV` - Environment: `sandbox` or `production` (default: `production`)
  - `sandbox` - Uses Hyperliquid testnet (wss://api.hyperliquid-testnet.xyz/ws)
  - `production` - Uses Hyperliquid mainnet (wss://api.hyperliquid.xyz/ws)
- `QUESTDB_URL` - QuestDB connection string (default: `host=localhost user=admin password=quest port=8812`)
- `REDIS_URL` - Redis connection string (default: `redis://127.0.0.1/`)
- `HISTORICAL_CANDLE_COUNT` - Number of historical candles to fetch on startup (default: 200)

**Configuration File:**
- Create a `config.json` file in the working directory to customize category weights and other settings (see `config.example.json` for a template)
- The configuration file is automatically loaded when the server starts

**Examples:**

```bash
# Custom port
PORT=3000 cargo run --bin server

# Enable periodic signal evaluation (production)
EVAL_INTERVAL_SECONDS=60 SYMBOLS=BTC cargo run --bin server

# Sandbox environment with custom historical candle count
PERPTRIX_ENV=sandbox EVAL_INTERVAL_SECONDS=60 SYMBOLS=BTC HISTORICAL_CANDLE_COUNT=500 cargo run --bin server

# Full configuration with multiple symbols
PORT=8080 EVAL_INTERVAL_SECONDS=30 SYMBOLS=BTC,ETH PERPTRIX_ENV=production cargo run --bin server

# Custom database connections
QUESTDB_URL="host=localhost user=admin password=quest port=8812" \
REDIS_URL="redis://127.0.0.1:6379" \
cargo run --bin server
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

**Note:** The server automatically:
1. Connects to QuestDB and Redis (with automatic reconnection if unavailable)
2. Fetches historical candles from Hyperliquid REST API on startup
3. Stores historical candles in QuestDB and caches them in Redis
4. Subscribes to real-time candle updates via WebSocket
5. Evaluates signals using cached/real-time data

If QuestDB or Redis are unavailable, the system will gracefully degrade and continue operating with in-memory buffers.

## ⚡ Signal Engine

### Indicator System

The signal engine uses 10 indicators organized into 5 categories. Each indicator produces signals that are scored and aggregated to generate the final trading signal.

#### Momentum Indicators (25% weight)

**RSI (Relative Strength Index) - 14 period**
- Measures overbought/oversold conditions
- Detects bullish/bearish divergences
- Signals: Oversold (+1), Overbought (-1), Divergences (±2)

**MACD (Moving Average Convergence Divergence) - 12/26/9**
- Tracks momentum changes via EMA crossovers
- Identifies trend reversals and momentum shifts
- Signals: Bullish/Bearish Cross (±2), Momentum (±1)

#### Trend Indicators (30% weight)

**EMA Crossover - 20/50 periods**
- Identifies trend direction and strength
- Detects golden cross (bullish) and death cross (bearish)
- Signals: Bullish/Bearish Cross (±2), Strong Trend (±1)

**SuperTrend - 10 period, 3.0 multiplier**
- Dynamic trailing stop indicator
- Identifies trend flips and continuation
- Signals: Bullish/Bearish Flip (±2), Trend Continuation (±1)

#### Volatility Indicators (15% weight)

**Bollinger Bands - 20 SMA, 2σ**
- Measures volatility and price extremes
- Detects breakouts, squeezes, and mean reversion
- Signals: Upper/Lower Breakout (±1), Squeeze/Mean Reversion (informational)

**ATR (Average True Range) - 14 period**
- Measures market volatility
- Classifies volatility regime (Low/Normal/Elevated/High)
- Used for SL/TP calculation and risk assessment

#### Volume Indicators (15% weight)

**OBV (On-Balance Volume)**
- Confirms price movements with volume
- Detects volume divergences
- Signals: Bullish/Bearish Divergence (±2), Confirmation (+1)

**Volume Profile**
- Identifies high/low volume nodes (POC)
- Detects support/resistance levels based on volume
- Signals: POC Support (+1), POC Resistance (-1), Near LVN (informational)

#### Perp Indicators (15% weight)

**Open Interest**
- Tracks new money entering/leaving the market
- Identifies squeeze conditions
- Signals: Bullish/Bearish Expansion (±2), Squeeze Conditions (±1)

**Funding Rate - 24-hour rolling average**
- Measures perpetual swap funding bias
- Detects extreme positioning
- Signals: Extreme Bias (inverse: -1 for long bias, +1 for short bias)

### Signal Aggregation

Indicators are combined using a category-based scoring system:

1. **Category Scoring**: Each category receives an integer score from -3 to +3 (or -2 to +2 for volatility/volume/perp):
   - Positive scores indicate bullish signals
   - Negative scores indicate bearish signals
   - Zero indicates neutral

2. **Total Score**: All category scores are summed to produce a total score

3. **Market Bias**: The total score determines market bias:
   - ≥ 7: Strong Bullish
   - ≥ 3: Bullish
   - -3 to 3: Neutral
   - ≤ -3: Bearish
   - ≤ -7: Strong Bearish

4. **Position**: Market bias maps to trading position:
   - Strong Bullish / Bullish → Long
   - Neutral → Neutral
   - Bearish / Strong Bearish → Short

5. **Confidence**: Calculated based on:
   - Alignment of category signals (more alignment = higher confidence)
   - Trend and momentum alignment bonus (+20% if aligned)
   - Misalignment penalty (-20% if not aligned)

6. **Risk Assessment**: Considers:
   - Volatility regime (high volatility increases risk)
   - Extreme funding rates (increases risk)
   - Weak total score (increases risk)
   - RSI divergences (decreases risk)

## 🧪 Testing

Run all tests:

```bash
cargo test
```

## ⚙️ Signal Engine Configuration

### Category Weights

Category weights are configurable via a JSON configuration file. The default weights are:
- **Momentum**: 25% (MACD, RSI)
- **Trend**: 30% (EMA, SuperTrend)
- **Volatility**: 15% (Bollinger Bands, ATR)
- **Volume**: 15% (OBV, Volume Profile)
- **Perp**: 15% (Funding Rate, Open Interest)

**Note:** The aggregator currently uses integer scoring (-3 to +3 per category) rather than applying these percentage weights directly. The weights are stored in the configuration for future use and documentation purposes.

#### Configuring Weights

Create a `config.json` file (or use `config.example.json` as a template) with your desired category weights:

```json
{
  "category_weights": {
    "momentum": 0.25,
    "trend": 0.30,
    "volatility": 0.15,
    "volume": 0.15,
    "perp": 0.15
  }
}
```

Weights should sum to 1.0 (100%).

### Direction Thresholds

The signal engine uses integer scores to determine market bias, which maps to trading positions:
- **Long**: Total score ≥ 3 (Bullish or Strong Bullish bias)
- **Short**: Total score ≤ -3 (Bearish or Strong Bearish bias)
- **Neutral**: Total score between -3 and 3

**Note:** The debug output shows both the integer score (used for decision making) and a normalized score (0-1 range) for reference. The integer score thresholds are what actually determine the signal direction.

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

## 🗺️ Implementation Roadmap

### ✅ Phase 1 — POC (Completed)
- Receive external indicators
- Generate LONG/SHORT signal + SL/TP + reasons
- QuestDB persistence (migrated from SQLite)

### ✅ Phase 2 — Signal Engine (Completed)
- **Momentum Indicators**: MACD (12/26/9), RSI (14)
- **Trend Indicators**: EMA (20/50 cross), SuperTrend (10, 3)
- **Volatility Indicators**: Bollinger Bands (20 SMA, 2σ), ATR (14)
- **Volume Indicators**: OBV, Volume Profile
- **Perp Indicators**: Funding Rate, Open Interest
- Category-based aggregation with integer scoring
- Signal decision engine (Long/Short/Neutral thresholds)
- SL/TP calculation from ATR
- Cloud runtime with HTTP health check

### ✅ Phase 3 — Exchange Adapter (Completed)
- WebSocket market data integration (Hyperliquid)
- Historical candle fetching via REST API
- OHLC reconstruction from real-time data
- Environment-based configuration (sandbox/production)
- Real-time data pipeline with automatic reconnection
- QuestDB for persistent storage
- Redis for fast caching
- Docker Compose setup for local development

### 🔜 Phase 3 — Remaining
- Structured logging and metrics
- Real-time funding rate and open interest updates

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


## 📄 License

This project is released under the MIT License. See [LICENSE.md](LICENSE.md)
for the full text and terms.

## 🤝 Contributing

Contributions are welcome! Please read
[CONTRIBUTING.md](CONTRIBUTING.md) for the workflow and quality checklist
before opening a pull request.

