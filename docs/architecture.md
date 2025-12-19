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
│  │                    CoinMarketCap API Client                           │  │
│  │  ┌───────────────────────────────────────────────────────────┐       │  │
│  │  │  CMC Client (REST) - Listings, Global Metrics, Fear&Greed │       │  │
│  │  └───────────────────────────────────────────────────────────┘       │  │
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
│                         Ingestors 📋 PLANNED                                │
│                                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                          │
│  │  WS Actor   │  │Poller Actor │  │ Cron Actor  │                          │
│  │ (RTDS/CLOB) │  │ (REST APIs) │  │  (Daily)    │                          │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                          │
│         └────────────────┼────────────────┘                                  │
│                          │ MarketEvent                                       │
│                          ▼                                                   │
│            ┌─────────────────────────┐                                       │
│            │       Dispatcher        │                                       │
│            │  - Message routing      │                                       │
│            │  - Backpressure control │                                       │
│            └────────────┬────────────┘                                       │
└─────────────────────────┼───────────────────────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│  Archiver   │   │   State     │   │   Policy    │
│ 📋 PLANNED  │   │  Manager    │   │   Engine    │
│             │   │ 📋 PLANNED  │   │ 📋 PLANNED  │
└──────┬──────┘   └──────┬──────┘   └──────┬──────┘
       ▼                 ▼                 ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Storage Layer 📋 PLANNED                             │
│                                                                              │
│  ┌────────────────────────┐        ┌────────────────────────┐               │
│  │     TimescaleDB        │        │         Redis          │               │
│  │  (Cold/Warm Data)      │        │  (Hot Data, TTL:15min) │               │
│  └────────────────────────┘        └────────────────────────┘               │
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

#### CoinMarketCap Client

| Client | Protocol | Key Features                                                |
| ------ | -------- | ----------------------------------------------------------- |
| CMC    | REST     | Cryptocurrency listings, global metrics, fear & greed index |

**Shared Infrastructure**:

- HTTP client with exponential backoff retry (3 attempts)
- WebSocket auto-reconnect with subscription recovery
- Connection pooling (10 idle connections per host)

### Ingestors 📋 PLANNED

Data collection actors that emit `MarketEvent` messages.

| Actor        | Source              | Description                               |
| ------------ | ------------------- | ----------------------------------------- |
| WS Actor     | RTDS/CLOB WebSocket | Real-time price, orderbook, trade streams |
| Poller Actor | REST APIs           | Market metadata, positions, balances      |
| Cron Actor   | Scheduled tasks     | Daily snapshots, cleanup, aggregations    |

### Dispatcher 📋 PLANNED

Central message hub routing `MarketEvent` to multiple consumers.

**Design Choice**: Dispatcher pattern over `tokio::sync::broadcast`:

- Independent `mpsc` channel per consumer
- Slow consumers don't block others
- Per-consumer message filtering and backpressure

### Processors 📋 PLANNED

#### Archiver

Buffers events and batch-writes to TimescaleDB (100 events or 1 second threshold).

#### State Manager

Maintains real-time state using local cache + Redis Pub/Sub to eliminate round-trip latency.

#### Policy Engine

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

## Data Layer 📋 PLANNED

### Hot Data (Redis)

| Key Pattern                            | Description                   |
| -------------------------------------- | ----------------------------- |
| `polymarket:price:{asset_id}`          | Current price, bid, ask       |
| `polymarket:orderbook:{market}`        | Price levels with sizes       |
| `polymarket:position:{wallet}:{asset}` | Position size, avg price, PnL |

### Cold Data (TimescaleDB)

```sql
-- Price time-series with continuous aggregation
CREATE TABLE prices (
    time TIMESTAMPTZ NOT NULL, asset_id TEXT NOT NULL,
    price NUMERIC(20,8), bid NUMERIC(20,8), ask NUMERIC(20,8)
);
SELECT create_hypertable('prices', 'time');

-- Hourly OHLCV aggregation
CREATE MATERIALIZED VIEW prices_1h WITH (timescaledb.continuous) AS
SELECT time_bucket('1 hour', time) AS bucket, asset_id,
       first(price, time) AS open, max(price) AS high,
       min(price) AS low, last(price, time) AS close
FROM prices GROUP BY bucket, asset_id;
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
│   ├── coinmarketcap/   # ✅ CoinMarketCap APIs (Listings, Metrics, F&G)
│   ├── http.rs          # ✅ Shared HTTP client with retry
│   └── {other}/         # 📋 Future data sources
├── engine/              # 📋 HFT engine
│   ├── events.rs        #    MarketEvent definitions
│   ├── dispatcher.rs    #    Message dispatcher
│   ├── ingestors/       #    WS, Poller, Cron actors
│   ├── state.rs         #    State Manager
│   ├── archiver.rs      #    TimescaleDB batch writer
│   ├── policy/          #    Policy engine (user-defined rules)
│   └── executor.rs      #    Action executor
├── storage/             # 📋 Redis + TimescaleDB clients
└── cli/                 # ✅ CLI commands
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

| Phase                  | Components                      | Status  |
| ---------------------- | ------------------------------- | ------- |
| 1. Core Infrastructure | events, dispatcher, ws ingestor | 📋 Next |
| 2. Data Persistence    | redis, timescale, archiver      | 📋      |
| 3. Policy Engine       | state, policy DSL, evaluator    | 📋      |
| 4. Execution Layer     | executor, notifications         | 📋      |
| 5. Operations          | Metrics, tracing, health checks | 📋      |
