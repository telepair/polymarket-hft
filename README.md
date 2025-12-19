# polymarket-hft

[![Crates.io](https://img.shields.io/crates/v/polymarket-hft.svg)](https://crates.io/crates/polymarket-hft)
[![Documentation](https://docs.rs/polymarket-hft/badge.svg)](https://docs.rs/polymarket-hft)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> [!CAUTION] > **Early Development (Pre-0.1.0)** - API wrappers are not fully tested. Breaking changes may occur. Do not use in production.

A high-frequency trading (HFT) system for [Polymarket](https://polymarket.com) with built-in API clients and CLI.

## Architecture Overview

The system is designed as a modular, event-driven architecture:

```text
Clients → Ingestors → Dispatcher → Policy Engine → Action Executor
                              ↓
                State Manager + Archiver
```

**Supported Clients & APIs** - Currently implemented clients and APIs are:

| API                      | Protocol         | Status |
| ------------------------ | ---------------- | ------ |
| Polymarket Data API      | REST             | ✅     |
| Polymarket Gamma Markets | REST             | ✅     |
| Polymarket CLOB          | REST + WebSocket | ✅     |
| Polymarket RTDS          | WebSocket        | ✅     |
| CoinMarketCap Standard   | REST             | ✅     |

**Storage** - State Manager and Archiver are implemented using Redis and TimescaleDB.

**Policy Engine** — Define trading rules via YAML/JSON without code:

```yaml
policies:
  - id: btc_alert
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

**Action Executor** - supports notifications, orders and audit logging.

See [Architecture](./docs/architecture.md) and [Policy Engine](./docs/policy.md) for details.

## Documentation

| Document                                   | Description                     |
| ------------------------------------------ | ------------------------------- |
| [Client Documentation](./docs/client.md)   | Usage guide for all API clients |
| [CLI Guide](./docs/cli.md)                 | Command-line interface usage    |
| [CLI Examples](./docs/cli_examples.md)     | Practical CLI examples          |
| [Architecture](./docs/architecture.md)     | System design and HFT engine    |
| [Policy Engine](./docs/policy.md)          | User-defined policy DSL         |
| [API Docs](https://docs.rs/polymarket-hft) | Full API reference              |

## Quick Start

### As a Library

```toml
[dependencies]
polymarket-hft = "0.0.6"
```

```rust
use polymarket_hft::client::polymarket::data::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let health = client.health().await?;
    println!("API status: {}", health.data);
    Ok(())
}
```

### As a CLI

```bash
cargo run -- data health
cargo run -- gamma get-markets -l 5
cargo run -- clob get-orderbook -m "0x..."
```

> [!NOTE] > **CoinMarketCap Integration**: The CoinMarketCap client is designed for the **Basic Plan** (free tier). You will need an API key from [CoinMarketCap Developer Portal](https://coinmarketcap.com/api/documentation/v1/).

See [Client Documentation](./docs/client.md) and [CLI Guide](./docs/cli.md) for details.

## Roadmap

- ✅ **v0.0.x** - API clients (Data, Gamma, CLOB, RTDS) and CLI
- 🚧 **v0.1.x** - HFT Engine: Dispatcher, State Management, Strategy Engine
- 📋 **v1.0.x** - Production-ready with validated trading strategies

See [Architecture](./docs/architecture.md) for detailed HFT engine design.

## Contributing

```bash
git clone https://github.com/telepair/polymarket-hft.git
cd polymarket-hft
cargo build && cargo test
cargo run -- --help
```

## License

MIT License - see [LICENSE](LICENSE).

## Disclaimer

Unofficial SDK, not affiliated with Polymarket. Use at your own risk for educational and research purposes.

## Links

- [Polymarket](https://polymarket.com) | [API Docs](https://docs.polymarket.com) | [GitHub](https://github.com/telepair/polymarket-hft) | [Issues](https://github.com/telepair/polymarket-hft/issues)
