//! Integration tests for the Alternative.me API client.
//!
//! Tests marked with `#[ignore]` require network access to the live API.
//! Run them with: `cargo test --test alternativeme_api_tests -- --ignored --nocapture`

use polymarket_hft::client::alternativeme::{
    Client, GetFearAndGreedRequest, GetGlobalRequest, GetTickerByIdRequest, GetTickerRequest,
};

// =============================================================================
// Ticker Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access to live API"]
async fn test_get_ticker() {
    let client = Client::new();
    let result = client
        .get_ticker(GetTickerRequest {
            limit: Some(5),
            ..Default::default()
        })
        .await;

    assert!(result.is_ok(), "Failed to get ticker: {:?}", result.err());

    let ticker = result.unwrap();
    assert!(!ticker.data.is_empty(), "Ticker data should not be empty");
    assert!(ticker.data.len() <= 5, "Should respect limit parameter");

    // Check first ticker has required fields and USD quote
    let first = &ticker.data[0];
    assert!(!first.name.is_empty());
    assert!(first.quotes.contains_key("USD"), "Should have USD quote");

    let usd_quote = &first.quotes["USD"];
    println!(
        "{} ({}) - ${:.2}, 24h: {:.2}%",
        first.name,
        first.symbol,
        usd_quote.price,
        usd_quote.percent_change_24h.unwrap_or(0.0)
    );
}

#[tokio::test]
#[ignore = "requires network access to live API"]
async fn test_get_ticker_by_id() {
    let client = Client::new();
    let result = client
        .get_ticker_by_id("bitcoin", GetTickerByIdRequest::default())
        .await;

    assert!(
        result.is_ok(),
        "Failed to get Bitcoin ticker: {:?}",
        result.err()
    );

    let ticker = result.unwrap();
    assert!(!ticker.data.is_empty(), "Should return Bitcoin data");

    // Find Bitcoin in the response
    let btc = ticker.data.values().next().unwrap();
    assert_eq!(btc.symbol, "BTC");
    assert!(btc.quotes.contains_key("USD"));

    let usd_quote = &btc.quotes["USD"];
    println!(
        "Bitcoin: ${:.2}, market cap: ${:.0}",
        usd_quote.price, usd_quote.market_cap
    );
}

// =============================================================================
// Global Metrics Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access to live API"]
async fn test_get_global() {
    let client = Client::new();
    let result = client.get_global(GetGlobalRequest::default()).await;

    assert!(result.is_ok(), "Failed to get global: {:?}", result.err());

    let global = result.unwrap();
    assert!(
        global.data.active_cryptocurrencies > 0,
        "Should have active cryptocurrencies"
    );
    assert!(
        global.data.bitcoin_percentage_of_market_cap > 0.0,
        "BTC dominance should be positive"
    );
    assert!(global.data.quotes.contains_key("USD"));

    let usd = &global.data.quotes["USD"];
    println!(
        "Total market cap: ${:.0}B, BTC dominance: {:.1}%",
        usd.total_market_cap / 1e9,
        global.data.bitcoin_percentage_of_market_cap
    );
}

// =============================================================================
// Fear and Greed Index Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access to live API"]
async fn test_get_fear_and_greed() {
    let client = Client::new();
    let result = client
        .get_fear_and_greed(GetFearAndGreedRequest::default())
        .await;

    assert!(
        result.is_ok(),
        "Failed to get Fear & Greed: {:?}",
        result.err()
    );

    let fng = result.unwrap();
    assert_eq!(fng.name, "Fear and Greed Index");
    assert!(!fng.data.is_empty(), "Should have at least one data point");

    let latest = &fng.data[0];
    let value: i32 = latest.value.parse().expect("Value should be numeric");
    assert!((0..=100).contains(&value), "Value should be 0-100");

    println!(
        "Fear & Greed Index: {} ({})",
        latest.value, latest.value_classification
    );
}

#[tokio::test]
#[ignore = "requires network access to live API"]
async fn test_get_fear_and_greed_historical() {
    let client = Client::new();
    let result = client
        .get_fear_and_greed(GetFearAndGreedRequest {
            limit: Some(7),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "Failed to get historical F&G: {:?}",
        result.err()
    );

    let fng = result.unwrap();
    assert_eq!(fng.data.len(), 7, "Should return 7 days of history");

    println!("Last 7 days Fear & Greed Index:");
    for (i, day) in fng.data.iter().enumerate() {
        println!(
            "  Day {}: {} ({})",
            i + 1,
            day.value,
            day.value_classification
        );
    }
}

// =============================================================================
// Unit Tests (No Network Required)
// =============================================================================

#[test]
fn test_client_creation() {
    let _client = Client::new();
}

#[test]
fn test_ticker_request_default() {
    let request = GetTickerRequest::default();
    assert!(request.limit.is_none());
    assert!(request.start.is_none());
    assert!(request.convert.is_none());
}

#[test]
fn test_global_request_default() {
    let request = GetGlobalRequest::default();
    assert!(request.convert.is_none());
}

#[test]
fn test_fear_and_greed_request_default() {
    let request = GetFearAndGreedRequest::default();
    assert!(request.limit.is_none());
    assert!(request.format.is_none());
}
