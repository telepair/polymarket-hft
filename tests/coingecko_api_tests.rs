//! CoinGecko API integration tests.
//!
//! These tests require `CG_API_KEY` environment variable to be set.
//! Run with: `cargo test --test coingecko_api_tests`

use polymarket_hft::client::coingecko::{
    Client, CoinDetailRequest, CoinHistoryRequest, CoinsListRequest, CoinsMarketsRequest,
    ExchangesRequest, MarketChartRequest, OhlcRequest, SimplePriceRequest,
};

fn get_client() -> Option<Client> {
    std::env::var("CG_API_KEY").ok().map(Client::new)
}

#[tokio::test]
async fn test_get_simple_price() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let request = SimplePriceRequest {
        ids: "bitcoin".to_string(),
        vs_currencies: "usd".to_string(),
        include_market_cap: Some(true),
        include_24hr_vol: Some(true),
        include_24hr_change: Some(true),
        include_last_updated_at: Some(true),
    };

    let result = client.get_simple_price(request).await;
    assert!(
        result.is_ok(),
        "get_simple_price failed: {:?}",
        result.err()
    );

    let response = result.unwrap();
    assert!(
        response.contains_key("bitcoin"),
        "Response should contain bitcoin"
    );

    let bitcoin = response.get("bitcoin").unwrap();
    let usd_price = bitcoin.get("usd").and_then(|v| *v);
    assert!(
        usd_price.is_some() && usd_price.unwrap() > 0.0,
        "Bitcoin price should be positive"
    );
}

#[tokio::test]
async fn test_get_coins_list() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let request = CoinsListRequest::default();
    let result = client.get_coins_list(request).await;
    assert!(result.is_ok(), "get_coins_list failed: {:?}", result.err());

    let response = result.unwrap();
    assert!(!response.is_empty(), "Coins list should not be empty");

    // Check that Bitcoin exists in the list
    let bitcoin = response.iter().find(|c| c.id == "bitcoin");
    assert!(bitcoin.is_some(), "Bitcoin should be in the coins list");
    assert_eq!(bitcoin.unwrap().symbol, "btc");
}

#[tokio::test]
async fn test_get_coins_markets() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let request = CoinsMarketsRequest {
        vs_currency: "usd".to_string(),
        ids: Some("bitcoin".to_string()),
        per_page: Some(1),
        ..Default::default()
    };

    let result = client.get_coins_markets(request).await;
    assert!(
        result.is_ok(),
        "get_coins_markets failed: {:?}",
        result.err()
    );

    let response = result.unwrap();
    assert_eq!(response.len(), 1, "Should return exactly 1 coin");
    assert_eq!(response[0].id, "bitcoin");
    assert!(
        response[0].current_price.is_some(),
        "Bitcoin should have a price"
    );
    assert!(
        response[0].market_cap.is_some(),
        "Bitcoin should have market cap"
    );
}

#[tokio::test]
async fn test_get_trending() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let result = client.get_trending().await;
    assert!(result.is_ok(), "get_trending failed: {:?}", result.err());

    let response = result.unwrap();
    // Trending may have coins, NFTs, or categories
    // At minimum, there should be some trending coins
    assert!(
        !response.coins.is_empty(),
        "Trending coins should not be empty"
    );

    // Check that trending coins have basic data
    let first_coin = &response.coins[0];
    assert!(!first_coin.item.id.is_empty(), "Coin should have an ID");
    assert!(!first_coin.item.name.is_empty(), "Coin should have a name");
}

#[tokio::test]
async fn test_get_global() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let result = client.get_global().await;
    assert!(result.is_ok(), "get_global failed: {:?}", result.err());

    let response = result.unwrap();
    let data = &response.data;

    // Check some basic global metrics
    assert!(
        data.active_cryptocurrencies.is_some() && data.active_cryptocurrencies.unwrap() > 0,
        "Should have active cryptocurrencies"
    );

    assert!(
        !data.total_market_cap.is_empty(),
        "Should have total market cap data"
    );

    // Check BTC dominance
    let btc_dominance = data.market_cap_percentage.get("btc");
    assert!(
        btc_dominance.is_some() && *btc_dominance.unwrap() > 0.0,
        "BTC dominance should be positive"
    );
}

#[tokio::test]
async fn test_get_supported_vs_currencies() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let result = client.get_supported_vs_currencies().await;
    assert!(
        result.is_ok(),
        "get_supported_vs_currencies failed: {:?}",
        result.err()
    );

    let response = result.unwrap();
    assert!(!response.is_empty(), "Should have supported currencies");
    assert!(response.contains(&"usd".to_string()), "Should support USD");
    assert!(response.contains(&"btc".to_string()), "Should support BTC");
}

#[tokio::test]
async fn test_get_exchanges() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let request = ExchangesRequest {
        per_page: Some(5),
        ..Default::default()
    };

    let result = client.get_exchanges(request).await;
    assert!(result.is_ok(), "get_exchanges failed: {:?}", result.err());

    let response = result.unwrap();
    assert!(!response.is_empty(), "Should have exchanges");
    assert!(response.len() <= 5, "Should respect per_page limit");

    // Check first exchange has basic data
    let first = &response[0];
    assert!(!first.id.is_empty(), "Exchange should have an ID");
    assert!(!first.name.is_empty(), "Exchange should have a name");
}

#[tokio::test]
async fn test_get_coin() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let request = CoinDetailRequest {
        id: "bitcoin".to_string(),
        localization: Some(false),
        tickers: Some(false),
        market_data: Some(true),
        community_data: Some(false),
        developer_data: Some(false),
        sparkline: Some(false),
    };

    let result = client.get_coin(request).await;
    assert!(result.is_ok(), "get_coin failed: {:?}", result.err());

    let response = result.unwrap();
    assert_eq!(response.id, "bitcoin");
    assert_eq!(response.symbol, "btc");
    assert!(!response.name.is_empty(), "Should have a name");
}

#[tokio::test]
async fn test_get_coin_market_chart() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let request = MarketChartRequest {
        id: "bitcoin".to_string(),
        vs_currency: "usd".to_string(),
        days: "1".to_string(),
        interval: None,
    };

    let result = client.get_coin_market_chart(request).await;
    assert!(
        result.is_ok(),
        "get_coin_market_chart failed: {:?}",
        result.err()
    );

    let response = result.unwrap();
    assert!(!response.prices.is_empty(), "Should have price data");
    assert!(
        !response.market_caps.is_empty(),
        "Should have market cap data"
    );
    assert!(
        !response.total_volumes.is_empty(),
        "Should have volume data"
    );
}

#[tokio::test]
async fn test_get_coin_history() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let request = CoinHistoryRequest {
        id: "bitcoin".to_string(),
        date: "30-12-2024".to_string(), // dd-mm-yyyy format
        localization: Some(false),
    };

    let result = client.get_coin_history(request).await;
    assert!(
        result.is_ok(),
        "get_coin_history failed: {:?}",
        result.err()
    );

    let response = result.unwrap();
    assert_eq!(response.id, Some("bitcoin".to_string()));
    assert_eq!(response.symbol, Some("btc".to_string()));
    assert!(response.market_data.is_some(), "Should have market data");
}

#[tokio::test]
async fn test_get_coin_ohlc() {
    let Some(client) = get_client() else {
        eprintln!("Skipping test: CG_API_KEY not set");
        return;
    };

    let request = OhlcRequest {
        id: "bitcoin".to_string(),
        vs_currency: "usd".to_string(),
        days: "1".to_string(),
    };

    let result = client.get_coin_ohlc(request).await;
    assert!(result.is_ok(), "get_coin_ohlc failed: {:?}", result.err());

    let response = result.unwrap();
    assert!(!response.is_empty(), "Should have OHLC data");

    // OHLC format: [timestamp, open, high, low, close]
    let first = &response[0];
    assert_eq!(first.len(), 5, "OHLC should have 5 values");
}
