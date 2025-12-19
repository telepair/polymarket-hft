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
| **CoinGecko**        | `client::coingecko`         | REST             | Coin prices, markets, trending, global stats.          |
| **Alternative.me**   | `client::alternativeme`     | REST             | Free crypto API: ticker, global, fear & greed.         |

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
| `get_cryptocurrency_map`           | `/v1/cryptocurrency/map`             | 1               | Cryptocurrency ID mapping      |
| `get_cryptocurrency_info`          | `/v1/cryptocurrency/info`            | 1               | Cryptocurrency metadata        |
| `get_quotes_latest`                | `/v2/cryptocurrency/quotes/latest`   | 1               | Quotes for specific coins      |
| `get_fiat_map`                     | `/v1/fiat/map`                       | 1               | Fiat currency ID mapping       |
| `get_price_conversion`             | `/v1/tools/price-conversion`         | 1               | Currency price conversion      |

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

## CoinGecko Client

The CoinGecko client provides access to the Demo API for cryptocurrency market data.

> [!IMPORTANT] > **API Key Required**: Register at [CoinGecko](https://www.coingecko.com/en/api) to get a free API key. The Demo Plan includes:
>
> - **10,000 calls/month**
> - **30 calls/minute** rate limit

### Quick Start

```rust
use polymarket_hft::client::coingecko::{
    Client,
    SimplePriceRequest,
    CoinsMarketsRequest
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with your API Key
    let client = Client::new("YOUR_CG_API_KEY");

    // 1. Get Simple Price for Bitcoin
    let prices = client.get_simple_price(SimplePriceRequest {
        ids: "bitcoin,ethereum".to_string(),
        vs_currencies: "usd".to_string(),
        include_24hr_change: Some(true),
        ..Default::default()
    }).await?;
    println!("BTC: ${:?}", prices.get("bitcoin"));

    // 2. Get Market Data (Top 10 by Market Cap)
    let markets = client.get_coins_markets(CoinsMarketsRequest {
        vs_currency: "usd".to_string(),
        per_page: Some(10),
        ..Default::default()
    }).await?;
    for coin in markets {
        println!("{}: ${:.2}", coin.name, coin.current_price.unwrap_or(0.0));
    }

    // 3. Get Trending Coins
    let trending = client.get_trending().await?;
    println!("Trending: {}", trending.coins[0].item.name);

    // 4. Get Global Market Data
    let global = client.get_global().await?;
    println!("BTC Dominance: {:.1}%",
        global.data.market_cap_percentage.get("btc").unwrap_or(&0.0));

    Ok(())
}
```

### Endpoints

| Method                        | Endpoint                          | Description                       |
| ----------------------------- | --------------------------------- | --------------------------------- |
| `get_simple_price`            | `/simple/price`                   | Get prices for coin IDs           |
| `get_supported_vs_currencies` | `/simple/supported_vs_currencies` | List supported vs currencies      |
| `get_coins_list`              | `/coins/list`                     | List all supported coins          |
| `get_coins_markets`           | `/coins/markets`                  | Market data with pagination       |
| `get_coin`                    | `/coins/{id}`                     | Detailed coin data by ID          |
| `get_coin_market_chart`       | `/coins/{id}/market_chart`        | Historical price/volume/marketcap |
| `get_coin_history`            | `/coins/{id}/history`             | Historical data at specific date  |
| `get_coin_ohlc`               | `/coins/{id}/ohlc`                | OHLC candlestick data             |
| `get_exchanges`               | `/exchanges`                      | List all exchanges                |
| `get_trending`                | `/search/trending`                | Trending coins, NFTs, categories  |
| `get_global`                  | `/global`                         | Global cryptocurrency stats       |

---

## Alternative.me Client

The Alternative.me client provides free access to cryptocurrency prices and the Fear & Greed Index.

> [!NOTE] > **No API key required**. Rate limit: 60 requests per minute.

### Quick Start

```rust
use polymarket_hft::client::alternativeme::{
    Client,
    GetTickerRequest,
    GetFearAndGreedRequest
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // 1. Get Latest Ticker Data (Top 5)
    let ticker = client.get_ticker(GetTickerRequest {
        limit: Some(5),
        ..Default::default()
    }).await?;

    println!("Top 5 Cryptocurrencies:");
    for crypto in ticker.data {
        println!("{}: ${}", crypto.name, crypto.quotes["USD"].price);
    }

    // 2. Get Global Market Metrics
    let global = client.get_global(Default::default()).await?;
    println!("Total Market Cap: ${:.0}B",
        global.data.quotes["USD"].total_market_cap / 1e9);

    // 3. Get Fear and Greed Index
    let fng = client.get_fear_and_greed(
        GetFearAndGreedRequest::default()
    ).await?;
    println!("Fear & Greed: {} ({})",
        fng.data[0].value,
        fng.data[0].value_classification
    );

    Ok(())
}
```

### Endpoints

| Method               | Endpoint          | Description               |
| -------------------- | ----------------- | ------------------------- |
| `get_listings`       | `/v2/listings/`   | All cryptocurrency IDs    |
| `get_ticker`         | `/v2/ticker/`     | Price, volume, market cap |
| `get_ticker_by_id`   | `/v2/ticker/{id}` | Specific cryptocurrency   |
| `get_global`         | `/v2/global/`     | Global market metrics     |
| `get_fear_and_greed` | `/fng/`           | Fear and Greed Index      |

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
