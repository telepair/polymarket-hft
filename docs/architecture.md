# Architecture

This document describes the architecture for the polymarket-hft trading system.

## Status Legend

| Badge          | Meaning                                        |
| -------------- | ---------------------------------------------- |
| ✅ IMPLEMENTED | Production-ready, available in current release |
| 🚧 IN PROGRESS | Under active development                       |
| 📋 PLANNED     | Designed but not yet implemented               |

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                      Client Layer (SDK) ✅ IMPLEMENTED                      │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                     Polymarket API Clients                            │  │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐          │  │
│  │  │   Data    │  │   CLOB    │  │   Gamma   │  │   RTDS    │          │  │
│  │  │  (REST)   │  │(REST + WS)│  │  (REST)   │  │   (WS)    │          │  │
│  │  └───────────┘  └───────────┘  └───────────┘  └───────────┘          │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                   Crypto Market Data Clients                          │  │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐       │  │
│  │  │  AlternativeMe  │  │    CoinGecko    │  │  CoinMarketCap  │       │  │
│  │  │  (REST, Free)   │  │  (REST, API Key)│  │  (REST, API Key)│       │  │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘       │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                   │                                          │
│                    ┌──────────────▼──────────────┐                           │
│                    │    Shared HTTP Client       │                           │
│                    │  (retry, timeout, pooling)  │                           │
│                    └─────────────────────────────┘                           │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Ingestors 🚧 IN PROGRESS                                 │
│                                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                          │
│  │  WS Actor   │  │Poller Actor │  │ Cron Actor  │                          │
│  │ (RTDS/CLOB) │  │ (REST APIs) │  │  (Interval) │                          │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                          │
│         └────────────────┼────────────────┘                                  │
│                          │ Metric                                            │
│                          ▼                                                   │
│            ┌─────────────────────────┐                                       │
│            │    IngestorManager      │                                       │
│            │  - Job scheduling       │                                       │
│            │  - Cron/Interval based  │                                       │
│            └────────────┬────────────┘                                       │
└─────────────────────────┼───────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                   Storage Layer ✅ IMPLEMENTED                              │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │              StorageBackend Trait (store, get_latest, query_range)  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                          │                                                   │
│     ┌────────────────────┴────────────────────┐                             │
│     ▼                                         ▼                             │
│  ┌──────────────────────────┐    ┌──────────────────────────┐               │
│  │ LocalStorage ✅ DEFAULT  │    │ ExternalStorage 📋 PLAN  │               │
│  │  ┌─────────────────────┐ │    │  ┌─────────────────────┐ │               │
│  │  │  MemoryCache (moka) │ │    │  │    Redis (Hot)      │ │               │
│  │  │  TTL: 15min         │ │    │  │    TTL: 15min       │ │               │
│  │  └─────────────────────┘ │    │  └─────────────────────┘ │               │
│  │  ┌─────────────────────┐ │    │  ┌─────────────────────┐ │               │
│  │  │  SQLite (WAL mode)  │ │    │  │    TimescaleDB      │ │               │
│  │  │  Persistent Storage │ │    │  │    Time-series      │ │               │
│  │  └─────────────────────┘ │    │  └─────────────────────┘ │               │
│  └──────────────────────────┘    └──────────────────────────┘               │
└─────────────────────────────────────────────────────────────────────────────┘
                                          │
                                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Action Executor 📋 PLANNED                              │
│                                                                              │
│  ┌───────────────────┐ ┌───────────────────┐ ┌───────────────────┐          │
│  │   Order Executor  │ │   Notification    │ │   Audit Logger    │          │
│  └───────────────────┘ └───────────────────┘ └───────────────────┘          │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Components

### Client Layer ✅ IMPLEMENTED

Multi-source client architecture under `src/client/`. Currently implements Polymarket and CoinMarketCap APIs with extensibility for future data sources. See [Client Guide](./client.md) for usage details.

#### Polymarket Clients

| Client | Protocol  | Key Features                                         |
| ------ | --------- | ---------------------------------------------------- |
| Data   | REST      | User positions, trades, portfolio value              |
| CLOB   | REST + WS | Order management, EIP-712 signing, real-time updates |
| Gamma  | REST      | Market metadata, events, search                      |
| RTDS   | WebSocket | Real-time prices, trades, orderbook streams          |

#### AlternativeMe Client

| Client       | Protocol | Key Features                                               |
| ------------ | -------- | ---------------------------------------------------------- |
| AlternativeMe | REST     | Fear & Greed Index, cryptocurrency tickers, global metrics (free, no API key) |

#### CoinGecko Client

| Client   | Protocol | Key Features                                             |
| -------- | -------- | -------------------------------------------------------- |
| CoinGecko | REST     | Simple prices, market data, trending, global, OHLC data (API key required) |

#### CoinMarketCap Client

| Client | Protocol | Key Features                                                |
| ------ | -------- | ----------------------------------------------------------- |
| CMC    | REST     | Cryptocurrency listings, global metrics, fear & greed index (API key required) |

**Shared Infrastructure**:

- HTTP client with exponential backoff retry (3 attempts)
- WebSocket auto-reconnect with subscription recovery
- Connection pooling (10 idle connections per host)

### Ingestor Manager 🚧 IN PROGRESS

Schedules and executes data collection jobs based on YAML configuration.

| Schedule Type | Description                              |
| ------------- | ---------------------------------------- |
| Interval      | Fixed interval (e.g., every 60 seconds)  |
| Cron          | Cron expression (e.g., `0 0 * * *`)      |

**Features:**

- Dynamic job loading from YAML configuration
- Per-job retention period configuration
- Graceful shutdown handling

### Storage Layer ✅ IMPLEMENTED

Pluggable storage backend with write-through caching strategy.

#### StorageBackend Trait

Core trait defining storage operations:

```rust
pub trait StorageBackend: Send + Sync {
    fn store(&self, metrics: &[Metric]) -> BoxFuture<'_, Result<()>>;
    fn get_latest(&self, source: &str, name: &str) -> BoxFuture<'_, Result<Option<Metric>>>;
    fn query_range(&self, source, name, start, end, limit) -> BoxFuture<'_, Result<Vec<Metric>>>;
    fn cleanup_before(&self, cutoff_timestamp: i64) -> BoxFuture<'_, Result<u64>>;
    fn health_check(&self) -> BoxFuture<'_, Result<()>>;
}
```

#### LocalStorage (Default)

Combined local storage with in-memory cache and SQLite persistence.

| Component      | Technology | Purpose                              |
| -------------- | ---------- | ------------------------------------ |
| MemoryCache    | moka       | Hot data with TTL (default: 15min)   |
| SqliteStorage  | sqlx       | Persistent storage with WAL mode     |

**Write Strategy**: Write-through (writes to both cache and SQLite)
**Read Strategy**: Cache-first (cache hit returns immediately, fallback to SQLite)

**SQLite Optimizations:**

- WAL mode for better concurrency
- Multi-row INSERT batches (100 rows/batch)
- Automatic cleanup of old metrics

#### ExternalStorage 📋 PLANNED

Distributed storage for multi-instance deployments.

| Component   | Technology  | Purpose                    |
| ----------- | ----------- | -------------------------- |
| Hot Cache   | Redis       | Real-time state, Pub/Sub   |
| Cold Store  | TimescaleDB | Time-series persistence    |

### Policy Engine 📋 PLANNED

User-defined policies via YAML/JSON configuration. See [Policy Engine Guide](./policy.md) for details.

**Key Features:**

- **Declarative DSL** — Define conditions and actions without code
- **Composite Conditions** — AND/OR logic with time-window support
- **Multiple Actions** — Notifications, orders, webhooks
- **Rate Limiting** — Built-in cooldown per policy

```yaml
# Example: Price alert policy
policies:
  - id: btc_low_alert
    conditions:
      field: price
      asset: "BTC"
      operator: crosses_below
      value: 80000
    actions:
      - type: notification
        channel: telegram
        template: "BTC below $80K!"
```

### Action Executor 📋 PLANNED

| Executor       | Responsibility                            |
| -------------- | ----------------------------------------- |
| Order Executor | Submit/cancel orders via CLOB Trading API |
| Notification   | Send alerts via Telegram                  |
| Audit Logger   | Record all actions to TimescaleDB         |

## Data Layer

### Local Storage Schema (SQLite) ✅ IMPLEMENTED

```sql
CREATE TABLE metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,          -- e.g., 'alternativeme', 'polymarket'
    name TEXT NOT NULL,            -- e.g., 'fear_and_greed_index'
    value REAL NOT NULL,
    timestamp INTEGER NOT NULL,    -- Unix timestamp
    labels TEXT,                   -- JSON object for additional labels
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Indexes for efficient queries
CREATE INDEX idx_metrics_source_name_ts ON metrics(source, name, timestamp DESC);
CREATE INDEX idx_metrics_timestamp ON metrics(timestamp DESC);
```

### Configuration (YAML)

```yaml
storage:
  backend: local  # 'local' or 'external' (future)
  local:
    db_path: "data/metrics.db"
    cache_ttl_secs: 900        # 15 minutes
    cache_max_capacity: 10000
```

### External Storage (Future) 📋 PLANNED

#### Hot Data (Redis)

| Key Pattern                   | Description             |
| ----------------------------- | ----------------------- |
| `{source}::{name}`            | Latest metric value     |
| `{source}::{name}::{label}`   | Metric with label       |

#### Cold Data (TimescaleDB)

```sql
CREATE TABLE metrics (
    time TIMESTAMPTZ NOT NULL,
    source TEXT NOT NULL,
    name TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    labels JSONB
);
SELECT create_hypertable('metrics', 'time');
```

## Event Types 📋 PLANNED

```rust
pub enum MarketEvent {
    PriceUpdate { asset_id: String, price: Decimal, bid: Option<Decimal>, ask: Option<Decimal>, timestamp: u64 },
    OrderBookSnapshot { market: String, bids: Vec<PriceLevel>, asks: Vec<PriceLevel>, timestamp: u64 },
    Trade { market: String, side: Side, price: Decimal, size: Decimal, timestamp: u64 },
    PositionUpdate { wallet: String, asset_id: String, size: Decimal, avg_price: Decimal },
}
```

## Directory Structure

```text
src/
├── client/              # API clients
│   ├── polymarket/      # ✅ Polymarket APIs (Data, CLOB, Gamma, RTDS)
│   ├── alternativeme/   # ✅ Alternative.me APIs (Fear & Greed, Tickers, Global)
│   ├── coingecko/       # ✅ CoinGecko APIs (Prices, Markets, Trending, OHLC)
│   ├── coinmarketcap/   # ✅ CoinMarketCap APIs (Listings, Quotes, Metrics)
│   ├── http.rs          # ✅ Shared HTTP client with retry
│   └── {other}/         # 📋 Future data sources
├── config/              # ✅ Configuration management
│   ├── settings.rs      #    App config, storage config
│   └── job.rs           #    Ingestion job definitions
├── ingestor/            # 🚧 Data ingestion
│   └── manager.rs       #    Job scheduler with cron/interval support
├── storage/             # ✅ Storage layer
│   ├── backend.rs       #    StorageBackend trait definition
│   ├── local.rs         #    LocalStorage (SQLite + moka cache)
│   ├── sqlite.rs        #    SQLite backend with WAL mode
│   ├── cache.rs         #    In-memory cache with TTL (moka)
│   ├── model.rs         #    Metric, DataSource definitions
│   └── archiver.rs      #    Legacy archiver trait (deprecated)
├── engine/              # 📋 HFT engine (future)
│   ├── policy/          #    Policy engine (user-defined rules)
│   └── executor.rs      #    Action executor
└── cli/                 # ✅ CLI commands
    └── serve.rs         #    Data ingestion server
```

## Design Decisions

| Decision          | Choice                         | Rationale                           |
| ----------------- | ------------------------------ | ----------------------------------- |
| Message Bus       | Dispatcher (mpsc per consumer) | Avoid slow consumer blocking        |
| Policy Definition | YAML/JSON DSL                  | User-defined without recompilation  |
| State Sync        | Local cache + Pub/Sub          | Eliminate Redis round-trip per tick |
| Data TTL          | Redis 15 minutes               | Support technical indicators        |
| Batch Write       | 100 events / 1 second          | Balance throughput vs latency       |

## Implementation Phases

| Phase                  | Components                              | Status         |
| ---------------------- | --------------------------------------- | -------------- |
| 1. Client Layer        | Polymarket, CMC, AlternativeMe clients  | ✅ IMPLEMENTED |
| 2. Storage Layer       | LocalStorage (SQLite + moka)            | ✅ IMPLEMENTED |
| 3. Ingestor Manager    | Job scheduling, interval/cron support   | 🚧 IN PROGRESS |
| 4. External Storage    | Redis + TimescaleDB backend             | 📋 PLANNED     |
| 5. Policy Engine       | state, policy DSL, evaluator            | 📋 PLANNED     |
| 6. Execution Layer     | executor, notifications                 | 📋 PLANNED     |
| 7. Operations          | Metrics, tracing, health checks         | 📋 PLANNED     |
