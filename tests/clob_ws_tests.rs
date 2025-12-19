//! CLOB WebSocket integration tests.
//!
//! These tests require network access and connect to the live Polymarket WebSocket.
//! They are marked `#[ignore]` and must be run explicitly with `--ignored`.

use polymarket_hft::client::polymarket::clob::ws::{ClobWsClient, WsAuth, WsMessage};
use std::time::Duration;

/// Test connecting to the market channel and receiving messages.
///
/// Run with: `cargo test --test clob_ws_tests -- --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn test_clob_ws_market_channel() {
    // Enable tracing for debugging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init()
        .ok();

    let mut client = ClobWsClient::builder().auto_reconnect(false).build();

    // Use a known active token ID (this may need updating)
    // Get a valid one from: polymarket gamma markets --limit 1
    let asset_ids = vec![
        "71321045679252212594626385532706912750332728571942532289631379312455583992563".to_string(),
    ];

    // Subscribe to market channel
    client
        .subscribe_market(asset_ids)
        .await
        .expect("Failed to subscribe");

    println!("Connected and subscribed to market channel");

    // Wait for a message (with timeout)
    let timeout = Duration::from_secs(30);
    let result = tokio::time::timeout(timeout, async { client.next_message().await }).await;

    match result {
        Ok(Some(msg)) => {
            println!("Received message type: {}", msg.event_type());
            match &msg {
                WsMessage::Book(book) => {
                    println!(
                        "  Book: asset={} bids={} asks={}",
                        book.asset_id,
                        book.bids.len(),
                        book.asks.len()
                    );
                }
                WsMessage::PriceChange(pc) => {
                    println!(
                        "  PriceChange: market={} changes={}",
                        pc.market,
                        pc.price_changes.len()
                    );
                }
                WsMessage::LastTradePrice(ltp) => {
                    println!("  LastTradePrice: price={} size={}", ltp.price, ltp.size);
                }
                _ => {
                    println!("  Other message type");
                }
            }
        }
        Ok(None) => {
            panic!("Connection closed without receiving a message");
        }
        Err(_) => {
            // Timeout is acceptable for market channel if there's no activity
            println!("Timeout waiting for message - market may be inactive");
        }
    }

    // Disconnect
    client.disconnect().await;
}

/// Test connecting to the user channel with authentication.
///
/// Requires environment variables:
/// - POLY_API_KEY
/// - POLY_API_SECRET  
/// - POLY_PASSPHRASE
///
/// Run with: `cargo test --test clob_ws_tests test_clob_ws_user_channel -- --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn test_clob_ws_user_channel() {
    // Enable tracing for debugging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init()
        .ok();

    // Get auth from environment
    let auth = match WsAuth::from_env() {
        Some(auth) => auth,
        None => {
            println!("Skipping user channel test - missing auth env vars");
            println!("Set POLY_API_KEY, POLY_API_SECRET, POLY_PASSPHRASE");
            return;
        }
    };

    let mut client = ClobWsClient::builder().auto_reconnect(false).build();

    // Subscribe to user channel with empty market IDs (receive all)
    let market_ids = vec![];

    client
        .subscribe_user(market_ids, auth)
        .await
        .expect("Failed to subscribe");

    println!("Connected and subscribed to user channel");

    // Wait briefly then disconnect (user events are rare)
    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("Disconnecting...");
    client.disconnect().await;
    println!("User channel test completed successfully");
}
