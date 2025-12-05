# CLI Usage Guide

This guide covers how to use the `polymarket-hft` command-line interface.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/telepair/polymarket-hft.git
cd polymarket-hft

# Build and install
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

## Commands

### Data API

#### Health Check

Check the API health status.

```bash
cargo run -- data health
```

---

### User Commands

#### Get User Positions

Get current positions for a user.

```bash
cargo run -- data get-user-positions -u <USER_ADDRESS> [OPTIONS]
```

**Options:**

- `-u, --user <ADDRESS>` - User Profile Address (required)
- `-m, --market <MARKET_ID>` - Market condition IDs to filter by (can be specified multiple times)
- `-e, --event-id <EVENT_ID>` - Event IDs to filter by (can be specified multiple times)
- `--size-threshold <SIZE>` - Minimum position size (>= 0)
- `--redeemable <BOOL>` - Filter for redeemable positions
- `--mergeable <BOOL>` - Filter for mergeable positions
- `-l, --limit <LIMIT>` - Limit results (0-500, default: 100)
- `-o, --offset <OFFSET>` - Offset for pagination (0-10000, default: 0)
- `--sort-by <FIELD>` - Sort field (CURRENT, INITIAL, TOKENS, CASHPNL, PERCENTPNL, TITLE, RESOLVING, PRICE, AVGPRICE)
- `--sort-direction <DIR>` - Sort direction (ASC or DESC)
- `-t, --title <TITLE>` - Title filter (max 160 chars)

**Example:**

```bash
cargo run -- data get-user-positions -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10
```

#### Get User Closed Positions

Get closed positions for a user.

```bash
cargo run -- data get-user-closed-positions -u <USER_ADDRESS> [OPTIONS]
```

**Options:**

- `-u, --user <ADDRESS>` - User Profile Address (required)
- `-m, --market <MARKET_ID>` - Market condition IDs to filter by (can be specified multiple times)
- `-t, --title <TITLE>` - Title filter (max 100 chars)
- `-e, --event-id <EVENT_ID>` - Event IDs to filter by (can be specified multiple times)
- `-l, --limit <LIMIT>` - Limit results (0-50, default: 10)
- `-o, --offset <OFFSET>` - Offset for pagination (0-100000, default: 0)
- `--sort-by <FIELD>` - Sort field (REALIZEDPNL, TITLE, PRICE, AVGPRICE, TIMESTAMP)
- `--sort-direction <DIR>` - Sort direction (ASC or DESC)

**Example:**

```bash
cargo run -- data get-user-closed-positions -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10
```

#### Get User Portfolio Value

Get the total value of a user's positions.

```bash
cargo run -- data get-user-portfolio-value -u <USER_ADDRESS> [-m <MARKET_ID>...]
```

**Options:**

- `-u, --user <ADDRESS>` - Ethereum address of the user (required)
- `-m, --market <MARKET_ID>` - Market ID to filter by (optional, can be specified multiple times)

**Example:**

```bash
# Get total value across all markets
cargo run -- data get-user-portfolio-value -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839

# Get value for specific markets
cargo run -- data get-user-portfolio-value -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -m 0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917
```

#### Get User Traded Markets

Get the number of markets a user has traded.

```bash
cargo run -- data get-user-traded-markets -u <USER_ADDRESS>
```

**Options:**

- `-u, --user <ADDRESS>` - Ethereum address of the user (required)

**Example:**

```bash
cargo run -- data get-user-traded-markets -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839
```

#### Get User Activity

Get on-chain activity for a user.

```bash
cargo run -- data get-user-activity -u <USER_ADDRESS> [OPTIONS]
```

**Options:**

- `-u, --user <ADDRESS>` - User Profile Address (required)
- `-l, --limit <LIMIT>` - Limit results (0-500, default: 100)
- `-o, --offset <OFFSET>` - Offset for pagination (0-10000, default: 0)
- `-m, --market <MARKET_ID>` - Market condition IDs to filter by (mutually exclusive with event-id)
- `-e, --event-id <EVENT_ID>` - Event IDs to filter by (mutually exclusive with market)
- `-t, --type <TYPE>` - Activity types to filter by (TRADE, SPLIT, MERGE, REDEEM, REWARD, CONVERSION)
- `--start <TIMESTAMP>` - Start timestamp (>= 0)
- `--end <TIMESTAMP>` - End timestamp (>= 0)
- `--sort-by <FIELD>` - Sort field (TIMESTAMP, TOKENS, CASH)
- `--sort-direction <DIR>` - Sort direction (ASC or DESC)
- `--side <SIDE>` - Trade side filter (BUY or SELL)

**Example:**

```bash
cargo run -- data get-user-activity -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 50 -t TRADE
```

#### Get Trades

Get trades for a user or markets.

```bash
cargo run -- data get-trades [OPTIONS]
```

**Options:**

- `-u, --user <ADDRESS>` - User Profile Address (optional)
- `-m, --market <MARKET_ID>` - Market condition IDs to filter by (mutually exclusive with event-id)
- `-e, --event-id <EVENT_ID>` - Event IDs to filter by (mutually exclusive with market)
- `-l, --limit <LIMIT>` - Limit results (0-10000, default: 100)
- `-o, --offset <OFFSET>` - Offset for pagination (0-10000, default: 0)
- `--taker-only <BOOL>` - Filter for taker-only trades
- `--filter-type <TYPE>` - Filter type (CASH or TOKENS, must be provided with filter-amount)
- `--filter-amount <AMOUNT>` - Filter amount (>= 0, must be provided with filter-type)
- `-s, --side <SIDE>` - Trade side filter (BUY or SELL)

**Default:** `taker-only` defaults to `true` when omitted. Set `--taker-only false` to include maker trades.

**Example:**

```bash
# Get trades for a user
cargo run -- data get-trades -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 50

# Get trades for a market
cargo run -- data get-trades -m 0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917
```

---

### Market Commands

#### Get Market Top Holders

Get the top holders for markets.

```bash
cargo run -- data get-market-top-holders -m <MARKET_ID> [-l <LIMIT>] [--min-balance <MIN_BALANCE>]
```

**Options:**

- `-m, --market <MARKET_ID>` - Market ID (required, can be specified multiple times)
- `-l, --limit <LIMIT>` - Limit results (0-500, default: 100)
- `--min-balance <MIN_BALANCE>` - Minimum balance (0-999999, default: 1)

**Example:**

```bash
cargo run -- data get-market-top-holders -m 0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917 -l 100 --min-balance 100
```

#### Get Open Interest

Get open interest for one or more markets.

```bash
cargo run -- data get-open-interest -m <MARKET_ID>
```

**Options:**

- `-m, --market <MARKET_ID>` - Market ID (required, can be specified multiple times)

**Example:**

```bash
cargo run -- data get-open-interest -m 0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917
```

#### Get Event Live Volume

Get live volume for an event.

```bash
cargo run -- data get-event-live-volume -i <EVENT_ID>
```

**Options:**

- `-i, --id <EVENT_ID>` - Event ID (required, must be >= 1)

**Example:**

```bash
cargo run -- data get-event-live-volume -i 123
```

---

### Gamma Markets API

Use the `gamma` subcommand for discovery/search metadata.

#### Core listing

- Sports: `cargo run -- gamma get-sports`
- Teams: `cargo run -- gamma get-teams [-l <LIMIT>] [-o <OFFSET>] [--league <LEAGUE>] [--name <NAME>] [--abbreviation <ABBR>]`
- Tags: `cargo run -- gamma get-tags [-l <LIMIT>] [-o <OFFSET>] [--include-template <BOOL>] [--is-carousel <BOOL>]`
- Series: `cargo run -- gamma get-series [-l <LIMIT>] [-o <OFFSET>] [--slug <SLUG>] [--closed <BOOL>] [--recurrence <STR>]`
- Events: `cargo run -- gamma get-events [-l <LIMIT>] [-o <OFFSET>] [--tag-id <ID>] [--exclude-tag-id <ID>] [--active <BOOL>] [--closed <BOOL>] [--related-tags <BOOL>] [--order <FIELD>] [--ascending <BOOL>]`
- Markets: `cargo run -- gamma get-markets [-l <LIMIT>] [-o <OFFSET>] [--id <ID>...] [--slug <SLUG>...] [--tag-id <ID>] [--event-id <EVENT_ID>] [--related-tags <BOOL>] [--closed <BOOL>] [--include-tag <BOOL>]`

#### Single-entity lookups

- Tag by ID: `cargo run -- gamma get-tag-by-id <TAG_ID>`
- Tag by slug: `cargo run -- gamma get-tag-by-slug <SLUG> [--include-template <BOOL>]`
- Tag relationships: `cargo run -- gamma get-tag-relationships-by-tag <TAG_ID> [--status <STATUS>] [--omit-empty <BOOL>]`
- Related tags by tag: `cargo run -- gamma get-tags-related-to-tag <TAG_ID> [--status <STATUS>] [--omit-empty <BOOL>]`
- Event by ID/slug: `cargo run -- gamma get-event-by-id <ID> [--include-chat <BOOL>] [--include-template <BOOL>]`  
  `cargo run -- gamma get-event-by-slug <SLUG> [--include-chat <BOOL>] [--include-template <BOOL>]`
- Event tags: `cargo run -- gamma get-event-tags <EVENT_ID>`
- Market by ID/slug: `cargo run -- gamma get-market-by-id <ID> [--include-tag <BOOL>]`  
  `cargo run -- gamma get-market-by-slug <SLUG> [--include-tag <BOOL>]`
- Market tags: `cargo run -- gamma get-market-tags <MARKET_ID>`
- Series by ID: `cargo run -- gamma get-series-by-id <SERIES_ID> [--include-chat <BOOL>]`

#### Comments & profiles

- Comments by parent: `cargo run -- gamma get-comments --parent-entity-type Event --parent-entity-id <ID> [-l <LIMIT>] [-o <OFFSET>] [--get-positions <BOOL>] [--holders-only <BOOL>]`
- Comment by ID: `cargo run -- gamma get-comment-by-id <ID> [--get-positions <BOOL>]`
- Comments by user: `cargo run -- gamma get-comments-by-user-address <ADDRESS> [-l <LIMIT>] [-o <OFFSET>] [--order <ORDER>] [--ascending <BOOL>]`

#### Search

- Cross-entity search: `cargo run -- gamma search "<QUERY>" [--cache <BOOL>] [--events-status <STATUS>] [--events-tag <TAG>...] [--limit-per-type <N>] [--page <N>] [--optimized <BOOL>]`

#### Output

All commands emit JSON for downstream tooling.

## Output Format

All commands output JSON format for easy parsing and integration with other tools.

## Help

Get help for any command:

```bash
# General help
cargo run -- --help

# Data API help
cargo run -- data --help

# Specific command help
cargo run -- data get-user-positions --help
```
