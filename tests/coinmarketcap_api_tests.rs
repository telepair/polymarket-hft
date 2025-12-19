//! Integration tests for the CoinMarketCap API client.
//!
//! Tests marked with `#[ignore]` require network access to the live CoinMarketCap API.
//! Run them with: `cargo test --test coinmarketcap_api_tests -- --ignored --nocapture`
//!
//! **Note**: These tests require a valid CMC_API_KEY environment variable.
//! Get a free API key at: https://coinmarketcap.com/api/

use polymarket_hft::client::coinmarketcap::{
    Client, GetFearAndGreedLatestRequest, GetGlobalMetricsQuotesLatestRequest,
    GetListingsLatestRequest,
};

/// Helper to get API key from environment.
fn get_api_key() -> String {
    std::env::var("CMC_API_KEY").expect("CMC_API_KEY environment variable must be set")
}

// =============================================================================
// Listings Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access and CMC_API_KEY"]
async fn test_get_listings_latest() {
    let client = Client::new(get_api_key());
    let result = client
        .get_listings_latest(GetListingsLatestRequest {
            limit: Some(5),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_listings_latest should succeed: {:?}",
        result
    );
    let response = result.unwrap();
    assert!(!response.data.is_empty(), "Should return at least one coin");
    assert_eq!(response.status.error_code, 0, "Error code should be 0");

    // Verify first coin has expected fields
    let first = &response.data[0];
    assert!(!first.name.is_empty(), "Coin name should not be empty");
    assert!(!first.symbol.is_empty(), "Coin symbol should not be empty");
    assert!(first.quote.contains_key("USD"), "Should have USD quote");
}

#[tokio::test]
#[ignore = "requires network access and CMC_API_KEY"]
async fn test_get_listings_latest_with_filters() {
    let client = Client::new(get_api_key());
    let result = client
        .get_listings_latest(GetListingsLatestRequest {
            start: Some(1),
            limit: Some(10),
            price_min: Some(100.0),
            sort: Some("market_cap".to_string()),
            sort_dir: Some("desc".to_string()),
            convert: Some("USD".to_string()),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_listings_latest with filters should succeed: {:?}",
        result
    );
    let response = result.unwrap();

    // All returned coins should have price >= 100
    for coin in &response.data {
        if let Some(usd_quote) = coin.quote.get("USD") {
            assert!(
                usd_quote.price >= 100.0,
                "Coin {} price {} should be >= 100",
                coin.symbol,
                usd_quote.price
            );
        }
    }
}

// =============================================================================
// Global Metrics Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access and CMC_API_KEY"]
async fn test_get_global_metrics_quotes_latest() {
    let client = Client::new(get_api_key());
    let result = client
        .get_global_metrics_quotes_latest(GetGlobalMetricsQuotesLatestRequest::default())
        .await;

    assert!(
        result.is_ok(),
        "get_global_metrics_quotes_latest should succeed: {:?}",
        result
    );
    let response = result.unwrap();
    assert_eq!(response.status.error_code, 0, "Error code should be 0");

    // Verify essential metrics
    assert!(
        response.data.btc_dominance > 0.0,
        "BTC dominance should be positive"
    );
    assert!(
        response.data.active_cryptocurrencies > 0,
        "Should have active cryptocurrencies"
    );
    assert!(
        response.data.quote.contains_key("USD"),
        "Should have USD quote"
    );
}

#[tokio::test]
#[ignore = "requires network access and CMC_API_KEY"]
async fn test_get_global_metrics_with_convert() {
    let client = Client::new(get_api_key());
    let result = client
        .get_global_metrics_quotes_latest(GetGlobalMetricsQuotesLatestRequest {
            convert: Some("EUR".to_string()),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_global_metrics with EUR convert should succeed: {:?}",
        result
    );
    let response = result.unwrap();
    assert!(
        response.data.quote.contains_key("EUR"),
        "Should have EUR quote"
    );
}

// =============================================================================
// Fear and Greed Index Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access and CMC_API_KEY"]
async fn test_get_fear_and_greed_latest() {
    let client = Client::new(get_api_key());
    let result = client
        .get_fear_and_greed_latest(GetFearAndGreedLatestRequest::default())
        .await;

    assert!(
        result.is_ok(),
        "get_fear_and_greed_latest should succeed: {:?}",
        result
    );
    let response = result.unwrap();
    assert_eq!(response.status.error_code, 0, "Error code should be 0");

    // Fear and Greed index should be between 0-100
    assert!(
        response.data.value >= 0.0 && response.data.value <= 100.0,
        "Fear and Greed value {} should be between 0-100",
        response.data.value
    );
    assert!(
        !response.data.value_classification.is_empty(),
        "Value classification should not be empty"
    );
}

// =============================================================================
// API Key Info Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access and CMC_API_KEY"]
async fn test_get_key_info() {
    let client = Client::new(get_api_key());
    let result = client.get_key_info().await;

    assert!(result.is_ok(), "get_key_info should succeed: {:?}", result);
    let response = result.unwrap();
    assert_eq!(response.status.error_code, 0, "Error code should be 0");

    // Verify plan info for Basic plan (some fields may be omitted depending on plan)
    assert!(
        response.data.plan.credit_limit_monthly.unwrap_or(0) > 0,
        "Should have monthly credit limit"
    );
    assert!(
        response.data.plan.rate_limit_minute.unwrap_or(0) > 0,
        "Should have rate limit per minute"
    );

    // Verify usage tracking
    if let Some(current_month) = &response.data.usage.current_month {
        assert!(
            current_month.credits_left.unwrap_or(0) >= 0,
            "Monthly credits left should be >= 0"
        );
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_invalid_api_key() {
    let client = Client::new("invalid-api-key-12345");
    let result = client
        .get_listings_latest(GetListingsLatestRequest::default())
        .await;

    // Should fail with unauthorized error
    // The error handling depends on the response - CMC returns 401 or error in body
    assert!(
        result.is_ok(),
        "Request completes but response contains error"
    );
    let response = result.unwrap();
    assert_ne!(
        response.status.error_code, 0,
        "Error code should be non-zero for invalid API key"
    );
}

// =============================================================================
// Unit Tests (No Network Required)
// =============================================================================

#[test]
fn test_client_creation() {
    let client = Client::new("test-key");
    // Just ensure it compiles and doesn't panic
    drop(client);
}

#[test]
fn test_request_default() {
    let request = GetListingsLatestRequest::default();
    assert!(request.limit.is_none());
    assert!(request.start.is_none());
    assert!(request.convert.is_none());
}

#[test]
fn test_global_metrics_request_default() {
    let request = GetGlobalMetricsQuotesLatestRequest::default();
    assert!(request.convert.is_none());
    assert!(request.convert_id.is_none());
}
