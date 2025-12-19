# Library Usage Guide

This guide covers how to use `polymarket-hft` as a Rust library in your project.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
polymarket-hft = "0.0.5"
```

## Quick Start

```rust
use polymarket_hft::client::polymarket::data::{Client, GetUserPositionsRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Data API client
    let client = Client::new();

    // Check API health
    let health = client.health().await?;
    println!("API status: {}", health.data);

    // Get top holders for markets
    let markets = &["0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917"];
    let holders = client.get_market_top_holders(markets, None, None).await?;
    for h in holders {
        println!("Token {} has {} holders", h.token, h.holders.len());
    }

    // Get total value of user's positions
    let values = client.get_user_portfolio_value("0x56687bf447db6ffa42ffe2204a05edaa20f55839", None).await?;
    for v in values {
        println!("User {} has value: {}", v.user, v.value);
    }

    // Get user's traded markets count
    let traded = client.get_user_traded_markets("0x56687bf447db6ffa42ffe2204a05edaa20f55839").await?;
    println!("User {} has traded {} markets", traded.user, traded.traded);

    // Get open interest for markets
    let oi = client.get_open_interest(markets).await?;
    for item in oi {
        println!("Market {} has open interest: {}", item.market, item.value);
    }

    // Get live volume for an event
    let volume = client.get_event_live_volume(123).await?;
    println!("Total volume: {}", volume.total);

    // Get user's current positions
    let positions = client.get_user_positions(GetUserPositionsRequest {
        user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
        ..Default::default()
    }).await?;
    for pos in &positions {
        println!("Position: {} - Size: {}", pos.title, pos.size);
    }

    Ok(())
}
```

## Data API Client

### Creating a Client

```rust
use polymarket_hft::client::polymarket::data::Client;

// Create with default settings (includes automatic retry with exponential backoff)
let client = Client::new();

// Create with custom base URL
let client = Client::with_base_url("https://custom-api.example.com")?;

// Create with custom retry configuration (5 retries instead of default 3)
let client = Client::with_retries("https://data-api.polymarket.com", 5)?;

// Create with custom HTTP client (will be wrapped with default retry middleware)
let http_client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
let client = Client::with_http_client(http_client);
```

### Retry Behavior

All API clients include automatic retry with exponential backoff for transient failures:

- **Default retries**: 3 attempts
- **Retried errors**: Connection errors, timeouts, 5xx server errors
- **Not retried**: 4xx client errors (bad request, not found, etc.)

You can customize the retry count using `with_retries()` method on any client.

### Available Methods

| Method                                                | Description                         |
| ----------------------------------------------------- | ----------------------------------- |
| `health()`                                            | Check API health status             |
| `get_market_top_holders(markets, limit, min_balance)` | Get top holders for markets         |
| `get_user_portfolio_value(user, markets)`             | Get total value of user's positions |
| `get_user_traded_markets(user)`                       | Get user's traded markets count     |
| `get_open_interest(markets)`                          | Get open interest for markets       |
| `get_event_live_volume(event_id)`                     | Get live volume for an event        |
| `get_user_positions(request)`                         | Get current positions for a user    |
| `get_user_closed_positions(request)`                  | Get closed positions for a user     |
| `get_user_activity(request)`                          | Get on-chain activity for a user    |
| `get_trades(request)`                                 | Get trades for a user or markets    |

## Gamma API Client

```rust
use polymarket_hft::client::polymarket::gamma::{Client, GetMarketsRequest, GetEventsRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // List markets (first 5, include tags)
    let markets = client
        .get_markets(GetMarketsRequest {
            limit: Some(5),
            include_tag: Some(true),
            ..Default::default()
        })
        .await?;
    println!("Markets returned: {}", markets.len());

    // List events with related tags
    let events = client
        .get_events(GetEventsRequest {
            related_tags: Some(true),
            limit: Some(5),
            ..Default::default()
        })
        .await?;
    println!("Events returned: {}", events.len());

    Ok(())
}
```

### Available Methods (Gamma)

| Method                                               | Description                            |
| ---------------------------------------------------- | -------------------------------------- |
| `get_sports()`                                       | List sports metadata                   |
| `get_teams(request)`                                 | List teams                             |
| `get_tags(request)`                                  | List tags                              |
| `get_tag_by_id(id)`                                  | Get a tag by ID                        |
| `get_tag_by_slug(slug, include_tpl)`                 | Get a tag by slug                      |
| `get_tag_relationships_by_tag(...)`                  | Get tag relationships by tag ID        |
| `get_tag_relationships_by_slug(...)`                 | Get tag relationships by tag slug      |
| `get_tags_related_to_tag(...)`                       | Get related tags for a tag ID          |
| `get_tags_related_to_slug(...)`                      | Get related tags for a tag slug        |
| `get_events(request)`                                | List events                            |
| `get_event_by_id(id, include_chat, include_tpl)`     | Get an event by ID                     |
| `get_event_by_slug(slug, include_chat, include_tpl)` | Get an event by slug                   |
| `get_event_tags(id)`                                 | List tags for an event                 |
| `get_markets(request)`                               | List markets                           |
| `get_market_by_id(id, include_tag)`                  | Get a market by ID                     |
| `get_market_by_slug(slug, include_tag)`              | Get a market by slug                   |
| `get_market_tags(id)`                                | List tags for a market                 |
| `get_series(request)`                                | List series                            |
| `get_series_by_id(id, include_chat)`                 | Get a series by ID                     |
| `get_comments(request)`                              | List comments for an entity            |
| `get_comment_by_id(id, get_positions)`               | Get a comment by ID                    |
| `get_comments_by_user_address(...)`                  | List comments by user address          |
| `search(request)`                                    | Search markets, events, profiles, tags |

## CLOB API Client

The CLOB client provides access to Polymarket's Central Limit Order Book (CLOB) API.

### Public Client (Read-Only)

```rust
use polymarket_hft::client::polymarket::clob::{Client, Side, GetPriceHistoryRequest, PriceHistoryInterval};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Get order book for a token
    let order_book = client.get_order_book("token_id").await?;
    println!("Bids: {}, Asks: {}", order_book.bids.len(), order_book.asks.len());

    // Get market price
    let price = client.get_market_price("token_id", Side::Buy).await?;
    println!("Price: {}", price.price);

    // Get midpoint price
    let midpoint = client.get_midpoint_price("token_id").await?;
    println!("Midpoint: {}", midpoint.mid);

    // Get price history
    let history = client.get_price_history(GetPriceHistoryRequest {
        market: "token_id",
        interval: Some(PriceHistoryInterval::OneDay),
        ..Default::default()
    }).await?;
    println!("History points: {}", history.history.len());

    // Get simplified markets
    let markets = client.get_simplified_markets(None).await?;
    println!("Markets: {}", markets.data.len());

    // Get market trade events
    let events = client.get_market_trades_events("condition_id").await?;
    println!("Trade events: {}", events.len());

    Ok(())
}
```

### Trading Client (Authenticated)

```rust
use polymarket_hft::client::polymarket::clob::{
    TradingClient, ApiKeyCreds, Chain, OrderType, UserLimitOrder, TickSize, Side,
};
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create wallet from private key
    let wallet: PrivateKeySigner = "0x...".parse()?;

    // Get or create API credentials
    let creds = ApiKeyCreds {
        key: "your_api_key".to_string(),
        secret: "your_secret".to_string(),
        passphrase: "your_passphrase".to_string(),
    };

    // Create trading client
    let client = TradingClient::new(wallet.clone(), creds, Chain::Polygon);

    // Or create/derive API key programmatically
    let new_creds = client.create_or_derive_api_key(None).await?;

    // Create and post a limit order
    let order = UserLimitOrder {
        token_id: "token_id".to_string(),
        price: 0.55,
        size: 10.0,
        side: Side::Buy,
        fee_rate_bps: None,
        nonce: None,
        expiration: None,
        taker: None,
    };

    let result = client.create_and_post_limit_order(
        &order,
        TickSize::PointZeroOne,
        false,
        OrderType::Gtc,
    ).await?;
    println!("Order posted: {:?}", result);

    // Get open orders
    let orders = client.get_open_orders(None).await?;
    println!("Open orders: {}", orders.len());

    // Cancel all orders
    client.cancel_all().await?;

    Ok(())
}
```

### Available Methods (CLOB Public Client)

| Method                                    | Description                               |
| ----------------------------------------- | ----------------------------------------- |
| `get_ok()`                                | Health check                              |
| `get_server_time()`                       | Get server time                           |
| `get_order_book(token_id)`                | Get order book for a token                |
| `get_order_books(request)`                | Get order books for multiple tokens       |
| `get_order_book_hash(token_id)`           | Get order book hash for change detection  |
| `get_market_price(token_id, side)`        | Get market price for a token and side     |
| `get_market_prices()`                     | Get all market prices                     |
| `get_market_prices_by_request(request)`   | Get prices for specified tokens/sides     |
| `get_midpoint_price(token_id)`            | Get midpoint price for a token            |
| `get_price_history(request)`              | Get price history for a token             |
| `get_last_trade_price(token_id)`          | Get last trade price                      |
| `get_last_trades_prices(token_ids)`       | Get last trade prices for multiple tokens |
| `get_spreads(request)`                    | Get bid-ask spreads for tokens            |
| `get_tick_size(token_id)`                 | Get tick size for a token                 |
| `get_neg_risk(token_id)`                  | Check if token uses negative risk         |
| `get_fee_rate_bps(token_id)`              | Get fee rate in basis points              |
| `get_markets(request)`                    | Get markets with pagination               |
| `get_market(condition_id)`                | Get a single market                       |
| `get_simplified_markets(cursor)`          | Get simplified markets                    |
| `get_sampling_markets(cursor)`            | Get sampling markets                      |
| `get_sampling_simplified_markets(cursor)` | Get sampling simplified markets           |
| `get_market_trades_events(condition_id)`  | Get live trade events for a market        |

### Available Methods (CLOB Trading Client)

| Method                                     | Description                     |
| ------------------------------------------ | ------------------------------- |
| **API Key Management (L1)**                |                                 |
| `create_api_key(nonce)`                    | Create new API key with L1 auth |
| `derive_api_key(nonce)`                    | Derive existing API key         |
| `create_or_derive_api_key(nonce)`          | Create or derive API key        |
| **API Key Management (L2)**                |                                 |
| `get_api_keys()`                           | Get all API keys                |
| `delete_api_key()`                         | Delete current API key          |
| `get_closed_only_mode()`                   | Get ban/closed-only status      |
| **Order Submission**                       |                                 |
| `post_order(order, order_type)`            | Post a signed order             |
| `post_orders(orders)`                      | Post multiple signed orders     |
| `create_limit_order(order, tick, neg)`     | Create signed limit order       |
| `create_market_order(order, tick, neg)`    | Create signed market order      |
| `create_and_post_limit_order(...)`         | Create and post limit order     |
| `create_and_post_market_order(...)`        | Create and post market order    |
| **Order Queries**                          |                                 |
| `get_open_order(order_id)`                 | Get an open order by ID         |
| `get_open_orders(params)`                  | Get all open orders             |
| `get_trades(params)`                       | Get trade history               |
| `get_trades_paginated(params, cursor)`     | Get trades with pagination      |
| **Order Cancellation**                     |                                 |
| `cancel_order(order_id)`                   | Cancel a single order           |
| `cancel_orders(order_ids)`                 | Cancel multiple orders          |
| `cancel_all()`                             | Cancel all open orders          |
| `cancel_market_orders(market, asset_id)`   | Cancel orders for a market      |
| **Balance & Allowance**                    |                                 |
| `get_balance_allowance(params)`            | Get balance and allowance       |
| `update_balance_allowance(params)`         | Update balance/allowance cache  |
| **Order Scoring**                          |                                 |
| `is_order_scoring(order_id)`               | Check if order is scoring       |
| `are_orders_scoring(order_ids)`            | Check multiple orders scoring   |
| **Notifications**                          |                                 |
| `get_notifications()`                      | Get notifications               |
| `drop_notifications(ids)`                  | Delete notifications            |
| **Utility**                                |                                 |
| `calculate_market_price(token, side, amt)` | Calculate optimal market price  |

## CLOB WebSocket Client

The CLOB WebSocket client provides real-time streaming of order book updates, price changes, and user events.

```rust
use polymarket_hft::client::polymarket::clob::ws::{ClobWsClient, WsAuth, WsMessage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClobWsClient::builder()
        .auto_reconnect(true)
        .build();

    // Subscribe to market channel (orderbook, price changes)
    let asset_ids = vec!["71321045679252212594626385532706912750332728571942532289631379312455583992563".to_string()];
    client.subscribe_market(asset_ids).await?;

    // Process messages
    while let Some(msg) = client.next_message().await {
        match msg {
            WsMessage::Book(book) => println!("Order book: {} bids, {} asks", book.bids.len(), book.asks.len()),
            WsMessage::PriceChange(pc) => println!("Price change: {}", pc.market),
            WsMessage::LastTradePrice(ltp) => println!("Last trade: {} @ {}", ltp.size, ltp.price),
            _ => {}
        }
    }

    client.disconnect().await;
    Ok(())
}
```

### User Channel (Authenticated)

```rust
// Get auth from environment variables (POLY_API_KEY, POLY_API_SECRET, POLY_PASSPHRASE)
let auth = WsAuth::from_env().expect("Missing auth env vars");

// Or create manually
let auth = WsAuth::new("api_key", "api_secret", "passphrase");

// Subscribe to user channel
client.subscribe_user(vec![], auth).await?;

while let Some(msg) = client.next_message().await {
    match msg {
        WsMessage::Trade(trade) => println!("Trade: {} status={:?}", trade.id, trade.status),
        WsMessage::Order(order) => println!("Order: {} type={:?}", order.id, order.order_type),
        _ => {}
    }
}
```

### Message Types

| Message          | Channel | Description                                    |
| ---------------- | ------- | ---------------------------------------------- |
| `Book`           | Market  | Order book snapshot (bids/asks)                |
| `PriceChange`    | Market  | Price level changes with best bid/ask          |
| `TickSizeChange` | Market  | Tick size updates                              |
| `LastTradePrice` | Market  | Last trade execution                           |
| `Trade`          | User    | Trade events (MATCHED, MINED, CONFIRMED, etc.) |
| `Order`          | User    | Order events (PLACEMENT, UPDATE, CANCELLATION) |

### Method Details

#### `get_user_positions`

Get current positions for a user with various filter and sort options.

```rust
use polymarket_hft::client::polymarket::data::{Client, GetUserPositionsRequest, PositionSortBy, SortDirection};

// Get all positions for a user
let positions = client.get_user_positions(GetUserPositionsRequest {
    user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ..Default::default()
}).await?;

// Get positions with filters
let positions = client.get_user_positions(GetUserPositionsRequest {
    user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    size_threshold: Some(1.0),
    redeemable: Some(false),
    mergeable: Some(false),
    limit: Some(10),
    offset: Some(0),
    sort_by: Some(PositionSortBy::CashPnl),
    sort_direction: Some(SortDirection::Desc),
    ..Default::default()
}).await?;
```

#### `get_user_closed_positions`

Get closed positions for a user.

```rust
use polymarket_hft::client::polymarket::data::{Client, GetUserClosedPositionsRequest, ClosedPositionSortBy, SortDirection};

// Get all closed positions for a user
let positions = client.get_user_closed_positions(GetUserClosedPositionsRequest {
    user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ..Default::default()
}).await?;

// Get closed positions with filters
let positions = client.get_user_closed_positions(GetUserClosedPositionsRequest {
    user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    limit: Some(10),
    offset: Some(0),
    sort_by: Some(ClosedPositionSortBy::RealizedPnl),
    sort_direction: Some(SortDirection::Desc),
    ..Default::default()
}).await?;
```

#### `get_user_activity`

Get on-chain activity for a user.

```rust
use polymarket_hft::client::polymarket::data::{Client, GetUserActivityRequest, ActivityType, ActivitySortBy, SortDirection, TradeSide};

// Get all activity for a user
let activity = client.get_user_activity(GetUserActivityRequest {
    user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    ..Default::default()
}).await?;

// Get activity with filters
let activity_types = vec![ActivityType::Trade];
let activity = client.get_user_activity(GetUserActivityRequest {
    user: "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
    limit: Some(50),
    activity_types: Some(&activity_types),
    sort_by: Some(ActivitySortBy::Timestamp),
    sort_direction: Some(SortDirection::Desc),
    side: Some(TradeSide::Buy),
    ..Default::default()
}).await?;
```

#### `get_trades`

Get trades for a user or markets.

```rust
use polymarket_hft::client::polymarket::data::{Client, GetTradesRequest, TradeSide, TradeFilterType};

// Get trades for a user
let trades = client.get_trades(GetTradesRequest {
    user: Some("0x56687bf447db6ffa42ffe2204a05edaa20f55839"),
    limit: Some(100),
    ..Default::default()
}).await?;

// Get trades for a market with filters
let markets = vec![
    "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",
];
let trades = client.get_trades(GetTradesRequest {
    markets: Some(&markets),
    limit: Some(50),
    taker_only: Some(true),
    filter_type: Some(TradeFilterType::Cash),
    filter_amount: Some(10.0),
    side: Some(TradeSide::Buy),
    ..Default::default()
}).await?;
```

## RTDS (Real-Time Data Service) Client

The RTDS client provides WebSocket-based real-time data streaming.

```rust
use polymarket_hft::client::polymarket::rtds::{RtdsClient, Subscription, ClobAuth};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RtdsClient::builder()
        .auto_reconnect(true)
        .build();

    // Connect to RTDS
    client.connect().await?;

    // Subscribe to crypto prices
    client.subscribe(vec![
        Subscription::new("crypto_prices", "update")
            .with_filter(r#"{"symbol":"BTCUSDT"}"#)
    ]).await?;

    // Process messages
    while let Some(msg) = client.next_message().await {
        println!("{}/{}: {:?}", msg.topic, msg.message_type, msg.payload);
    }

    client.disconnect().await;
    Ok(())
}
```

### Subscription with CLOB Authentication

```rust
// For clob_user topic, authentication is required
client.subscribe(vec![
    Subscription::new("clob_user", "*")
        .with_clob_auth(ClobAuth::new("key", "secret", "passphrase"))
]).await?;
```

### Supported Topics

| Topic                     | Types                                                                                                        | Filter                                                   |
| ------------------------- | ------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------- |
| `activity`                | `trades`, `orders_matched`                                                                                   | `{"event_slug":"..."}` or `{"market_slug":"..."}`        |
| `comments`                | `comment_created`, `comment_removed`, `reaction_created`, `reaction_removed`                                 | `{"parentEntityID":n,"parentEntityType":"Event/Series"}` |
| `rfq`                     | `request_*`, `quote_*`                                                                                       | None                                                     |
| `crypto_prices`           | `update`                                                                                                     | `{"symbol":"BTCUSDT"}`                                   |
| `crypto_prices_chainlink` | `update`                                                                                                     | `{"symbol":"..."}`                                       |
| `equity_prices`           | `update`                                                                                                     | `{"symbol":"AAPL"}`                                      |
| `clob_user`               | `order`, `trade`                                                                                             | None (requires ClobAuth)                                 |
| `clob_market`             | `price_change`, `agg_orderbook`, `last_trade_price`, `tick_size_change`, `market_created`, `market_resolved` | `["token_id1","token_id2",...]`                          |

### Message Payload Types

All payload types are available in `polymarket_hft::client::rtds::types`:

- `ActivityTrade` - Activity trades
- `Comment`, `Reaction` - Comments and reactions
- `Request`, `Quote` - RFQ messages
- `CryptoPrice`, `EquityPrice` - Price updates
- `ClobOrder`, `ClobUserTrade` - CLOB user events
- `PriceChanges`, `AggOrderbook`, `LastTradePrice`, `TickSizeChange`, `ClobMarket` - CLOB market data

## Error Handling

The SDK provides comprehensive error handling with strongly typed error variants:

| Error Type   | Description                                   |
| ------------ | --------------------------------------------- |
| `Http`       | HTTP request failures (from `reqwest::Error`) |
| `Api`        | API returned an error response                |
| `BadRequest` | Invalid parameters or input (e.g., addresses) |
| `Serde`      | Serialization or deserialization errors       |
| `Url`        | URL parsing errors                            |
| `Other`      | Generic errors with custom messages           |

### Example

```rust
use polymarket_hft::client::polymarket::data::Client;
use polymarket_hft::error::PolymarketError;

async fn example() {
    let client = Client::new();

    match client.get_user_traded_markets("invalid-address").await {
        Ok(result) => println!("Traded: {}", result.traded),
        Err(PolymarketError::BadRequest(msg)) => {
            eprintln!("Bad request: {}", msg)
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

## API Documentation

For detailed API documentation, visit [docs.rs/polymarket-hft](https://docs.rs/polymarket-hft).
