//! RTDS (Real-Time Data Service) WebSocket integration tests.
//!
//! These tests require network access and connect to the live Polymarket WebSocket.
//! They are marked `#[ignore]` and must be run explicitly with `--ignored`.

use polymarket_hft::client::polymarket::rtds::{RtdsClient, Subscription};
use std::time::Duration;

/// Test connecting to the RTDS server and receiving a message.
///
/// Run with: `cargo test --test rtds_tests -- --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn test_rtds_connect_and_subscribe() {
    // Enable tracing for debugging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init()
        .ok();

    let mut client = RtdsClient::builder().auto_reconnect(false).build();

    // Connect
    client.connect().await.expect("Failed to connect");

    // Subscribe to crypto prices (BTC)
    client
        .subscribe(vec![
            Subscription::new("crypto_prices", "update").with_filter(r#"{"symbol":"BTCUSDT"}"#),
        ])
        .await
        .expect("Failed to subscribe");

    // Wait for a message (with timeout)
    let timeout = Duration::from_secs(30);
    let result = tokio::time::timeout(timeout, async {
        // The first message may be a historical data dump
        client.next_message().await
    })
    .await;

    match result {
        Ok(Some(msg)) => {
            println!("Received message:");
            println!("  Topic: {}", msg.topic);
            println!("  Type: {}", msg.message_type);
            println!("  Timestamp: {}", msg.timestamp);
            println!("  Payload: {}", msg.payload);
            assert_eq!(msg.topic, "crypto_prices");
        }
        Ok(None) => {
            panic!("Connection closed without receiving a message");
        }
        Err(_) => {
            panic!("Timeout waiting for message");
        }
    }

    // Disconnect
    client.disconnect().await;
}

/// Test subscribing to CLOB market data.
///
/// Run with: `cargo test --test rtds_tests test_rtds_clob_market -- --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn test_rtds_clob_market() {
    let mut client = RtdsClient::builder().auto_reconnect(false).build();

    client.connect().await.expect("Failed to connect");

    // Subscribe to CLOB market events (market_created doesn't require token IDs filter)
    client
        .subscribe(vec![Subscription::new("clob_market", "market_created")])
        .await
        .expect("Failed to subscribe");

    // Just verify we can subscribe successfully - market_created events are rare
    println!("Successfully subscribed to clob_market/market_created");

    // Disconnect after a short delay
    tokio::time::sleep(Duration::from_secs(2)).await;
    client.disconnect().await;
}
