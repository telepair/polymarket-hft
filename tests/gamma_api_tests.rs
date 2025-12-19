//! Integration tests for the Polymarket Gamma Markets API client.
//!
//! Tests marked with `#[ignore]` require network access to the live Polymarket Gamma API.
//! Run them with: `cargo test --test gamma_api_tests -- --ignored --nocapture`

use polymarket_hft::PolymarketError;
use polymarket_hft::client::polymarket::gamma::{
    Client, GetCommentsByUserAddressRequest, GetCommentsRequest, GetEventsRequest,
    GetMarketsRequest, GetSeriesRequest, GetTagsRequest, GetTeamsRequest, SearchRequest,
};

// =============================================================================
// Smoke Tests (network)
// =============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_sports() {
    let client = Client::new();
    let result = client.get_sports().await;

    assert!(result.is_ok(), "get_sports should succeed: {:?}", result);
    let sports = result.unwrap();
    assert!(
        !sports.is_empty(),
        "Expected at least one sport entry from Gamma"
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_tags() {
    let client = Client::new();
    let result = client
        .get_tags(GetTagsRequest {
            limit: Some(5),
            offset: Some(0),
            ..Default::default()
        })
        .await;

    assert!(result.is_ok(), "get_tags should succeed: {:?}", result);
    let tags = result.unwrap();
    assert!(!tags.is_empty(), "Expected at least one tag");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_series() {
    let client = Client::new();
    let result = client
        .get_series(GetSeriesRequest {
            limit: Some(5),
            offset: Some(0),
            ..Default::default()
        })
        .await;

    assert!(result.is_ok(), "get_series should succeed: {:?}", result);
    let series = result.unwrap();
    assert!(!series.is_empty(), "Expected at least one series");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_events() {
    let client = Client::new();
    let result = client
        .get_events(GetEventsRequest {
            limit: Some(5),
            offset: Some(0),
            ..Default::default()
        })
        .await;

    assert!(result.is_ok(), "get_events should succeed: {:?}", result);
    let events = result.unwrap();
    assert!(!events.is_empty(), "Expected at least one event");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_markets() {
    let client = Client::new();
    let result = client
        .get_markets(GetMarketsRequest {
            limit: Some(5),
            offset: Some(0),
            ..Default::default()
        })
        .await;

    assert!(result.is_ok(), "get_markets should succeed: {:?}", result);
    let markets = result.unwrap();
    assert!(!markets.is_empty(), "Expected at least one market");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_get_teams() {
    let client = Client::new();
    let result = client
        .get_teams(GetTeamsRequest {
            limit: Some(5),
            offset: Some(0),
            order: None,
            ascending: None,
            league: None,
            name: None,
            abbreviation: None,
        })
        .await;

    assert!(result.is_ok(), "get_teams should succeed: {:?}", result);
    let teams = result.unwrap();
    assert!(!teams.is_empty(), "Expected at least one team");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tag_detail_flows() {
    let client = Client::new();
    let tags = client
        .get_tags(GetTagsRequest {
            limit: Some(1),
            offset: Some(0),
            ..Default::default()
        })
        .await
        .expect("get_tags should succeed");
    assert!(!tags.is_empty(), "Expected at least one tag");
    let tag = &tags[0];

    let by_id = client.get_tag_by_id(&tag.id).await;
    assert!(by_id.is_ok(), "get_tag_by_id should succeed: {:?}", by_id);

    if let Some(slug) = &tag.slug {
        let by_slug = client.get_tag_by_slug(slug, None).await;
        assert!(
            by_slug.is_ok(),
            "get_tag_by_slug should succeed: {:?}",
            by_slug
        );

        let rel_slug = client.get_tag_relationships_by_slug(slug, None, None).await;
        assert!(
            rel_slug.is_ok(),
            "get_tag_relationships_by_slug should succeed: {:?}",
            rel_slug
        );

        let related_slug = client.get_tags_related_to_slug(slug, None, None).await;
        assert!(
            related_slug.is_ok(),
            "get_tags_related_to_slug should succeed: {:?}",
            related_slug
        );
    }

    let rel_id = client
        .get_tag_relationships_by_tag(&tag.id, None, None)
        .await;
    assert!(
        rel_id.is_ok(),
        "get_tag_relationships_by_id should succeed: {:?}",
        rel_id
    );

    let related_id = client.get_tags_related_to_tag(&tag.id, None, None).await;
    assert!(
        related_id.is_ok(),
        "get_tags_related_to_id should succeed: {:?}",
        related_id
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_event_detail_flows() {
    let client = Client::new();
    let events = client
        .get_events(GetEventsRequest {
            limit: Some(1),
            offset: Some(0),
            ..Default::default()
        })
        .await
        .expect("get_events should succeed");
    assert!(!events.is_empty(), "Expected at least one event");
    let event = &events[0];

    let by_id = client.get_event_by_id(&event.id, None, None).await;
    assert!(by_id.is_ok(), "get_event_by_id should succeed: {:?}", by_id);

    if let Some(slug) = &event.slug {
        let by_slug = client.get_event_by_slug(slug, None, None).await;
        assert!(
            by_slug.is_ok(),
            "get_event_by_slug should succeed: {:?}",
            by_slug
        );
    }

    let tags = client.get_event_tags(&event.id).await;
    assert!(tags.is_ok(), "get_event_tags should succeed: {:?}", tags);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_market_detail_flows() {
    let client = Client::new();
    let markets = client
        .get_markets(GetMarketsRequest {
            limit: Some(1),
            offset: Some(0),
            ..Default::default()
        })
        .await
        .expect("get_markets should succeed");
    assert!(!markets.is_empty(), "Expected at least one market");
    let market = &markets[0];

    let by_id = client.get_market_by_id(&market.id, None).await;
    assert!(
        by_id.is_ok(),
        "get_market_by_id should succeed: {:?}",
        by_id
    );

    if let Some(slug) = &market.slug {
        let by_slug = client.get_market_by_slug(slug, None).await;
        assert!(
            by_slug.is_ok(),
            "get_market_by_slug should succeed: {:?}",
            by_slug
        );
    }

    let tags = client.get_market_tags(&market.id).await;
    assert!(tags.is_ok(), "get_market_tags should succeed: {:?}", tags);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_series_detail_flow() {
    let client = Client::new();
    let series_list = client
        .get_series(GetSeriesRequest {
            limit: Some(1),
            offset: Some(0),
            ..Default::default()
        })
        .await
        .expect("get_series should succeed");
    assert!(!series_list.is_empty(), "Expected at least one series");
    let series = &series_list[0];

    let by_id = client.get_series_by_id(&series.id, None).await;
    assert!(
        by_id.is_ok(),
        "get_series_by_id should succeed: {:?}",
        by_id
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_comments_endpoints() {
    let client = Client::new();
    let events = client
        .get_events(GetEventsRequest {
            limit: Some(1),
            offset: Some(0),
            ..Default::default()
        })
        .await
        .expect("get_events should succeed");
    assert!(!events.is_empty(), "Expected at least one event");
    let event_id = events[0].id.as_str();
    let comments = client
        .get_comments(GetCommentsRequest {
            parent_entity_type: Some("Event"),
            parent_entity_id: Some(event_id),
            limit: Some(1),
            offset: Some(0),
            order: None,
            ascending: None,
            get_positions: None,
            holders_only: None,
        })
        .await;
    assert!(
        comments.is_ok(),
        "get_comments should succeed: {:?}",
        comments
    );

    if let Ok(list) = comments
        && let Some(first) = list.first()
        && let Some(user_address) = &first.user_address
    {
        let by_user = client
            .get_comments_by_user_address(GetCommentsByUserAddressRequest {
                user_address,
                limit: Some(1),
                offset: Some(0),
                order: None,
                ascending: None,
            })
            .await;
        assert!(
            by_user.is_ok(),
            "get_comments_by_user_address should succeed: {:?}",
            by_user
        );
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_search_endpoint() {
    let client = Client::new();
    let result = client
        .search(SearchRequest {
            q: "US",
            cache: None,
            events_status: None,
            page: None,
            events_tag: None,
            keep_closed_markets: None,
            sort: None,
            ascending: None,
            search_tags: None,
            search_profiles: None,
            recurrence: None,
            exclude_tag_id: None,
            optimized: None,
            limit_per_type: Some(1),
        })
        .await;

    assert!(result.is_ok(), "search should succeed: {:?}", result);
}

// =============================================================================
// Validation Tests (no network)
// =============================================================================

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[test]
fn test_markets_invalid_limit() {
    let request = GetMarketsRequest {
        limit: Some(0), // invalid: must be 1..=1000
        ..Default::default()
    };
    let result = request.validate();

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
#[test]
fn test_markets_invalid_limit_upper_bound() {
    let request = GetMarketsRequest {
        limit: Some(1001), // invalid: must be 1..=1000
        ..Default::default()
    };
    let result = request.validate();

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("limit"),
                "Error should mention 'limit' upper bound: {}",
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
#[test]
fn test_events_invalid_tag_id() {
    let request = GetEventsRequest {
        tag_id: Some("abc"), // must be digits
        ..Default::default()
    };
    let result = request.validate();

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("tag_id"),
                "Error should mention 'tag_id': {}",
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
#[test]
fn test_search_empty_query_rejected() {
    let request = SearchRequest {
        q: "   ",
        cache: None,
        events_status: None,
        page: None,
        events_tag: None,
        keep_closed_markets: None,
        sort: None,
        ascending: None,
        search_tags: None,
        search_profiles: None,
        recurrence: None,
        exclude_tag_id: None,
        optimized: None,
        limit_per_type: Some(1),
    };

    let result = request.validate();

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("query"),
                "Error should mention empty query validation: {}",
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
#[test]
fn test_search_invalid_exclude_tag_id() {
    let request = SearchRequest {
        q: "economy",
        cache: None,
        events_status: None,
        page: None,
        events_tag: None,
        keep_closed_markets: None,
        sort: None,
        ascending: None,
        search_tags: None,
        search_profiles: None,
        recurrence: None,
        exclude_tag_id: Some(vec!["abc".to_string()]), // should be digits
        optimized: None,
        limit_per_type: Some(1),
    };

    let result = request.validate();

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(
                msg.contains("digits"),
                "Error should mention exclude_tag_id digits validation: {}",
                msg
            );
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}

#[test]
fn test_client_with_invalid_base_url() {
    let result = Client::with_base_url("not a url");

    assert!(
        result.is_err(),
        "Client::with_base_url should fail on invalid URL"
    );
}

#[test]
fn test_client_with_custom_base_url() {
    let result = Client::with_base_url("https://example.com/");

    assert!(
        result.is_ok(),
        "Client::with_base_url should accept valid URL"
    );
}

#[cfg_attr(
    target_os = "macos",
    ignore = "reqwest native TLS unavailable in sandboxed macOS tests"
)]
#[test]
fn test_series_invalid_slug() {
    let request = GetSeriesRequest {
        slug: Some("   "), // empty after trim
        ..Default::default()
    };
    let result = request.validate();

    assert!(result.is_err());
    match result.unwrap_err() {
        PolymarketError::BadRequest(msg) => {
            assert!(msg.contains("slug"), "Error should mention 'slug': {}", msg);
        }
        e => panic!("Expected BadRequest error, got: {:?}", e),
    }
}
