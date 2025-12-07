# CLI Commands Examples

All commands verified on 2025-12-07. Binary: `polymarket` (or `cargo run --`).

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

## Quick Reference

| API   | Command                   | Required Args              |
| ----- | ------------------------- | -------------------------- |
| Data  | health                    | -                          |
| Data  | get-user-positions        | `-u <ADDRESS>`             |
| Data  | get-user-closed-positions | `-u <ADDRESS>`             |
| Data  | get-user-portfolio-value  | `-u <ADDRESS>`             |
| Data  | get-user-traded-markets   | `-u <ADDRESS>`             |
| Data  | get-user-activity         | `-u <ADDRESS>`             |
| Data  | get-trades                | (optional filters)         |
| Data  | get-market-top-holders    | `-m <MARKET_ID>`           |
| Data  | get-open-interest         | `-m <MARKET_ID>`           |
| Data  | get-event-live-volume     | `-i <EVENT_ID>`            |
| Gamma | get-sports                | -                          |
| Gamma | get-teams                 | (optional filters)         |
| Gamma | get-tags                  | (optional filters)         |
| Gamma | get-series                | (optional filters)         |
| Gamma | get-events                | (optional filters)         |
| Gamma | get-markets               | (optional filters)         |
| Gamma | get-tag-by-id             | `<TAG_ID>`                 |
| Gamma | get-tag-by-slug           | `<SLUG>`                   |
| Gamma | get-event-by-id           | `<EVENT_ID>`               |
| Gamma | get-event-by-slug         | `<SLUG>`                   |
| Gamma | get-market-by-id          | `<MARKET_ID>`              |
| Gamma | get-market-by-slug        | `<SLUG>`                   |
| Gamma | search                    | `"<QUERY>"`                |
| CLOB  | get-order-book            | `-t <TOKEN_ID>`            |
| CLOB  | get-order-books           | `-t <TOKEN_ID>` (multiple) |
| CLOB  | get-market-price          | `-t <TOKEN_ID> -s <SIDE>`  |
| CLOB  | get-midpoint-price        | `-t <TOKEN_ID>`            |
| CLOB  | get-price-history         | `-m <TOKEN_ID>`            |
| CLOB  | get-spreads               | `-t <TOKEN_ID>` (multiple) |
