# polymarket-hft

[![Crates.io](https://img.shields.io/crates/v/polymarket-hft.svg)](https://crates.io/crates/polymarket-hft)
[![Documentation](https://docs.rs/polymarket-hft/badge.svg)](https://docs.rs/polymarket-hft)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-frequency trading (HFT) system for [Polymarket](https://polymarket.com) with built-in API clients and CLI.

## Features

- 🚀 **High Performance** - Built on Tokio for high-performance async operations
- 📊 **Built-in APIs** - Integrated clients for Data API, CLOB, CLOB WebSocket, Gamma, and RTDS
- 🔒 **Type-Safe** - Strongly typed API with comprehensive error handling
- 🛠️ **CLI Tool** - Command-line interface for quick API access and testing
- ⚡ **Low Latency** - Optimized for trading scenarios requiring fast execution
- 📚 **Well-Documented** - Extensive documentation and examples

## Documentation

| Document                                   | Description                           |
| ------------------------------------------ | ------------------------------------- |
| [Library Guide](./docs/library.md)         | How to use the SDK as a Rust library  |
| [CLI Guide](./docs/cli.md)                 | How to use the command-line interface |
| [API Docs](https://docs.rs/polymarket-hft) | Full API documentation                |
| [Examples](./examples)                     | Example code                          |

## Quick Start

### As a Library

```rust
use polymarket_hft::data::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let health = client.health().await?;
    println!("API status: {}", health.data);
    Ok(())
}
```

👉 See [Library Guide](./docs/library.md) for more details.

### As a CLI

```bash
# Check API health
cargo run -- data health

# Get user's traded markets count
cargo run -- data get-user-traded-markets -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839
```

👉 See [CLI Guide](./docs/cli.md) for more details.

## Supported APIs

| API               | Status | Description                                                                                      |
| ----------------- | ------ | ------------------------------------------------------------------------------------------------ |
| **Data API**      | ✅     | Health, holders, value, traded, open interest, live volume, positions, trades, activity         |
| **CLOB**          | 🚧     | Orderbook, pricing, spreads, orders, trades                                                      |
| **CLOB WebSocket**| 🚧     | Real-time orderbook updates, trade streams                                                       |
| **Gamma Markets** | 🚧     | Sports, events, markets, search                                                                  |
| **RTDS**          | 🚧     | Real-time price feeds and comments                                                               |

## Installation

```toml
[dependencies]
polymarket-hft = "0.1"
```

## Architecture

```text
polymarket-hft/
├── src/
│   ├── data/          # Data API client
│   ├── clob/          # CLOB REST API client (planned)
│   ├── clob_ws/       # CLOB WebSocket client (planned)
│   ├── gamma/         # Gamma Markets API client (planned)
│   ├── rtds/          # RTDS streaming client (planned)
│   ├── commands/      # CLI command implementations
│   └── main.rs        # CLI entry point
```

## Development Status

🚧 **This project is in active development.**

### Current Status

- ✅ Project structure and module organization
- ✅ Data API client with 10 endpoints:
  - `health` - API health check
  - `get_market_top_holders` - Get top holders for markets
  - `get_user_portfolio_value` - Get total value of user's positions
  - `get_user_traded_markets` - Get user's traded markets count
  - `get_open_interest` - Get open interest for markets
  - `get_event_live_volume` - Get live volume for an event
  - `get_user_positions` - Get current positions for a user
  - `get_user_closed_positions` - Get closed positions for a user
  - `get_user_activity` - Get on-chain activity for a user
  - `get_trades` - Get trades for a user or markets
- ✅ CLI tool for Data API
- ✅ CI/CD pipeline (GitHub Actions)

### Roadmap

1. ~~Implement Data API endpoints~~ ✅
2. Implement CLOB REST API endpoints
3. Implement CLOB WebSocket connectivity
4. Implement Gamma Markets API endpoints
5. Implement RTDS streaming
6. Add HFT trading strategies framework
7. Add comprehensive integration tests
8. Publish to crates.io

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

```bash
# Clone the repository
git clone https://github.com/telepair/polymarket-hft.git
cd polymarket-hft

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE)
file for details.

## Disclaimer

This is an unofficial SDK and is not affiliated with Polymarket.
Use at your own risk. This software is intended for educational and research purposes.

## Links

- [Polymarket](https://polymarket.com)
- [Polymarket API Docs](https://docs.polymarket.com)
- [GitHub](https://github.com/telepair/polymarket-hft)
- [Issues](https://github.com/telepair/polymarket-hft/issues)
