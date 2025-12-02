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

**Cloud Runtime & Observability:**
- Separated services: API server, WebSocket service, and workers
- Production-ready job queue system using Apalis (Redis backend)
- HTTP API server with health, metrics, and tracing middleware
- WebSocket service for real-time market data ingestion
- Background workers for signal evaluation (horizontally scalable)
- Prometheus metrics + OpenTelemetry tracing pipelines wired to Grafana/Tempo
- Environment-based configuration (sandbox/production)

### Missing / In Progress

- Dashboard & backtester
- Execution engine (order placement, trade management)

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
┌─────────────────────┐   │
│ WebSocket Service   │   │
│ (Singleton)         │   │
│ - Maintains WS conn │   │
│ - Subscribes symbols│   │
└───────────┬─────────┘   │
            │             │
            ├─────────────┘
            │ Writes
            ▼
    ┌───────────────┐
    │   QuestDB     │ (Persistent Storage)
    └───────┬───────┘
            │
            ├──────────────┐
            │              │
            ▼              ▼
    ┌───────────────┐  ┌───────────┐
    │     Redis     │  │ In-Memory │
    │ (Cache/Queue) │  │   Buffer  │
    └───────┬───────┘  └───────┬───┘
            │                  │
            │                  │
            ├──────────────────┴──────────┐
            │                             │
            ▼                             ▼
┌─────────────────────┐      ┌─────────────────────┐
│   API Server        │      │   Workers (N)       │
│ (Horizontally       │      │ (Horizontally       │
│  Scalable)          │      │  Scalable)          │
│ - Health/Metrics    │      │ - FetchCandlesJob   │
│ - Business Logic    │      │ - EvaluateSignalJob │
│ - Reads from Redis  │      │ - StoreSignalJob    │
│   /QuestDB          │      │ - Reads from Redis  │
└─────────────────────┘      │   /QuestDB          │
                             └─────────┬───────-───┘
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

### Service Architecture

The system consists of three independent services:

1. **WebSocket Service** (Singleton)
   - Maintains long-lived WebSocket connection to market data provider
   - Receives real-time updates and writes to Redis/QuestDB
   - Should run as a single instance

2. **API Server** (Horizontally Scalable)
   - HTTP API with health check, metrics, and business logic endpoints
   - Stateless - can run multiple instances behind a load balancer
   - Reads from Redis/QuestDB

3. **Workers** (Horizontally Scalable)
   - Process signal evaluation jobs from Redis queue
   - Three job types: FetchCandles → EvaluateSignal → StoreSignal
   - Can run multiple instances for parallel processing
   - Reads from Redis/QuestDB (never creates connections)

## 📁 Project Structure

```
perptrix/
  config.example.json   # Example configuration file with category weights
  src/
    bin/                # Executable binaries
      api-server.rs     # HTTP API server (stateless, scalable)
      websocket-service.rs  # WebSocket data ingestion (singleton)
      worker.rs         # Job processing workers (scalable)
    common/             # Shared helpers (math utilities: EMA, SMA, std dev)
    config/             # Configuration management (JSON-based config)
    core/               # Core runtime components
      ├── http.rs       # HTTP endpoints (health check, metrics)
      ├── runtime.rs    # Apalis worker setup
      └── scheduler.rs  # Cron-based job scheduler
    db/                 # Persistence adapters (QuestDB)
    cache/              # Caching layer (Redis)
    jobs/               # Job queue system
      ├── context.rs    # Job context for dependency injection
      ├── handlers.rs   # Job handlers (fetch, evaluate, store)
      ├── types.rs      # Job type definitions
      └── workflow.rs   # Workflow utilities
    evaluation/         # Signal scoring and validation utilities
    engine/             # Signal aggregation and scoring
      ├── aggregator.rs # Category-based signal aggregation (integer scoring)
      └── signal.rs     # Trading signal types and market bias
    indicators/         # Indicator implementations organized by category
      ├── momentum/     # MACD, RSI
      ├── trend/        # EMA, SuperTrend
      ├── volatility/   # Bollinger Bands, ATR
      ├── volume/       # OBV, Volume Profile (beyond RFC Phase 2)
      ├── perp/         # Funding Rate, Open Interest (beyond RFC Phase 2)
      └── registry.rs   # Indicator registry and category system
    models/             # Shared DTOs (Candle, IndicatorSet, SignalOutput)
    services/           # Market data provider interface
      hyperliquid/      # Hyperliquid WebSocket and REST clients
        client.rs       # WebSocket client with reconnection logic
        messages.rs     # WebSocket message types
        provider.rs     # Market data provider implementation
        rest.rs         # REST API client for historical data
        subscriptions.rs # Subscription management
      websocket/        # WebSocket service management
    signals/            # Signal evaluation engine
      ├── decision.rs   # Direction thresholds and SL/TP logic
      └── engine.rs     # Main signal evaluation orchestrator
    strategies/         # Strategy definitions (placeholder)
    lib.rs              # Crate root exposing layered modules
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

Perptrix uses Docker Compose to run all services together. This includes:
- Infrastructure services (QuestDB, Redis, Prometheus, Grafana, Tempo)
- Perptrix services (WebSocket service, API server, Workers)

#### Quick Start

Start all services:

```bash
docker-compose up -d
```

This will start:
- **QuestDB** on ports 9000 (HTTP) and 8812 (PostgreSQL wire protocol)
- **Redis** on port 6379
- **Prometheus** on port 9090 (metrics collection)
- **Grafana** on port 3000 (monitoring dashboard)
- **Grafana Tempo** on ports 4318 (OTLP HTTP) and 3200 (query API) (trace storage)
- **Perptrix WebSocket Service** (maintains market data connection)
- **Perptrix API Server** on port 8080 (HTTP API)
- **Perptrix Worker** (processes signal evaluation jobs)

#### Environment Variables

You can customize the services using environment variables:

```bash
# Set symbols to monitor
SYMBOLS=BTC,ETH,SOL

# Set evaluation interval (seconds)
EVAL_INTERVAL_SECONDS=60

# Set API port
API_PORT=8080

# Set worker concurrency
WORKER_CONCURRENCY=5

# Use sandbox environment
PERPTRIX_ENV=sandbox

# Start with custom configuration
SYMBOLS=BTC,ETH EVAL_INTERVAL_SECONDS=30 docker-compose up -d
```

#### Scaling Workers

To run multiple worker instances:

```bash
docker-compose up -d --scale worker=3
```

This will start 3 worker containers for parallel job processing.

#### Viewing Logs

View logs for all services:

```bash
docker-compose logs -f
```

View logs for a specific service:

```bash
docker-compose logs -f websocket-service
docker-compose logs -f api-server
docker-compose logs -f worker
```

#### Stopping Services

Stop all services:

```bash
docker-compose down
```

Stop and remove volumes (clears all data):

```bash
docker-compose down -v
```

#### Accessing Services

- **QuestDB Console**: http://localhost:9000
- **API Server**: http://localhost:8080
  - Health: http://localhost:8080/health
  - Metrics: http://localhost:8080/metrics
- **Grafana**: http://localhost:3000 (default credentials: admin/admin)
- **Prometheus**: http://localhost:9090
- **Tempo**: http://localhost:3200

#### Building Images

To rebuild the Perptrix services after code changes:

```bash
docker-compose build
docker-compose up -d
```

Or rebuild and restart in one command:

```bash
docker-compose up -d --build
```

#### Building Individual Services

You can also build individual Docker images using the `BINARY` build argument:

```bash
# Build API server
docker build --build-arg BINARY=api-server -t perptrix-api-server .

# Build WebSocket service
docker build --build-arg BINARY=websocket-service -t perptrix-websocket-service .

# Build Worker
docker build --build-arg BINARY=worker -t perptrix-worker .
```

Each Dockerfile build creates a single binary image, making builds faster and images smaller.

## 🚀 Usage

### Service Architecture

Perptrix consists of three independent services that can be run separately:

1. **WebSocket Service** - Maintains connection to market data provider (singleton)
2. **API Server** - HTTP API with health, metrics, and business logic (horizontally scalable)
3. **Workers** - Process signal evaluation jobs from queue (horizontally scalable)

### Setup

1. Copy the environment template to create your local `.env` file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` to adjust configuration values as needed for your local setup.

**Note:** The `.env` file is gitignored and will not be committed. Use `.env.example` as a template. All services automatically load `.env` on startup via [`dotenvy`](https://crates.io/crates/dotenvy).

### Running Services

#### 1. WebSocket Service (Required - Run First)

The WebSocket service maintains the connection to the market data provider and writes data to Redis/QuestDB. **Run this as a singleton (one instance)**.

```bash
# Basic usage
cargo run --bin websocket-service

# With symbols to subscribe
SYMBOLS=BTC,ETH cargo run --bin websocket-service

# Sandbox environment
PERPTRIX_ENV=sandbox SYMBOLS=BTC cargo run --bin websocket-service
```

#### 2. API Server (Optional - Can Run Multiple Instances)

The API server provides HTTP endpoints for health checks, metrics, and business logic. **This is stateless and can be horizontally scaled**.

```bash
# Basic usage (default port 8080)
cargo run --bin api-server

# Custom port
PORT=3000 cargo run --bin api-server

# Multiple instances (different ports)
PORT=8080 cargo run --bin api-server &
PORT=8081 cargo run --bin api-server &
```

#### 3. Workers (Required - Can Run Multiple Instances)

Workers process signal evaluation jobs from the Redis queue. **Can run multiple instances for parallel processing**.

```bash
# Basic usage
EVAL_INTERVAL_SECONDS=60 SYMBOLS=BTC cargo run --bin worker

# Custom concurrency
WORKER_CONCURRENCY=10 EVAL_INTERVAL_SECONDS=60 SYMBOLS=BTC,ETH cargo run --bin worker

# Multiple worker instances
EVAL_INTERVAL_SECONDS=60 SYMBOLS=BTC cargo run --bin worker &
EVAL_INTERVAL_SECONDS=60 SYMBOLS=BTC cargo run --bin worker &
```

### Complete Example

For a full setup, run all three services:

```bash
# Terminal 1: WebSocket Service (singleton)
SYMBOLS=BTC,ETH cargo run --bin websocket-service

# Terminal 2: API Server
PORT=8080 cargo run --bin api-server

# Terminal 3: Worker (can run multiple)
EVAL_INTERVAL_SECONDS=60 SYMBOLS=BTC,ETH cargo run --bin worker
```

### Environment Variables

**Common Variables (all services):**
- `PERPTRIX_ENV` - Environment: `sandbox` or `production` (default: `production`)
  - `sandbox` - Uses Hyperliquid testnet (wss://api.hyperliquid-testnet.xyz/ws)
  - `production` - Uses Hyperliquid mainnet (wss://api.hyperliquid.xyz/ws)
- `QUESTDB_URL` - QuestDB connection string (default: `host=localhost user=admin password=quest port=8812`)
- `REDIS_URL` - Redis connection string (default: `redis://127.0.0.1/`)
- `HISTORICAL_CANDLE_COUNT` - Number of historical candles to fetch on startup (default: 200)
- `OTEL_EXPORTER_OTLP_ENDPOINT` - OpenTelemetry OTLP endpoint for traces (default: `http://localhost:4318`)
- `OTEL_SERVICE_NAME` - Service name for traces (default: `perptrix-signal-engine`)

**WebSocket Service:**
- `SYMBOLS` - Comma-separated list of symbols to subscribe to (optional, can be configured in workers)

**API Server:**
- `PORT` - HTTP server port (default: 8080)

**Workers:**
- `EVAL_INTERVAL_SECONDS` - Signal evaluation interval in seconds (required, must be > 0)
- `SYMBOLS` - Comma-separated list of symbols to evaluate (required)
- `WORKER_CONCURRENCY` - Number of concurrent jobs per worker (default: number of symbols)

### Health Check

The API server exposes a health check endpoint:

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

### How It Works

1. **WebSocket Service** connects to the market data provider and receives real-time updates
2. Updates are stored in **Redis** (cache) and **QuestDB** (persistent storage)
3. **Workers** periodically enqueue `FetchCandlesJob` for each symbol (via cron scheduler)
4. Jobs are processed in sequence: FetchCandles → EvaluateSignal → StoreSignal
5. **API Server** provides endpoints to query signals, metrics, and health status

All services communicate via Redis/QuestDB - there's no direct coupling between services.

### Metrics Endpoint

The API server exposes a Prometheus metrics endpoint:

```bash
curl http://localhost:8080/metrics
```

This endpoint returns metrics in Prometheus text format, including:
- **HTTP Metrics**: Request count, latency, in-flight requests
- **Signal Metrics**: Evaluation count, duration, active evaluations, errors
- **System Metrics**: Database, cache, and WebSocket connection status
- **Job Queue Metrics**: Job processing rates, queue depth, worker status

### Observability

Perptrix includes comprehensive observability with Prometheus metrics and OpenTelemetry tracing:

**Metrics (Prometheus):**
- HTTP request metrics (count, latency, errors)
- Signal evaluation metrics (count, duration, success/failure)
- System health metrics (database, cache, WebSocket connection status)

**Traces (OpenTelemetry → Grafana Tempo):**
- HTTP request traces (automatic via middleware)
- Signal evaluation lifecycle
- Database operations
- Cache operations
- WebSocket message processing

**Monitoring Stack:**
- **Prometheus**: Scrapes metrics from the `/metrics` endpoint every 10 seconds
- **Grafana**: Pre-configured with Prometheus and Tempo datasources for visualization
- **Grafana Tempo**: Receives traces via OTLP HTTP on port 4318

To view metrics and traces:
1. Start all services: `docker-compose up -d`
2. Access Grafana at http://localhost:3000 (admin/admin)
3. Create dashboards using the pre-configured Prometheus and Tempo datasources

**Note:** If Tempo is unavailable, the application will continue without tracing. Metrics are always available via the `/metrics` endpoint.

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
- Measures perpetual swap funding bias with live + historical pulls
- Detects extreme positioning/crowding and widens SL/TP when necessary
- Signals: Extreme long bias (bearish fade), extreme short bias (bullish squeeze), moderate lean adjusts perp score ±1

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
   - Extreme funding rates (increase risk when trading with the crowd, reduce it when fading extremes)
   - Weak total score (increases risk)
   - RSI divergences (decreases risk)
   - Funding-driven SL/TP widening when crowds are stretched (protects against forced exits)

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

### ✅ Phase 3.5 — Production Job Queue (Completed)
- Apalis job queue system with Redis backend
- Separated services: WebSocket service, API server, workers
- Job workflow: FetchCandlesJob → EvaluateSignalJob → StoreSignalJob
- Cron-based job scheduling
- Automatic retries with exponential backoff
- Horizontal scalability for API servers and workers
- WebSocket service as singleton for data ingestion

### 🔜 Phase 3 — Remaining
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

