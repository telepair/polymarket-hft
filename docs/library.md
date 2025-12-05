# Library Usage Guide

This guide covers how to use `polymarket-hft` as a Rust library in your project.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
polymarket-hft = "0.0.1"
```

## Quick Start

```rust
use polymarket_hft::data::{Client, GetUserPositionsRequest};

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
use polymarket_hft::data::Client;

// Create with default settings
let client = Client::new();

// Create with custom base URL
let client = Client::with_base_url("https://custom-api.example.com")?;

// Create with custom HTTP client
let http_client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
let client = Client::with_http_client(http_client);
```

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
use polymarket_hft::gamma::{Client, GetMarketsRequest, GetEventsRequest};

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

### Method Details

#### `get_user_positions`

Get current positions for a user with various filter and sort options.

```rust
use polymarket_hft::data::{Client, GetUserPositionsRequest, PositionSortBy, SortDirection};

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
use polymarket_hft::data::{Client, GetUserClosedPositionsRequest, ClosedPositionSortBy, SortDirection};

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
use polymarket_hft::data::{Client, GetUserActivityRequest, ActivityType, ActivitySortBy, SortDirection, TradeSide};

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
use polymarket_hft::data::{Client, GetTradesRequest, TradeSide, TradeFilterType};

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
use polymarket_hft::data::Client;
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
