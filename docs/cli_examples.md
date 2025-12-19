# CLI Commands Examples

All commands verified on 2025-12-19. Binary: `polymarket` (or `cargo run --`).

---

## Data API

### Health Check

```bash
polymarket data health
# Output: {"data": "OK"}
```

### User Commands

```bash
# Get user positions
polymarket data get-user-positions \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10

# Get user closed positions
polymarket data get-user-closed-positions \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10

# Get user portfolio value
polymarket data get-user-portfolio-value \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839

# Get user traded markets count
polymarket data get-user-traded-markets \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839

# Get user activity
polymarket data get-user-activity \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10 -t TRADE

# Get trades
polymarket data get-trades \
  -u 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10
```

### Market Commands

```bash
# Get market top holders
polymarket data get-market-top-holders \
  -m 0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917 -l 10

# Get open interest
polymarket data get-open-interest \
  -m 0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917

# Get event live volume
polymarket data get-event-live-volume -i 17000
```

---

## Gamma API

### Core Listings

```bash
# Get sports
polymarket gamma get-sports

# Get teams
polymarket gamma get-teams -l 10 --league nfl

# Get tags
polymarket gamma get-tags -l 10

# Get series
polymarket gamma get-series -l 10

# Get events
polymarket gamma get-events -l 10 --closed false

# Get markets
polymarket gamma get-markets -l 10 --closed false
```

### Single-Entity Lookups

```bash
# Tag by ID
polymarket gamma get-tag-by-id 2

# Tag by slug
polymarket gamma get-tag-by-slug politics

# Tag relationships
polymarket gamma get-tag-relationships-by-tag 2

# Related tags
polymarket gamma get-tags-related-to-tag 2

# Event by ID
polymarket gamma get-event-by-id 17000

# Event by slug
polymarket gamma get-event-by-slug trump-cryptocurrency-executive-order-in-first-week

# Event tags
polymarket gamma get-event-tags 17000

# Market by ID
polymarket gamma get-market-by-id 516861

# Market by slug
polymarket gamma get-market-by-slug will-bitcoin-reach-1000000-by-december-31-2025

# Market tags
polymarket gamma get-market-tags 516861

# Series by ID
polymarket gamma get-series-by-id 1
```

### Comments

```bash
# Comments by parent entity
polymarket gamma get-comments --parent-entity-type Event --parent-entity-id 17000 -l 10

# Comment by ID
polymarket gamma get-comment-by-id 1

# Comments by user
polymarket gamma get-comments-by-user-address 0x56687bf447db6ffa42ffe2204a05edaa20f55839 -l 10
```

### Search

```bash
# Cross-entity search
polymarket gamma search "bitcoin" --limit-per-type 3
```

---

## CLOB API

### Order Book

```bash
# Get order book for single token
polymarket clob get-order-book \
  -t 60487116984468020978247225474488676749601001829886755968952521846780452448915

# Get order books for multiple tokens
polymarket clob get-order-books \
  -t 60487116984468020978247225474488676749601001829886755968952521846780452448915 \
  -t 81104637750588840860328515305303028259865221573278091453716127842023614249200
```

### Pricing

```bash
# Get market price for token and side
polymarket clob get-market-price \
  -t 60487116984468020978247225474488676749601001829886755968952521846780452448915 \
  -s BUY

# Get midpoint price
polymarket clob get-midpoint-price \
  -t 60487116984468020978247225474488676749601001829886755968952521846780452448915

# Get price history (1 day interval)
polymarket clob get-price-history \
  -m 60487116984468020978247225474488676749601001829886755968952521846780452448915 \
  -i 1d
```

> [!WARNING] > `polymarket clob get-market-prices` may return 400 error - this is a known API limitation.

### Spreads

```bash
# Get spreads for tokens
polymarket clob get-spreads \
  -t 60487116984468020978247225474488676749601001829886755968952521846780452448915
```

---

## CoinMarketCap API

> [!NOTE]
> Requires `CMC_API_KEY` environment variable. Get a free API key at: <https://coinmarketcap.com/api/>

### Listings

```bash
# Get top 10 cryptocurrencies
polymarket cmc get-listings -l 10

# Get listings with filters
polymarket cmc get-listings -l 20 --price-min 1 --price-max 1000 --convert EUR

# Filter by type and tag
polymarket cmc get-listings -l 10 --cryptocurrency-type tokens --tag defi
```

### Market Metrics

```bash
# Get global market metrics (total market cap, BTC dominance, etc.)
polymarket cmc get-global-metrics

# With specific currency
polymarket cmc get-global-metrics --convert EUR

# Get Fear and Greed Index
polymarket cmc get-fear-and-greed

# Check API key usage
polymarket cmc get-key-info
```

---

## CLOB WebSocket

### Market Channel

```bash
# Subscribe to order book updates for a token
polymarket clob-ws market \
  -a 60487116984468020978247225474488676749601001829886755968952521846780452448915 \
  -n 5 --timeout 30

# Subscribe to multiple tokens with compact output
polymarket clob-ws market \
  -a 60487116984468020978247225474488676749601001829886755968952521846780452448915,81104637750588840860328515305303028259865221573278091453716127842023614249200 \
  -o compact
```

### User Channel (Requires Auth)

```bash
# Subscribe to user order/trade updates
# Requires POLY_API_KEY, POLY_API_SECRET, POLY_PASSPHRASE env vars
polymarket clob-ws user \
  -m 0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917 \
  -n 10 --timeout 60
```

---

## RTDS (Real-Time Data Service)

### Subscribe to Topics

```bash
# Subscribe to crypto prices
polymarket rtds subscribe -t crypto_prices -n 5

# Subscribe with filter
polymarket rtds subscribe -t crypto_prices -n 10 \
  --filter '{"symbol":"BTCUSDT"}'

# Subscribe to activity stream
polymarket rtds subscribe -t activity -T "*" -n 20 --timeout 120

# Subscribe with compact output
polymarket rtds subscribe -t comments -n 5 -o compact
```

> [!TIP]
> Available topics: `activity`, `comments`, `rfq`, `crypto_prices`,
> `crypto_prices_chainlink`, `equity_prices`, `clob_user`, `clob_market`

---

## Quick Reference

| API     | Command            | Required Args             |
| ------- | ------------------ | ------------------------- |
| Data    | health             | -                         |
| Data    | get-user-positions | `-u <ADDRESS>`            |
| Data    | get-trades         | (optional filters)        |
| Data    | get-open-interest  | `-m <MARKET_ID>`          |
| Gamma   | get-sports         | -                         |
| Gamma   | get-events         | (optional filters)        |
| Gamma   | get-markets        | (optional filters)        |
| Gamma   | get-event-by-id    | `<EVENT_ID>`              |
| Gamma   | get-market-by-id   | `<MARKET_ID>`             |
| Gamma   | search             | `"<QUERY>"`               |
| CLOB    | get-order-book     | `-t <TOKEN_ID>`           |
| CLOB    | get-market-price   | `-t <TOKEN_ID> -s <SIDE>` |
| CLOB    | get-midpoint-price | `-t <TOKEN_ID>`           |
| CLOB    | get-price-history  | `-m <TOKEN_ID>`           |
| CLOB WS | market             | `-a <ASSET_IDS>`          |
| CLOB WS | user               | `-m <MARKET_IDS>` + auth  |
| CMC     | get-listings       | (optional filters)        |
| CMC     | get-global-metrics | (optional)                |
| CMC     | get-fear-and-greed | -                         |
| CMC     | get-key-info       | -                         |
| RTDS    | subscribe          | `-t <TOPIC>`              |
