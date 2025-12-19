# Policy Engine

This document describes the Policy Engine for the polymarket-hft trading system — a unified, user-defined rules engine with a configuration-driven approach.

## Overview

The Policy Engine enables users to define **micro-policies** — small, reusable rules that trigger actions when conditions are met. Policies are defined in YAML/JSON configuration files and can be created via Web UI in the future.

**Key Features:**

- **Declarative DSL** — Define policies without writing code
- **Composite Conditions** — AND/OR logic with time-window support
- **Multiple Action Types** — Notifications, orders, and webhooks
- **Rate Limiting** — Built-in cooldown to prevent alert fatigue

---

## Policy Structure

```yaml
policies:
  - id: btc_low_alert # Unique identifier
    name: "BTC Below $80K" # Human-readable name
    enabled: true # Toggle on/off
    priority: 100 # Higher = evaluated first
    cooldown: 1h # Minimum interval between triggers
    conditions: ... # When to trigger
    actions: ... # What to do
```

---

## Conditions

### Simple Condition

```yaml
conditions:
  field: price # Event field: price, volume, bid, ask
  asset: "BTC" # Filter by asset (optional)
  market: "0x..." # Filter by market ID (optional)
  operator: "<" # <, <=, >, >=, ==, !=, contains, matches
  value: 80000
```

### Composite Conditions

**AND (all must be true):**

```yaml
conditions:
  all:
    - field: price
      asset: "BTC"
      operator: "<"
      value: 80000
    - field: volume_24h
      operator: ">"
      value: 1000000
```

**OR (any must be true):**

```yaml
conditions:
  any:
    - field: price
      operator: "<"
      value: 80000
    - field: price
      operator: ">"
      value: 120000
```

### Time Window Conditions

Evaluate conditions over a rolling time window:

```yaml
conditions:
  field: price
  asset: "BTC"
  window:
    duration: 5m # Window size: 1m, 5m, 1h, etc.
    aggregation: change_pct # See aggregation types below
  operator: "<"
  value: -5 # Price dropped >5% in 5 minutes
```

**Aggregation Types:**

| Type         | Description                                       |
| ------------ | ------------------------------------------------- |
| `first`      | First value in window                             |
| `last`       | Last value in window                              |
| `min`        | Minimum value                                     |
| `max`        | Maximum value                                     |
| `avg`        | Average value                                     |
| `sum`        | Sum of values                                     |
| `change`     | Absolute change (last - first)                    |
| `change_pct` | Percentage change ((last - first) / first \* 100) |

### Threshold Crossing (Edge Trigger)

Only trigger when the value **crosses** a threshold, not while it remains above/below:

```yaml
conditions:
  field: price
  asset: "BTC"
  operator: crosses_below # crosses_above, crosses_below
  value: 80000
```

---

## Actions

### Notification

Send alerts via configured channels:

```yaml
actions:
  - type: notification
    channel: telegram # telegram, email, webhook
    template: |
      🚨 **Alert: {{ policy.name }}**
      Asset: {{ event.asset }}
      Price: ${{ event.price }}
      Time: {{ now | date }}
```

### Order

Place or cancel orders:

```yaml
actions:
  - type: order
    operation: place # place, cancel, cancel_all
    order:
      market: "{{ event.market }}"
      side: buy # buy, sell
      type: limit # limit, market
      price: "{{ event.price * 0.99 }}"
      size: 100
```

### Webhook (External API)

Call external APIs:

```yaml
actions:
  - type: webhook
    method: POST
    url: "https://api.example.com/hooks/trading"
    headers:
      Authorization: "Bearer {{ env.API_TOKEN }}"
    body:
      event_type: price_alert
      asset: "{{ event.asset }}"
      price: "{{ event.price }}"
```

### Multiple Actions

Chain multiple actions in sequence:

```yaml
actions:
  - type: notification
    channel: telegram
    template: "Executing buy order..."
  - type: order
    operation: place
    order: ...
```

---

## Template Variables

Templates use `{{ }}` syntax for variable substitution:

| Variable       | Description                    |
| -------------- | ------------------------------ |
| `event.asset`  | Asset ID from triggering event |
| `event.market` | Market ID                      |
| `event.price`  | Current price                  |
| `event.bid`    | Current bid price              |
| `event.ask`    | Current ask price              |
| `policy.id`    | Policy ID                      |
| `policy.name`  | Policy name                    |
| `env.VAR_NAME` | Environment variable           |
| `now`          | Current timestamp              |

---

## Example Policies

### 1. Simple Price Alert

```yaml
policies:
  - id: btc_low_alert
    name: "BTC Below $80K"
    enabled: true
    cooldown: 1h
    conditions:
      field: price
      asset: "BTC"
      operator: crosses_below
      value: 80000
    actions:
      - type: notification
        channel: telegram
        template: "🔴 BTC dropped below $80,000! Current: ${{ event.price }}"
```

### 2. Volatility Detection

```yaml
policies:
  - id: btc_volatility
    name: "BTC High Volatility"
    enabled: true
    cooldown: 30m
    conditions:
      all:
        - field: price
          asset: "BTC"
          window:
            duration: 5m
            aggregation: change_pct
          operator: "<"
          value: -3
        - field: volume_24h
          operator: ">"
          value: 500000
    actions:
      - type: notification
        channel: telegram
        template: "⚠️ BTC volatility spike! Dropped 3%+ in 5 minutes"
```

### 3. Auto-Buy on Dip

```yaml
policies:
  - id: btc_auto_buy
    name: "Auto Buy BTC Dip"
    enabled: true
    cooldown: 24h
    conditions:
      field: price
      asset: "BTC"
      operator: crosses_below
      value: 75000
    actions:
      - type: notification
        channel: telegram
        template: "🟢 Executing BTC dip buy at ${{ event.price }}"
      - type: order
        operation: place
        order:
          market: "{{ env.BTC_MARKET_ID }}"
          side: buy
          type: market
          size: 100
```

---

## Configuration Loading

Policies are loaded from YAML/JSON files at startup:

```bash
# Single file
polymarket-hft --policies policies.yaml

# Directory (all *.yaml files)
polymarket-hft --policies-dir ./policies/
```

Future: Web UI for creating and managing policies via JSON/YAML import.
