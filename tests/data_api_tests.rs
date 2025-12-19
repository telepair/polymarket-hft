//! Integration tests for the Polymarket SDK Data API client.
//!
//! Tests marked with `#[ignore]` require network access to the live Polymarket API.
//! Run them with: `cargo test --test data_api_tests -- --ignored --nocapture`

use polymarket_hft::PolymarketError;
use polymarket_hft::client::polymarket::data::{
    ActivitySortBy, ActivityType, Client, ClosedPositionSortBy, GetTradesRequest,
    GetUserActivityRequest, GetUserClosedPositionsRequest, GetUserPositionsRequest, PositionSortBy,
    SortDirection, TradeFilterType, TradeSide,
};

// Well-known test data from Polymarket
// Using a public user address that has trading history
const TEST_USER: &str = "0x56687bf447db6ffa42ffe2204a05edaa20f55839";
const TEST_MARKET: &str = "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917";
const TEST_EVENT_ID: i64 = 903;

// =============================================================================
// Health Check Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_health_check() {
    let client = Client::new();
    let result = client.health().await;

    assert!(result.is_ok(), "Health check should succeed: {:?}", result);
    let health = result.unwrap();
    assert_eq!(health.data, "OK", "Health status should be 'OK'");
}

// =============================================================================
// User Positions Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_user_positions() {
    let client = Client::new();
    let result = client
        .get_user_positions(GetUserPositionsRequest {
            user: TEST_USER,
            limit: Some(10),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_user_positions should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_user_positions_with_filters() {
    let client = Client::new();
    let result = client
        .get_user_positions(GetUserPositionsRequest {
            user: TEST_USER,
            size_threshold: Some(0.1),
            redeemable: Some(false),
            mergeable: Some(false),
            limit: Some(5),
            offset: Some(0),
            sort_by: Some(PositionSortBy::CashPnl),
            sort_direction: Some(SortDirection::Desc),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_user_positions with filters should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_user_closed_positions() {
    let client = Client::new();
    let result = client
        .get_user_closed_positions(GetUserClosedPositionsRequest {
            user: TEST_USER,
            limit: Some(10),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_user_closed_positions should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_user_closed_positions_with_sort() {
    let client = Client::new();
    let result = client
        .get_user_closed_positions(GetUserClosedPositionsRequest {
            user: TEST_USER,
            limit: Some(5),
            sort_by: Some(ClosedPositionSortBy::Timestamp),
            sort_direction: Some(SortDirection::Desc),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_user_closed_positions with sort should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_user_portfolio_value() {
    let client = Client::new();
    let result = client.get_user_portfolio_value(TEST_USER, None).await;

    assert!(
        result.is_ok(),
        "get_user_portfolio_value should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_user_portfolio_value_with_markets() {
    let client = Client::new();
    let markets = vec![TEST_MARKET];
    let result = client
        .get_user_portfolio_value(TEST_USER, Some(&markets))
        .await;

    assert!(
        result.is_ok(),
        "get_user_portfolio_value with markets should succeed: {:?}",
        result
    );
}

// =============================================================================
// User Activity Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_user_activity() {
    let client = Client::new();
    let result = client
        .get_user_activity(GetUserActivityRequest {
            user: TEST_USER,
            limit: Some(10),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_user_activity should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_user_activity_with_filters() {
    let client = Client::new();
    let activity_types = vec![ActivityType::Trade];
    let result = client
        .get_user_activity(GetUserActivityRequest {
            user: TEST_USER,
            limit: Some(20),
            activity_types: Some(&activity_types),
            sort_by: Some(ActivitySortBy::Timestamp),
            sort_direction: Some(SortDirection::Desc),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_user_activity with filters should succeed: {:?}",
        result
    );
}

// =============================================================================
// Trades Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_trades_for_user() {
    let client = Client::new();
    let result = client
        .get_trades(GetTradesRequest {
            user: Some(TEST_USER),
            limit: Some(10),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_trades for user should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_trades_for_market() {
    let client = Client::new();
    let markets = vec![TEST_MARKET];
    let result = client
        .get_trades(GetTradesRequest {
            markets: Some(&markets),
            limit: Some(10),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_trades for market should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_trades_with_filters() {
    let client = Client::new();
    let result = client
        .get_trades(GetTradesRequest {
            user: Some(TEST_USER),
            limit: Some(20),
            taker_only: Some(true),
            filter_type: Some(TradeFilterType::Cash),
            filter_amount: Some(1.0),
            side: Some(TradeSide::Buy),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_ok(),
        "get_trades with filters should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_user_traded_markets() {
    let client = Client::new();
    let result = client.get_user_traded_markets(TEST_USER).await;

    assert!(
        result.is_ok(),
        "get_user_traded_markets should succeed: {:?}",
        result
    );

    let traded = result.unwrap();
    assert_eq!(traded.user.to_lowercase(), TEST_USER.to_lowercase());
}

// =============================================================================
// Market Data Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_open_interest() {
    let client = Client::new();
    let markets = vec![TEST_MARKET];
    let result = client.get_open_interest(&markets).await;

    assert!(
        result.is_ok(),
        "get_open_interest should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_event_live_volume() {
    let client = Client::new();
    let result = client.get_event_live_volume(TEST_EVENT_ID).await;

    assert!(
        result.is_ok(),
        "get_event_live_volume should succeed: {:?}",
        result
    );
}

// =============================================================================
// Holders Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_market_top_holders() {
    let client = Client::new();
    let markets = vec![TEST_MARKET];
    let result = client
        .get_market_top_holders(&markets, Some(10), None)
        .await;

    assert!(
        result.is_ok(),
        "get_market_top_holders should succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_market_top_holders_with_min_balance() {
    let client = Client::new();
    let markets = vec![TEST_MARKET];
    let result = client
        .get_market_top_holders(&markets, Some(5), Some(100))
        .await;

    assert!(
        result.is_ok(),
        "get_market_top_holders with min_balance should succeed: {:?}",
        result
    );
}

// =============================================================================
// Validation Tests (No Network Required)
// =============================================================================

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[tokio::test]
async fn test_invalid_user_address() {
    let client = Client::new();
    let result = client
        .get_user_positions(GetUserPositionsRequest {
            user: "invalid_address",
            ..Default::default()
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("address") || msg.contains("0x"),
                "Error should mention address format: {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[tokio::test]
async fn test_invalid_limit() {
    let client = Client::new();
    let result = client
        .get_user_positions(GetUserPositionsRequest {
            user: TEST_USER,
            limit: Some(1000), // exceeds max of 500
            ..Default::default()
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("limit"),
                "Error should mention 'limit': {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[tokio::test]
async fn test_invalid_offset() {
    let client = Client::new();
    let result = client
        .get_user_positions(GetUserPositionsRequest {
            user: TEST_USER,
            offset: Some(10001), // exceeds max of 10000
            ..Default::default()
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("offset"),
                "Error should mention 'offset': {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[tokio::test]
async fn test_invalid_market_id() {
    let client = Client::new();
    let markets = vec!["not_a_valid_market_id"];
    let result = client.get_open_interest(&markets).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("market"),
                "Error should mention 'market': {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[tokio::test]
async fn test_invalid_event_id() {
    let client = Client::new();
    let result = client.get_event_live_volume(0).await; // event_id must be >= 1

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("event"),
                "Error should mention 'event': {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[tokio::test]
async fn test_trades_filter_type_without_amount() {
    let client = Client::new();
    let result = client
        .get_trades(GetTradesRequest {
            user: Some(TEST_USER),
            filter_type: Some(TradeFilterType::Cash),
            // filter_amount is missing
            ..Default::default()
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("filterType") || msg.contains("filterAmount"),
                "Error should mention filter parameters: {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[tokio::test]
async fn test_trades_negative_filter_amount() {
    let client = Client::new();
    let result = client
        .get_trades(GetTradesRequest {
            user: Some(TEST_USER),
            filter_type: Some(TradeFilterType::Cash),
            filter_amount: Some(-1.0),
            ..Default::default()
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("filterAmount"),
                "Error should mention non-negative filterAmount: {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[tokio::test]
async fn test_trades_limit_out_of_range() {
    let client = Client::new();
    let result = client
        .get_trades(GetTradesRequest {
            user: Some(TEST_USER),
            limit: Some(10001), // exceeds max of 10000
            ..Default::default()
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("limit"),
                "Error should mention 'limit' range: {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[tokio::test]
async fn test_trades_market_and_event_conflict() {
    let client = Client::new();
    let markets = vec![TEST_MARKET];
    let event_ids = vec![TEST_EVENT_ID];
    let result = client
        .get_trades(GetTradesRequest {
            markets: Some(&markets),
            event_ids: Some(&event_ids),
            ..Default::default()
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("mutually exclusive"),
                "Error should mention mutually exclusive filters: {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[tokio::test]
async fn test_user_activity_market_and_event_conflict() {
    let client = Client::new();
    let markets = vec![TEST_MARKET];
    let event_ids = vec![TEST_EVENT_ID];
    let result = client
        .get_user_activity(GetUserActivityRequest {
            user: TEST_USER,
            markets: Some(&markets),
            event_ids: Some(&event_ids),
            ..Default::default()
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("mutually exclusive"),
                "Error should mention mutually exclusive filters: {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
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
    let result = Client::with_base_url("https://custom-api.example.com/");
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
fn test_client_default() {
    let client = Client::default();
    // Just ensure it compiles and doesn't panic
    drop(client);
}
