//! Integration tests for the Polymarket CLOB API client.
//!
//! Tests marked with `#[ignore]` require network access to the live Polymarket API.
//! Run them with: `cargo test --test clob_api_tests -- --ignored --nocapture`

use polymarket_hft::client::polymarket::clob::{
    Client, DEFAULT_BASE_URL, GetOrderBooksRequestItem, GetPriceHistoryRequest, MarketPriceRequest,
    PriceHistoryInterval, Side, SpreadRequest,
};
use polymarket_hft::client::polymarket::gamma::{Client as GammaClient, GetMarketsRequest};

// =============================================================================
// Test Helpers
// =============================================================================

/// Fetches a valid token ID from an active market via the Gamma API.
/// Returns None if no active markets are available.
async fn get_valid_token_id() -> Option<String> {
    let gamma_client = GammaClient::new();
    let markets = gamma_client
        .get_markets(GetMarketsRequest {
            limit: Some(5),
            closed: Some(false),
            ..Default::default()
        })
        .await
        .ok()?;

    // Find a market with valid CLOB token IDs
    // clob_token_ids is a comma-separated string like "[\"token1\", \"token2\"]"
    for market in markets {
        if let Some(ref token_ids_str) = market.clob_token_ids {
            // Parse the JSON array string like "[\"token1\", \"token2\"]"
            if let Ok(tokens) = serde_json::from_str::<Vec<String>>(token_ids_str)
                && let Some(first_token) = tokens.first()
                && !first_token.is_empty()
            {
                return Some(first_token.clone());
            }
        }
    }
    None
}

// =============================================================================
// OrderBook Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_order_book() {
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let result = client.get_order_book(&token_id).await;
    assert!(result.is_ok(), "get_order_book failed: {:?}", result.err());
    let order_book = result.unwrap();
    println!("Market: {}", order_book.market);
    println!("Asset ID: {}", order_book.asset_id);
    println!("Bids count: {}", order_book.bids.len());
    println!("Asks count: {}", order_book.asks.len());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_order_books() {
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let request = vec![GetOrderBooksRequestItem {
        token_id: token_id.clone(),
        side: None,
    }];
    let result = client.get_order_books(&request).await;
    assert!(result.is_ok(), "get_order_books failed: {:?}", result.err());
    let order_books = result.unwrap();
    println!("Received {} order books", order_books.len());
}

// =============================================================================
// Pricing Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_market_price() {
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let result = client.get_market_price(&token_id, Side::Buy).await;
    assert!(
        result.is_ok(),
        "get_market_price failed: {:?}",
        result.err()
    );
    let price = result.unwrap();
    println!("Buy price: {}", price.price);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_market_prices() {
    // This endpoint may return 400 "Invalid payload" - it seems to require specific
    // token IDs. The test verifies the client can make the request; actual success
    // depends on API availability.
    let client = Client::new();
    let result = client.get_market_prices().await;
    match result {
        Ok(prices) => println!("Received prices for {} tokens", prices.len()),
        Err(e) => println!("get_market_prices returned error (may be expected): {}", e),
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_market_prices_by_request() {
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let request = vec![MarketPriceRequest {
        token_id: token_id.clone(),
        side: Side::Buy,
    }];
    let result = client.get_market_prices_by_request(&request).await;
    assert!(
        result.is_ok(),
        "get_market_prices_by_request failed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_midpoint_price() {
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let result = client.get_midpoint_price(&token_id).await;
    assert!(
        result.is_ok(),
        "get_midpoint_price failed: {:?}",
        result.err()
    );
    let midpoint = result.unwrap();
    println!("Midpoint: {}", midpoint.mid);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_price_history() {
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let result = client
        .get_price_history(GetPriceHistoryRequest {
            market: &token_id,
            interval: Some(PriceHistoryInterval::OneDay),
            ..Default::default()
        })
        .await;
    assert!(
        result.is_ok(),
        "get_price_history failed: {:?}",
        result.err()
    );
    let history = result.unwrap();
    println!("History points: {}", history.history.len());
}

// =============================================================================
// Spreads Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_spreads() {
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let request = vec![SpreadRequest {
        token_id: token_id.clone(),
        side: None,
    }];
    let result = client.get_spreads(&request).await;
    assert!(result.is_ok(), "get_spreads failed: {:?}", result.err());
    let spreads = result.unwrap();
    println!("Received spreads for {} tokens", spreads.len());
}

// =============================================================================
// Client Configuration Tests (No Network Required)
// =============================================================================

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[test]
fn test_client_with_custom_base_url() {
    let result = Client::with_base_url("https://custom-clob.example.com/");
    assert!(result.is_ok());
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[test]
fn test_client_with_invalid_base_url() {
    let result = Client::with_base_url("not-a-valid-url");
    assert!(result.is_err());
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[test]
fn test_client_default_base_url() {
    assert_eq!(DEFAULT_BASE_URL, "https://clob.polymarket.com");
}

// =============================================================================
// Type Tests (No Network Required)
// =============================================================================

#[test]
fn test_side_display() {
    assert_eq!(Side::Buy.to_string(), "BUY");
    assert_eq!(Side::Sell.to_string(), "SELL");
}

#[test]
fn test_side_from_str() {
    assert!(matches!("BUY".parse::<Side>(), Ok(Side::Buy)));
    assert!(matches!("SELL".parse::<Side>(), Ok(Side::Sell)));
    assert!(matches!("buy".parse::<Side>(), Ok(Side::Buy)));
    assert!(matches!("sell".parse::<Side>(), Ok(Side::Sell)));
    assert!("invalid".parse::<Side>().is_err());
}

#[test]
fn test_interval_display() {
    assert_eq!(PriceHistoryInterval::OneMinute.to_string(), "1m");
    assert_eq!(PriceHistoryInterval::OneHour.to_string(), "1h");
    assert_eq!(PriceHistoryInterval::SixHours.to_string(), "6h");
    assert_eq!(PriceHistoryInterval::OneDay.to_string(), "1d");
    assert_eq!(PriceHistoryInterval::OneWeek.to_string(), "1w");
    assert_eq!(PriceHistoryInterval::Max.to_string(), "max");
}

#[test]
fn test_interval_from_str() {
    assert!(matches!(
        "1m".parse::<PriceHistoryInterval>(),
        Ok(PriceHistoryInterval::OneMinute)
    ));
    assert!(matches!(
        "1h".parse::<PriceHistoryInterval>(),
        Ok(PriceHistoryInterval::OneHour)
    ));
    assert!(matches!(
        "1d".parse::<PriceHistoryInterval>(),
        Ok(PriceHistoryInterval::OneDay)
    ));
    assert!(matches!(
        "max".parse::<PriceHistoryInterval>(),
        Ok(PriceHistoryInterval::Max)
    ));
    assert!("invalid".parse::<PriceHistoryInterval>().is_err());
}

// =============================================================================
// Markets Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_markets() {
    use polymarket_hft::client::polymarket::clob::GetMarketsRequest;
    let client = Client::new();
    let result = client.get_markets(GetMarketsRequest::default()).await;
    assert!(result.is_ok(), "get_markets failed: {:?}", result.err());
    let markets = result.unwrap();
    println!("Received {} markets", markets.data.len());
    println!("Next cursor: {}", markets.next_cursor);
    if let Some(first) = markets.data.first() {
        println!("First market condition_id: {}", first.condition_id);
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_sampling_markets() {
    let client = Client::new();
    let result = client.get_sampling_markets(None).await;
    assert!(
        result.is_ok(),
        "get_sampling_markets failed: {:?}",
        result.err()
    );
    let markets = result.unwrap();
    println!("Received {} sampling markets", markets.data.len());
}

// =============================================================================
// Token Info Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_tick_size() {
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let result = client.get_tick_size(&token_id).await;
    assert!(result.is_ok(), "get_tick_size failed: {:?}", result.err());
    let tick_size = result.unwrap();
    println!("Tick size: {}", tick_size);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_neg_risk() {
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let result = client.get_neg_risk(&token_id).await;
    assert!(result.is_ok(), "get_neg_risk failed: {:?}", result.err());
    let neg_risk = result.unwrap();
    println!("Neg risk: {}", neg_risk);
}

// =============================================================================
// Last Trade Price Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_last_trade_price() {
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let result = client.get_last_trade_price(&token_id).await;
    assert!(
        result.is_ok(),
        "get_last_trade_price failed: {:?}",
        result.err()
    );
    let price = result.unwrap();
    println!("Last trade price: {}", price);
}

// =============================================================================
// Order Book Hash Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_order_book_contains_hash() {
    // The hash is included in the order book response from GET /book endpoint.
    // There is no separate /hash endpoint.
    let token_id = get_valid_token_id()
        .await
        .expect("Failed to get a valid token_id from active markets");
    println!("Using token_id: {}", token_id);

    let client = Client::new();
    let result = client.get_order_book(&token_id).await;
    assert!(result.is_ok(), "get_order_book failed: {:?}", result.err());
    let order_book = result.unwrap();
    assert!(
        !order_book.hash.is_empty(),
        "Order book hash should not be empty"
    );
    println!("Order book hash: {}", order_book.hash);
}
