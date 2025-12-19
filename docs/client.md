# Client Documentation

This guide covers how to use the various API clients provided by `polymarket-hft`.

## Overview

The library provides clients for the following services:

| Service              | Module                      | Protocol         | Description                                            |
| -------------------- | --------------------------- | ---------------- | ------------------------------------------------------ |
| **Polymarket Data**  | `client::polymarket::data`  | REST             | User data, positions, trades, portfolio value.         |
| **Polymarket Gamma** | `client::polymarket::gamma` | REST             | Market discovery, events, tags, comments.              |
| **Polymarket CLOB**  | `client::polymarket::clob`  | REST / WebSocket | Order book, trading, price history.                    |
| **Polymarket RTDS**  | `client::polymarket::rtds`  | WebSocket        | Real-time data streaming (prices, activity).           |
| **CoinMarketCap**    | `client::coinmarketcap`     | REST             | Cryptocurrency listings, global metrics, fear & greed. |

## Common Features

### HTTP Client & Retries

All REST clients share a common HTTP infrastructure that provides:

- **Automatic Retries**: Exponential backoff for transient failures (timeouts, 5xx errors).
- **connection Pooling**: Efficient connection reuse.
- **Timeouts**: configurable request and connection timeouts.

You can customize the HTTP behavior when creating a client:

```rust
use polymarket_hft::client::http::HttpClientConfig;
use std::time::Duration;

let config = HttpClientConfig::default()
    .with_max_retries(5)
    .with_timeout(Duration::from_secs(60));
```

## CoinMarketCap Client

The CoinMarketCap client provides access to the Standard API using the **Basic Plan** (free tier).

> [!IMPORTANT] > **API Key Required**: Register at [CoinMarketCap Developer Portal](https://coinmarketcap.com/api/) to get a free API key. The Basic Plan includes:
>
> - **10,000 credits/month** (resets at UTC midnight on the 1st)
> - **333 credits/day** (resets at UTC midnight)
> - **30 requests/minute** rate limit

### Quick Start

```rust
use polymarket_hft::client::coinmarketcap::{
    Client,
    GetListingsLatestRequest,
    GetGlobalMetricsQuotesLatestRequest,
    GetFearAndGreedLatestRequest
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with your API Key
    let client = Client::new("YOUR_CMC_API_KEY");

    // 1. Get Latest Listings (e.g., Top 5 Cryptocurrencies)
    let listings = client.get_listings_latest(GetListingsLatestRequest {
        limit: Some(5),
        ..Default::default()
    }).await?;

    println!("Top 5 Cryptocurrencies:");
    for crypto in listings.data {
        println!("{}: ${}", crypto.name, crypto.quote["USD"].price);
    }

    // 2. Get Global Market Metrics
    let metrics = client.get_global_metrics_quotes_latest(
        GetGlobalMetricsQuotesLatestRequest::default()
    ).await?;
    println!("BTC Dominance: {}%", metrics.data.btc_dominance);

    // 3. Get Fear and Greed Index
    let fear_greed = client.get_fear_and_greed_latest(
        GetFearAndGreedLatestRequest::default()
    ).await?;
    println!("Current Index: {} ({})",
        fear_greed.data.value,
        fear_greed.data.value_classification
    );

    // 4. Check API Usage
    let key_info = client.get_key_info().await?;
    println!("Credits remaining today: {}",
        key_info.data.usage.current_day.credits_left
    );

    Ok(())
}
```

### Endpoints

| Method                             | Endpoint                             | Credits         | Description                    |
| ---------------------------------- | ------------------------------------ | --------------- | ------------------------------ |
| `get_listings_latest`              | `/v1/cryptocurrency/listings/latest` | 1 per 200 coins | Latest cryptocurrency listings |
| `get_global_metrics_quotes_latest` | `/v1/global-metrics/quotes/latest`   | 1               | Global market metrics          |
| `get_fear_and_greed_latest`        | `/v3/fear-and-greed/latest`          | 1               | Fear and Greed Index           |
| `get_key_info`                     | `/v1/key/info`                       | 0               | API key usage info             |

### Request Parameters

#### `GetListingsLatestRequest`

| Parameter                           | Type             | Description                                             |
| ----------------------------------- | ---------------- | ------------------------------------------------------- |
| `start`                             | `Option<i32>`    | Offset for pagination (1-based)                         |
| `limit`                             | `Option<i32>`    | Number of results (default: 100, max: 5000)             |
| `price_min` / `price_max`           | `Option<f64>`    | Filter by price range                                   |
| `market_cap_min` / `market_cap_max` | `Option<f64>`    | Filter by market cap                                    |
| `volume_24h_min` / `volume_24h_max` | `Option<f64>`    | Filter by 24h volume                                    |
| `convert`                           | `Option<String>` | Currency for quotes (e.g., "USD", "EUR")                |
| `sort`                              | `Option<String>` | Sort field: `market_cap`, `name`, `price`, `volume_24h` |
| `sort_dir`                          | `Option<String>` | Sort direction: `asc` or `desc`                         |
| `cryptocurrency_type`               | `Option<String>` | Filter: `all`, `coins`, `tokens`                        |
| `tag`                               | `Option<String>` | Filter by tag: `defi`, `filesharing`, etc.              |

#### `GetGlobalMetricsQuotesLatestRequest`

| Parameter    | Type             | Description                        |
| ------------ | ---------------- | ---------------------------------- |
| `convert`    | `Option<String>` | Currency for quotes (default: USD) |
| `convert_id` | `Option<String>` | CoinMarketCap ID for conversion    |

### Error Handling

CoinMarketCap returns errors in the `status` object:

```rust
let response = client.get_listings_latest(request).await?;
if response.status.error_code != 0 {
    eprintln!("API Error: {:?}", response.status.error_message);
}
```

Common error codes:

| Code | Description                            |
| ---- | -------------------------------------- |
| 401  | Invalid or missing API key             |
| 402  | Payment required (plan limit exceeded) |
| 429  | Rate limit exceeded                    |
| 500  | Internal server error                  |

---

## Polymarket Data Client

Access user-centric data like positions and portfolio value.

```rust
use polymarket_hft::client::polymarket::data::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Get user portfolio value
    let values = client.get_user_portfolio_value("0xUserAddress...", None).await?;
    println!("Portfolio Value: {:?}", values);

    Ok(())
}
```

## Polymarket Gamma Client

Discover markets and events.

```rust
use polymarket_hft::client::polymarket::gamma::{Client, GetMarketsRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Find active markets
    let markets = client.get_markets(GetMarketsRequest {
        active: Some(true),
        limit: Some(10),
        ..Default::default()
    }).await?;

    Ok(())
}
```

## Polymarket CLOB Client

Interact with the Order Book and execute trades.

> **Note**: Trading requires a private key and API credentials.

```rust
use polymarket_hft::client::polymarket::clob::{Client, TradingClient, Side};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read-only access
    let public_client = Client::new();
    let book = public_client.get_order_book("token_id").await?;

    Ok(())
}
```

## Polymarket RTDS Client

Stream real-time data via WebSocket.

```rust
use polymarket_hft::client::polymarket::rtds::{RtdsClient, Subscription};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RtdsClient::builder().build();
    client.connect().await?;

    // Subscribe to price updates
    client.subscribe(vec![
        Subscription::new("crypto_prices", "update").with_filter(r#"{"symbol":"BTCUSDT"}"#)
    ]).await?;

    while let Some(msg) = client.next_message().await {
        println!("Update: {:?}", msg);
    }

    Ok(())
}
```
