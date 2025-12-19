# CLI Commands Examples

All commands verified on 2025-12-19. Binary: `polymarket` (or `cargo run --`).

---

## Data API

### Health Check

```bash
polymarket ds data health
# Output: {"data": "OK"}
```

### User Commands

```bash
# Get user positions
polymarket ds data get-user-positions \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10

# Get user closed positions
polymarket ds data get-user-closed-positions \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10

# Get user portfolio value
polymarket ds data get-user-portfolio-value \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839

# Get user traded markets count
polymarket ds data get-user-traded-markets \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839

# Get user activity
polymarket ds data get-user-activity \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10 -t TRADE

# Get trades
polymarket ds data get-trades \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10
```

### Market Commands

```bash
# Get market top holders
polymarket ds data get-market-top-holders \
  -m 0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917 -l 10

# Get open interest
polymarket ds data get-open-interest \
  -m 0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917

# Get event live volume
polymarket ds data get-event-live-volume -i 17000
```

---

## Gamma API

### Core Listings

```bash
# Get sports
polymarket ds gamma get-sports

# Get teams
polymarket ds gamma get-teams -l 10 --league nfl

# Get tags
polymarket ds gamma get-tags -l 10

# Get series
polymarket ds gamma get-series -l 10

# Get events
polymarket ds gamma get-events -l 10 --closed false

# Get markets
polymarket ds gamma get-markets -l 10 --closed false
```

### Single-Entity Lookups

```bash
# Tag by ID
polymarket ds gamma get-tag-by-id 2

# Tag by slug
polymarket ds gamma get-tag-by-slug politics

# Tag relationships
polymarket ds gamma get-tag-relationships-by-tag 2

# Related tags
polymarket ds gamma get-tags-related-to-tag 2

# Event by ID
polymarket ds gamma get-event-by-id 17000

# Event by slug
polymarket ds gamma get-event-by-slug trump-cryptocurrency-executive-order-in-first-week

# Event tags
polymarket ds gamma get-event-tags 17000

# Market by ID
polymarket ds gamma get-market-by-id 516861

# Market by slug
polymarket ds gamma get-market-by-slug will-bitcoin-reach-1000000-by-december-31-2025

# Market tags
polymarket ds gamma get-market-tags 516861

# Series by ID
polymarket ds gamma get-series-by-id 1
```

### Comments

```bash
# Comments by parent entity
polymarket ds gamma get-comments --parent-entity-type Event --parent-entity-id 17000 -l 10

# Comment by ID
polymarket ds gamma get-comment-by-id 1

# Comments by user
polymarket ds gamma get-comments-by-user-address 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10
```

### Search

```bash
# Cross-entity search
polymarket ds gamma search "bitcoin" --limit-per-type 3
```

---

## CLOB API

### Order Book

```bash
# Get order book for single token
polymarket ds clob get-order-book \
  -t 60487116984468020978247225474488676749601001829886755968952521846780452448915

# Get order books for multiple tokens
polymarket ds clob get-order-books \
  -t 60487116984468020978247225474488676749601001829886755968952521846780452448915 \
  -t 81104637750588840860328515305303028259865221573278091453716127842023614249200
```

### Pricing

```bash
# Get market price for token and side
polymarket ds clob get-market-price \
  -t 60487116984468020978247225474488676749601001829886755968952521846780452448915 \
  -s BUY

# Get midpoint price
polymarket ds clob get-midpoint-price \
  -t 60487116984468020978247225474488676749601001829886755968952521846780452448915

# Get price history (1 day interval)
polymarket ds clob get-price-history \
  -m 60487116984468020978247225474488676749601001829886755968952521846780452448915 \
  -i 1d
```

> [!WARNING] > `polymarket ds clob get-market-prices` may return 400 error - this is a known API limitation.

### Spreads

```bash
# Get spreads for tokens
polymarket ds clob get-spreads \
  -t 60487116984468020978247225474488676749601001829886755968952521846780452448915
```

---

## CoinMarketCap API

> [!NOTE]
> Requires `CMC_API_KEY` environment variable. Get a free API key at: <https://coinmarketcap.com/api/>

### Listings

```bash
# Get top 10 cryptocurrencies
polymarket ds cmc get-listings -l 10

# Get listings with filters
polymarket ds cmc get-listings -l 20 --price-min 1 --price-max 1000 --convert EUR

# Filter by type and tag
polymarket ds cmc get-listings -l 10 --cryptocurrency-type tokens --tag defi
```

### Market Metrics

```bash
# Get global market metrics (total market cap, BTC dominance, etc.)
polymarket ds cmc get-global-metrics

# With specific currency
polymarket ds cmc get-global-metrics --convert EUR

# Get Fear and Greed Index
polymarket ds cmc get-fear-and-greed

# Check API key usage
polymarket ds cmc get-key-info
```

### Cryptocurrency Map & Info

```bash
# Get cryptocurrency ID map (first 10)
polymarket ds cmc get-map -l 10

# Filter by symbols
polymarket ds cmc get-map -s BTC,ETH,SOL

# Get cryptocurrency metadata (logo, description, URLs)
polymarket ds cmc get-info -s BTC

# Get info for multiple coins
polymarket ds cmc get-info --symbol BTC,ETH --skip-invalid
```

### Quotes

```bash
# Get latest quotes for specific coins
polymarket ds cmc get-quotes -s BTC,ETH

# With EUR conversion
polymarket ds cmc get-quotes -s BTC --convert EUR
```

### Fiat Map & Price Conversion

```bash
# Get fiat currency ID map
polymarket ds cmc get-fiat-map -l 10

# Include precious metals (gold, silver)
polymarket ds cmc get-fiat-map --include-metals

# Convert 1 BTC to USD
polymarket ds cmc price-convert -a 1 -s BTC -c USD

# Convert to multiple currencies
polymarket ds cmc price-convert -a 100 -s ETH -c USD,EUR,GBP
```

---

## CoinGecko API

> [!NOTE]
> Requires `CG_API_KEY` environment variable. Get a free API key at: <https://www.coingecko.com/en/api>

### Simple Price

```bash
# Get Bitcoin price in USD
polymarket ds cg simple-price --ids bitcoin --vs-currencies usd

# Get multiple coins with additional data
polymarket ds cg simple-price --ids bitcoin,ethereum --vs-currencies usd,eur \
  --include-market-cap --include-24hr-change
```

### Coins List

```bash
# List all supported coins (limited output)
polymarket ds cg coins-list --limit 10

# Include platform addresses
polymarket ds cg coins-list --include-platform --limit 5
```

### Coins Markets

```bash
# Get top 10 coins by market cap
polymarket ds cg coins-markets --vs-currency usd --per-page 10

# Filter by specific coins with price change data
polymarket ds cg coins-markets --vs-currency usd --ids bitcoin,ethereum \
  --price-change-percentage 1h,24h,7d
```

### Trending

```bash
# Get trending coins, NFTs, and categories
polymarket ds cg trending
```

### Global

```bash
# Get global cryptocurrency market stats
polymarket ds cg global
```

### Exchanges

```bash
# Get list of exchanges (first 5)
polymarket ds cg exchanges --per-page 5

# Get exchanges with pagination
polymarket ds cg exchanges --per-page 10 --page 2
```

### Supported VS Currencies

```bash
# List all supported vs currencies
polymarket ds cg supported-vs-currencies
```

### Coin Detail

```bash
# Get detailed coin data
polymarket ds cg coin --id bitcoin

# Get coin data without localization
polymarket ds cg coin --id ethereum --no-localization
```

### Market Chart

```bash
# Get 1-day historical market chart
polymarket ds cg market-chart --id bitcoin --vs-currency usd --days 1

# Get 7-day historical data
polymarket ds cg market-chart --id ethereum --vs-currency eur --days 7
```

### History

```bash
# Get historical data at specific date (dd-mm-yyyy format)
polymarket ds cg history --id bitcoin --date 30-12-2024

# Without localization
polymarket ds cg history --id ethereum --date 01-01-2024 --no-localization
```

### OHLC

```bash
# Get 1-day OHLC candlestick data
polymarket ds cg ohlc --id bitcoin --vs-currency usd --days 1

# Get 7-day OHLC data
polymarket ds cg ohlc --id ethereum --vs-currency eur --days 7
```

---

## Alternative.me API

> [!NOTE]
> Free API - no API key required.

### Ticker

```bash
# Get top 10 cryptocurrencies by market cap
polymarket ds alt get-ticker --limit 10

# Get ticker sorted by 24h volume
polymarket ds alt get-ticker --limit 20 --sort volume_24h

# Get specific cryptocurrency (by ID or slug)
polymarket ds alt get-ticker-by-id bitcoin
polymarket ds alt get-ticker-by-id 1

# Get ticker with EUR conversion
polymarket ds alt get-ticker-by-id bitcoin --convert EUR
```

### Market Metrics

```bash
# Get global market metrics (total market cap, BTC dominance)
polymarket ds alt get-global

# With EUR conversion
polymarket ds alt get-global --convert EUR
```

### Fear and Greed Index

```bash
# Get latest Fear and Greed Index
polymarket ds alt get-fear-and-greed

# Get historical data (last 7 days)
polymarket ds alt get-fear-and-greed --limit 7

# Get historical data with date format
polymarket ds alt get-fear-and-greed --limit 30 --date-format us
```

---

## CLOB WebSocket

### Market Channel

```bash
# Subscribe to order book updates for a token
polymarket ds clob-ws market \
  -a 60487116984468020978247225474488676749601001829886755968952521846780452448915 \
  -n 5 --timeout 30

# Subscribe to multiple tokens with compact output
polymarket ds clob-ws market \
  -a 60487116984468020978247225474488676749601001829886755968952521846780452448915,81104637750588840860328515305303028259865221573278091453716127842023614249200 \
  -o compact
```

### User Channel (Requires Auth)

```bash
# Subscribe to user order/trade updates
# Requires POLY_API_KEY, POLY_API_SECRET, POLY_PASSPHRASE env vars
polymarket ds clob-ws user \
  -m 0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917 \
  -n 10 --timeout 60
```

---

## RTDS (Real-Time Data Service)

### Subscribe to Topics

```bash
# Subscribe to crypto prices
polymarket ds rtds subscribe -t crypto_prices -n 5

# Subscribe with filter
polymarket ds rtds subscribe -t crypto_prices -n 10 \
  --filter '{"symbol":"BTCUSDT"}'

# Subscribe to activity stream
polymarket ds rtds subscribe -t activity -T "*" -n 20 --timeout 120

# Subscribe with compact output
polymarket ds rtds subscribe -t comments -n 5 -o compact
```

> [!TIP]
> Available topics: `activity`, `comments`, `rfq`, `crypto_prices`,
> `crypto_prices_chainlink`, `equity_prices`, `clob_user`, `clob_market`

---

## Quick Reference

| API        | Command                 | Required Args             |
| ---------- | ----------------------- | ------------------------- |
| ds data    | health                  | -                         |
| ds data    | get-user-positions      | `-u <ADDRESS>`            |
| ds data    | get-trades              | (optional filters)        |
| ds data    | get-open-interest       | `-m <MARKET_ID>`          |
| ds gamma   | get-sports              | -                         |
| ds gamma   | get-events              | (optional filters)        |
| ds gamma   | get-markets             | (optional filters)        |
| ds gamma   | get-event-by-id         | `<EVENT_ID>`              |
| ds gamma   | get-market-by-id        | `<MARKET_ID>`             |
| ds gamma   | search                  | `"<QUERY>"`               |
| ds clob    | get-order-book          | `-t <TOKEN_ID>`           |
| ds clob    | get-market-price        | `-t <TOKEN_ID> -s <SIDE>` |
| ds clob    | get-midpoint-price      | `-t <TOKEN_ID>`           |
| ds clob    | get-price-history       | `-m <TOKEN_ID>`           |
| ds clob-ws | market                  | `-a <ASSET_IDS>`          |
| ds clob-ws | user                    | `-m <MARKET_IDS>` + auth  |
| ds cmc     | get-listings            | (optional filters)        |
| ds cmc     | get-global-metrics      | (optional)                |
| ds cmc     | get-fear-and-greed      | -                         |
| ds cmc     | get-key-info            | -                         |
| ds cmc     | get-map                 | (optional filters)        |
| ds cmc     | get-info                | `-s <SYMBOL>` or `--id`   |
| ds cmc     | get-quotes              | `-s <SYMBOL>` or `--id`   |
| ds cmc     | get-fiat-map            | (optional filters)        |
| ds cmc     | price-convert           | `-a <AMOUNT> -s <SYMBOL>` |
| ds cg      | simple-price            | `--ids <IDS>`             |
| ds cg      | supported-vs-currencies | -                         |
| ds cg      | coins-list              | (optional)                |
| ds cg      | coins-markets           | `--vs-currency <CUR>`     |
| ds cg      | exchanges               | (optional)                |
| ds cg      | coin                    | `--id <ID>`               |
| ds cg      | market-chart            | `--id <ID> --vs-currency` |
| ds cg      | history                 | `--id <ID> --date <DATE>` |
| ds cg      | ohlc                    | `--id <ID> --vs-currency` |
| ds cg      | trending                | -                         |
| ds cg      | global                  | -                         |
| ds alt     | get-ticker              | (optional filters)        |
| ds alt     | get-ticker-by-id        | `<ID>` or `<SLUG>`        |
| ds alt     | get-global              | (optional)                |
| ds alt     | get-fear-and-greed      | (optional)                |
| ds rtds    | subscribe               | `-t <TOPIC>`              |
