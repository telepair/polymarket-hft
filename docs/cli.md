# CLI Reference

Command-line interface reference for `polymarket-hft`.

> [!TIP]
> For quick start with working examples, see [CLI Examples](cli_examples.md).

## Installation

### From Source

```bash
git clone https://github.com/telepair/polymarket-hft.git
cd polymarket-hft
cargo install --path .
```

### From crates.io

```bash
cargo install polymarket-hft
```

## Usage

```bash
polymarket [COMMAND] [SUBCOMMAND] [OPTIONS]
```

---

## Data API

### Health Check

```bash
polymarket data health
```

### User Commands

#### get-user-positions

Get current positions for a user.

| Option                    | Description                                                                      |
| ------------------------- | -------------------------------------------------------------------------------- |
| `-u, --user <ADDRESS>`    | User address (required)                                                          |
| `-m, --market <ID>`       | Market condition IDs (multiple)                                                  |
| `-e, --event-id <ID>`     | Event IDs (multiple)                                                             |
| `--size-threshold <SIZE>` | Minimum position size                                                            |
| `--redeemable <BOOL>`     | Filter redeemable positions                                                      |
| `--mergeable <BOOL>`      | Filter mergeable positions                                                       |
| `-l, --limit <N>`         | Limit results (0-500, default: 100)                                              |
| `-o, --offset <N>`        | Pagination offset (0-10000)                                                      |
| `--sort-by <FIELD>`       | CURRENT, INITIAL, TOKENS, CASHPNL, PERCENTPNL, TITLE, RESOLVING, PRICE, AVGPRICE |
| `--sort-direction <DIR>`  | ASC or DESC                                                                      |
| `-t, --title <TITLE>`     | Title filter (max 160 chars)                                                     |

#### get-user-closed-positions

Get closed positions for a user.

| Option                   | Description                                    |
| ------------------------ | ---------------------------------------------- |
| `-u, --user <ADDRESS>`   | User address (required)                        |
| `-m, --market <ID>`      | Market condition IDs (multiple)                |
| `-t, --title <TITLE>`    | Title filter (max 100 chars)                   |
| `-e, --event-id <ID>`    | Event IDs (multiple)                           |
| `-l, --limit <N>`        | Limit results (0-50, default: 10)              |
| `-o, --offset <N>`       | Pagination offset (0-100000)                   |
| `--sort-by <FIELD>`      | REALIZEDPNL, TITLE, PRICE, AVGPRICE, TIMESTAMP |
| `--sort-direction <DIR>` | ASC or DESC                                    |

#### get-user-portfolio-value

Get total value of user's positions.

| Option                 | Description                     |
| ---------------------- | ------------------------------- |
| `-u, --user <ADDRESS>` | User address (required)         |
| `-m, --market <ID>`    | Market IDs (multiple, optional) |

#### get-user-traded-markets

Get count of markets user has traded.

| Option                 | Description             |
| ---------------------- | ----------------------- |
| `-u, --user <ADDRESS>` | User address (required) |

#### get-user-activity

Get on-chain activity for a user.

| Option                   | Description                                             |
| ------------------------ | ------------------------------------------------------- |
| `-u, --user <ADDRESS>`   | User address (required)                                 |
| `-l, --limit <N>`        | Limit results (0-500, default: 100)                     |
| `-o, --offset <N>`       | Pagination offset (0-10000)                             |
| `-m, --market <ID>`      | Market condition IDs (mutually exclusive with event-id) |
| `-e, --event-id <ID>`    | Event IDs (mutually exclusive with market)              |
| `-t, --type <TYPE>`      | TRADE, SPLIT, MERGE, REDEEM, REWARD, CONVERSION         |
| `--start <TS>`           | Start timestamp                                         |
| `--end <TS>`             | End timestamp                                           |
| `--sort-by <FIELD>`      | TIMESTAMP, TOKENS, CASH                                 |
| `--sort-direction <DIR>` | ASC or DESC                                             |
| `--side <SIDE>`          | BUY or SELL                                             |

#### get-trades

Get trades for user or markets.

| Option                 | Description                                             |
| ---------------------- | ------------------------------------------------------- |
| `-u, --user <ADDRESS>` | User address (optional)                                 |
| `-m, --market <ID>`    | Market condition IDs (mutually exclusive with event-id) |
| `-e, --event-id <ID>`  | Event IDs (mutually exclusive with market)              |
| `-l, --limit <N>`      | Limit results (0-10000, default: 100)                   |
| `-o, --offset <N>`     | Pagination offset (0-10000)                             |
| `--taker-only <BOOL>`  | Filter taker-only trades (default: true)                |
| `--filter-type <TYPE>` | CASH or TOKENS (requires filter-amount)                 |
| `--filter-amount <N>`  | Filter amount (requires filter-type)                    |
| `-s, --side <SIDE>`    | BUY or SELL                                             |

### Market Commands

#### get-market-top-holders

| Option              | Description                            |
| ------------------- | -------------------------------------- |
| `-m, --market <ID>` | Market ID (required, multiple)         |
| `-l, --limit <N>`   | Limit results (0-500, default: 100)    |
| `--min-balance <N>` | Minimum balance (0-999999, default: 1) |

#### get-open-interest

| Option              | Description                    |
| ------------------- | ------------------------------ |
| `-m, --market <ID>` | Market ID (required, multiple) |

#### get-event-live-volume

| Option                | Description               |
| --------------------- | ------------------------- |
| `-i, --id <EVENT_ID>` | Event ID (required, >= 1) |

---

## Gamma API

Discovery and search metadata API.

### Core Listings

| Command       | Options                                                                                                        |
| ------------- | -------------------------------------------------------------------------------------------------------------- |
| `get-sports`  | -                                                                                                              |
| `get-teams`   | `-l`, `-o`, `--league`, `--name`, `--abbreviation`                                                             |
| `get-tags`    | `-l`, `-o`, `--include-template`, `--is-carousel`                                                              |
| `get-series`  | `-l`, `-o`, `--slug`, `--closed`, `--recurrence`                                                               |
| `get-events`  | `-l`, `-o`, `--tag-id`, `--exclude-tag-id`, `--active`, `--closed`, `--related-tags`, `--order`, `--ascending` |
| `get-markets` | `-l`, `-o`, `--id`, `--slug`, `--tag-id`, `--event-id`, `--related-tags`, `--closed`, `--include-tag`          |

### Single-Entity Lookups

| Command                        | Arguments                                          |
| ------------------------------ | -------------------------------------------------- |
| `get-tag-by-id`                | `<TAG_ID>`                                         |
| `get-tag-by-slug`              | `<SLUG>` `[--include-template]`                    |
| `get-tag-relationships-by-tag` | `<TAG_ID>` `[--status]` `[--omit-empty]`           |
| `get-tags-related-to-tag`      | `<TAG_ID>` `[--status]` `[--omit-empty]`           |
| `get-event-by-id`              | `<ID>` `[--include-chat]` `[--include-template]`   |
| `get-event-by-slug`            | `<SLUG>` `[--include-chat]` `[--include-template]` |
| `get-event-tags`               | `<EVENT_ID>`                                       |
| `get-market-by-id`             | `<ID>` `[--include-tag]`                           |
| `get-market-by-slug`           | `<SLUG>` `[--include-tag]`                         |
| `get-market-tags`              | `<MARKET_ID>`                                      |
| `get-series-by-id`             | `<SERIES_ID>` `[--include-chat]`                   |

### Comments

| Command                        | Arguments                                                                                        |
| ------------------------------ | ------------------------------------------------------------------------------------------------ |
| `get-comments`                 | `--parent-entity-type` `--parent-entity-id` `[-l]` `[-o]` `[--get-positions]` `[--holders-only]` |
| `get-comment-by-id`            | `<ID>` `[--get-positions]`                                                                       |
| `get-comments-by-user-address` | `<ADDRESS>` `[-l]` `[-o]` `[--order]` `[--ascending]`                                            |

### Search

| Command  | Arguments                                                                                                    |
| -------- | ------------------------------------------------------------------------------------------------------------ |
| `search` | `"<QUERY>"` `[--cache]` `[--events-status]` `[--events-tag]` `[--limit-per-type]` `[--page]` `[--optimized]` |

---

## CLOB API

Order book data, pricing, and spreads.

### Order Book

| Command           | Options                                |
| ----------------- | -------------------------------------- |
| `get-order-book`  | `-t <TOKEN_ID>`                        |
| `get-order-books` | `-t <TOKEN_ID>` (multiple) `[-s SIDE]` |

### Pricing

| Command              | Options                                                                     |
| -------------------- | --------------------------------------------------------------------------- |
| `get-market-price`   | `-t <TOKEN_ID>` `-s <SIDE>`                                                 |
| `get-market-prices`  | - (may return API error)                                                    |
| `get-midpoint-price` | `-t <TOKEN_ID>`                                                             |
| `get-price-history`  | `-m <TOKEN_ID>` `[--start-ts]` `[--end-ts]` `[-i INTERVAL]` `[-r FIDELITY]` |

**Interval options**: 1m, 1h, 6h, 1d, 1w, max

### Spreads

| Command       | Options                    |
| ------------- | -------------------------- |
| `get-spreads` | `-t <TOKEN_ID>` (multiple) |

---

## CoinMarketCap API

> [!NOTE]
> Requires `CMC_API_KEY` environment variable. Get a free key at [CoinMarketCap API](https://coinmarketcap.com/api/).

### Commands

| Command              | Description                        |
| -------------------- | ---------------------------------- |
| `get-listings`       | Get latest cryptocurrency listings |
| `get-global-metrics` | Get global market metrics          |
| `get-fear-and-greed` | Get Fear and Greed Index           |
| `get-key-info`       | Get API key usage information      |

### get-listings

Get top cryptocurrencies with optional filtering.

| Option                                          | Description                                     |
| ----------------------------------------------- | ----------------------------------------------- |
| `-l, --limit <N>`                               | Number of results (default: 100, max: 5000)     |
| `--start <N>`                                   | Starting position for pagination (1-based)      |
| `--price-min <N>` / `--price-max <N>`           | Filter by price range                           |
| `--market-cap-min <N>` / `--market-cap-max <N>` | Filter by market cap                            |
| `-c, --convert <CURRENCY>`                      | Currency for quotes (e.g., USD, EUR)            |
| `--sort <FIELD>`                                | Sort field: market_cap, name, price, volume_24h |
| `--sort-dir <DIR>`                              | Sort direction: asc or desc                     |
| `--cryptocurrency-type <TYPE>`                  | Filter: all, coins, tokens                      |
| `--tag <TAG>`                                   | Filter by tag: defi, filesharing, etc.          |

### get-global-metrics

Get total market cap, BTC dominance, and other global metrics.

| Option                 | Description         |
| ---------------------- | ------------------- |
| `-c, --convert <CURR>` | Currency for quotes |

### Examples

```bash
# Set API key
export CMC_API_KEY="your-api-key"

# Get top 10 cryptocurrencies
polymarket cmc get-listings -l 10

# Get global market metrics
polymarket cmc get-global-metrics

# Get Fear and Greed Index
polymarket cmc get-fear-and-greed

# Check API usage
polymarket cmc get-key-info

# Get cryptocurrencies priced above $1000
polymarket cmc get-listings --price-min 1000 -l 20

# Get top 5 DeFi tokens by market cap
polymarket cmc get-listings --tag defi --sort market_cap --sort-dir desc -l 5
```

---

## RTDS (Real-Time Data Service)

Real-time WebSocket streaming for market data, trades, and prices.

### Subscribe

Subscribe to real-time data streams.

```bash
polymarket rtds subscribe --topic <TOPIC> [OPTIONS]
```

| Option                      | Description                                |
| --------------------------- | ------------------------------------------ |
| `-t, --topic <TOPIC>`       | Topic to subscribe (required)              |
| `-T, --message-type <TYPE>` | Message type, `*` for all (default: `*`)   |
| `-f, --filter <JSON>`       | Filter in JSON format                      |
| `--clob-key <KEY>`          | CLOB API key (or `POLY_API_KEY` env)       |
| `--clob-secret <SECRET>`    | CLOB API secret (or `POLY_API_SECRET` env) |
| `--clob-passphrase <PASS>`  | CLOB passphrase (or `POLY_PASSPHRASE` env) |
| `-n, --max-messages <N>`    | Max messages to receive (default: 10)      |
| `--timeout <SECS>`          | Timeout in seconds (default: 60)           |
| `-o, --output <FORMAT>`     | `json` or `compact` (default: json)        |

**Available Topics**: `activity`, `comments`, `rfq`, `crypto_prices`, `crypto_prices_chainlink`, `equity_prices`, `clob_user`, `clob_market`

### Examples

```bash
# Subscribe to BTC price updates
polymarket rtds subscribe -t crypto_prices -f '{"symbol":"BTCUSDT"}'

# Subscribe to activity trades (compact output)
polymarket rtds subscribe -t activity -T trades -o compact -n 5

# Subscribe to CLOB user events (with auth from env)
POLY_API_KEY=xxx POLY_API_SECRET=xxx POLY_PASSPHRASE=xxx \
  polymarket rtds subscribe -t clob_user
```

---

## CLOB WebSocket

Real-time WebSocket streaming for order book updates, price changes, and user events.

### Market Channel

Subscribe to market data (order book, price changes, trades).

```bash
polymarket clob-ws market --asset-ids <TOKEN_IDS> [OPTIONS]
```

| Option                   | Description                                       |
| ------------------------ | ------------------------------------------------- |
| `-a, --asset-ids <IDS>`  | Asset IDs (token IDs), comma-separated (required) |
| `-n, --max-messages <N>` | Max messages to receive (default: 10)             |
| `--timeout <SECS>`       | Timeout in seconds (default: 60)                  |
| `-o, --output <FORMAT>`  | `json` or `compact` (default: json)               |

### User Channel

Subscribe to user events (orders, trades) - requires authentication.

```bash
polymarket clob-ws user --market-ids <IDS> [OPTIONS]
```

| Option                   | Description                                            |
| ------------------------ | ------------------------------------------------------ |
| `-m, --market-ids <IDS>` | Market IDs (condition IDs), comma-separated (required) |
| `--api-key <KEY>`        | API key (or `POLY_API_KEY` env)                        |
| `--api-secret <SECRET>`  | API secret (or `POLY_API_SECRET` env)                  |
| `--passphrase <PASS>`    | Passphrase (or `POLY_PASSPHRASE` env)                  |
| `-n, --max-messages <N>` | Max messages to receive (default: 10)                  |
| `--timeout <SECS>`       | Timeout in seconds (default: 60)                       |
| `-o, --output <FORMAT>`  | `json` or `compact` (default: json)                    |

### Examples

```bash
# Subscribe to market channel for orderbook updates
polymarket clob-ws market -a "71321045679252212594626385532706912750332728571942532289631379312455583992563" -n 5

# Subscribe to user channel (auth from env)
export POLY_API_KEY="your-key"
export POLY_API_SECRET="your-secret"
export POLY_PASSPHRASE="your-passphrase"
polymarket clob-ws user -m "0xabc..." -o compact
```

---

## Output

All commands output JSON format.

## Help

```bash
polymarket --help
polymarket data --help
polymarket data get-user-positions --help
```
