//! Market models and endpoints for the Gamma API.

use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};
use url::Url;

use crate::error::{PolymarketError, Result};

use super::Client;
use super::events::{Category, Collection, Event, OptimizedImage};
use super::helpers::{
    deserialize_option_f64, deserialize_option_i64, deserialize_option_u64, validate_tag_id,
};
use super::tags::Tag;

/// Market representation from the Gamma API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    pub id: String,
    pub question: Option<String>,
    #[serde(alias = "conditionId")]
    pub condition_id: Option<String>,
    pub slug: Option<String>,
    #[serde(alias = "twitterCardImage")]
    pub twitter_card_image: Option<String>,
    #[serde(alias = "resolutionSource")]
    pub resolution_source: Option<String>,
    #[serde(alias = "endDate")]
    pub end_date: Option<String>,
    pub category: Option<String>,
    #[serde(alias = "ammType")]
    pub amm_type: Option<String>,
    pub liquidity: Option<String>,
    #[serde(alias = "sponsorName")]
    pub sponsor_name: Option<String>,
    #[serde(alias = "sponsorImage")]
    pub sponsor_image: Option<String>,
    #[serde(alias = "startDate")]
    pub start_date: Option<String>,
    #[serde(alias = "xAxisValue")]
    pub x_axis_value: Option<String>,
    #[serde(alias = "yAxisValue")]
    pub y_axis_value: Option<String>,
    #[serde(alias = "denominationToken")]
    pub denomination_token: Option<String>,
    pub fee: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    #[serde(alias = "lowerBound")]
    pub lower_bound: Option<String>,
    #[serde(alias = "upperBound")]
    pub upper_bound: Option<String>,
    pub description: Option<String>,
    pub outcomes: Option<String>,
    #[serde(alias = "outcomePrices")]
    pub outcome_prices: Option<String>,
    pub volume: Option<String>,
    pub active: Option<bool>,
    #[serde(alias = "marketType")]
    pub market_type: Option<String>,
    #[serde(alias = "formatType")]
    pub format_type: Option<String>,
    #[serde(alias = "lowerBoundDate")]
    pub lower_bound_date: Option<String>,
    #[serde(alias = "upperBoundDate")]
    pub upper_bound_date: Option<String>,
    pub closed: Option<bool>,
    #[serde(alias = "marketMakerAddress")]
    pub market_maker_address: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "createdBy"
    )]
    pub created_by: Option<u64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "updatedBy"
    )]
    pub updated_by: Option<u64>,
    #[serde(alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(alias = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(alias = "closedTime")]
    pub closed_time: Option<String>,
    pub archived: Option<bool>,
    #[serde(alias = "wideFormat")]
    pub wide_format: Option<bool>,
    pub new: Option<bool>,
    #[serde(alias = "mailchimpTag")]
    pub mailchimp_tag: Option<String>,
    pub featured: Option<bool>,
    pub resolved_by: Option<String>,
    pub restricted: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "marketGroup"
    )]
    pub market_group: Option<u64>,
    #[serde(alias = "groupItemTitle")]
    pub group_item_title: Option<String>,
    #[serde(alias = "groupItemThreshold")]
    pub group_item_threshold: Option<String>,
    #[serde(alias = "questionID")]
    pub question_id: Option<String>,
    #[serde(alias = "umaEndDate")]
    pub uma_end_date: Option<String>,
    #[serde(alias = "enableOrderBook")]
    pub enable_order_book: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "orderPriceMinTickSize"
    )]
    pub order_price_min_tick_size: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "orderMinSize"
    )]
    pub order_min_size: Option<f64>,
    #[serde(alias = "umaResolutionStatus")]
    pub uma_resolution_status: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "curationOrder"
    )]
    pub curation_order: Option<u64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volumeNum"
    )]
    pub volume_num: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "liquidityNum"
    )]
    pub liquidity_num: Option<f64>,
    #[serde(alias = "endDateIso")]
    pub end_date_iso: Option<String>,
    #[serde(alias = "startDateIso")]
    pub start_date_iso: Option<String>,
    #[serde(alias = "umaEndDateIso")]
    pub uma_end_date_iso: Option<String>,
    #[serde(alias = "hasReviewedDates")]
    pub has_reviewed_dates: Option<bool>,
    #[serde(alias = "readyForCron")]
    pub ready_for_cron: Option<bool>,
    #[serde(alias = "commentsEnabled")]
    pub comments_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub volume24hr: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub volume1wk: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub volume1mo: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub volume1yr: Option<f64>,
    #[serde(alias = "gameStartTime")]
    pub game_start_time: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "secondsDelay"
    )]
    pub seconds_delay: Option<u64>,
    #[serde(alias = "clobTokenIds")]
    pub clob_token_ids: Option<String>,
    #[serde(alias = "disqusThread")]
    pub disqus_thread: Option<String>,
    #[serde(alias = "shortOutcomes")]
    pub short_outcomes: Option<String>,
    #[serde(alias = "teamAID")]
    pub team_a_id: Option<String>,
    #[serde(alias = "teamBID")]
    pub team_b_id: Option<String>,
    #[serde(alias = "umaBond")]
    pub uma_bond: Option<String>,
    #[serde(alias = "umaReward")]
    pub uma_reward: Option<String>,
    #[serde(alias = "fpmmLive")]
    pub fpmm_live: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volume24hrAmm"
    )]
    pub volume24hr_amm: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volume1wkAmm"
    )]
    pub volume1wk_amm: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volume1moAmm"
    )]
    pub volume1mo_amm: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volume1yrAmm"
    )]
    pub volume1yr_amm: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volume24hrClob"
    )]
    pub volume24hr_clob: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volume1wkClob"
    )]
    pub volume1wk_clob: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volume1moClob"
    )]
    pub volume1mo_clob: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volume1yrClob"
    )]
    pub volume1yr_clob: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volumeAmm"
    )]
    pub volume_amm: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "volumeClob"
    )]
    pub volume_clob: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "liquidityAmm"
    )]
    pub liquidity_amm: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "liquidityClob"
    )]
    pub liquidity_clob: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "makerBaseFee"
    )]
    pub maker_base_fee: Option<u64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "takerBaseFee"
    )]
    pub taker_base_fee: Option<u64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_u64",
        alias = "customLiveness"
    )]
    pub custom_liveness: Option<u64>,
    #[serde(alias = "acceptingOrders")]
    pub accepting_orders: Option<bool>,
    #[serde(alias = "notificationsEnabled")]
    pub notifications_enabled: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_option_i64")]
    pub score: Option<i64>,
    #[serde(alias = "imageOptimized")]
    pub image_optimized: Option<OptimizedImage>,
    #[serde(alias = "iconOptimized")]
    pub icon_optimized: Option<OptimizedImage>,
    pub events: Option<Vec<Event>>,
    pub categories: Option<Vec<Category>>,
    pub collections: Option<Vec<Collection>>,
    pub tags: Option<Vec<Tag>>,
    pub creator: Option<String>,
    pub ready: Option<bool>,
    pub funded: Option<bool>,
    #[serde(alias = "pastSlugs")]
    pub past_slugs: Option<String>,
    #[serde(alias = "readyTimestamp")]
    pub ready_timestamp: Option<String>,
    #[serde(alias = "fundedTimestamp")]
    pub funded_timestamp: Option<String>,
    #[serde(alias = "acceptingOrdersTimestamp")]
    pub accepting_orders_timestamp: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub competitive: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "rewardsMinSize"
    )]
    pub rewards_min_size: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "rewardsMaxSpread"
    )]
    pub rewards_max_spread: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub spread: Option<f64>,
    #[serde(alias = "automaticallyResolved")]
    pub automatically_resolved: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "oneDayPriceChange"
    )]
    pub one_day_price_change: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "oneHourPriceChange"
    )]
    pub one_hour_price_change: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "oneWeekPriceChange"
    )]
    pub one_week_price_change: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "oneMonthPriceChange"
    )]
    pub one_month_price_change: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "oneYearPriceChange"
    )]
    pub one_year_price_change: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "lastTradePrice"
    )]
    pub last_trade_price: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "bestBid"
    )]
    pub best_bid: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_option_f64",
        alias = "bestAsk"
    )]
    pub best_ask: Option<f64>,
    #[serde(alias = "automaticallyActive")]
    pub automatically_active: Option<bool>,
    #[serde(alias = "clearBookOnStart")]
    pub clear_book_on_start: Option<bool>,
    #[serde(alias = "chartColor")]
    pub chart_color: Option<String>,
    #[serde(alias = "seriesColor")]
    pub series_color: Option<String>,
    #[serde(alias = "showGmpSeries")]
    pub show_gmp_series: Option<bool>,
    #[serde(alias = "showGmpOutcome")]
    pub show_gmp_outcome: Option<bool>,
    #[serde(alias = "manualActivation")]
    pub manual_activation: Option<bool>,
    #[serde(alias = "negRiskOther")]
    pub neg_risk_other: Option<bool>,
    #[serde(alias = "gameId")]
    pub game_id: Option<String>,
    #[serde(alias = "groupItemRange")]
    pub group_item_range: Option<String>,
    #[serde(alias = "sportsMarketType")]
    pub sports_market_type: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_f64")]
    pub line: Option<f64>,
    #[serde(alias = "umaResolutionStatuses")]
    pub uma_resolution_statuses: Option<String>,
    #[serde(alias = "pendingDeployment")]
    pub pending_deployment: Option<bool>,
    pub deploying: Option<bool>,
    #[serde(alias = "deployingTimestamp")]
    pub deploying_timestamp: Option<String>,
    #[serde(alias = "scheduledDeploymentTimestamp")]
    pub scheduled_deployment_timestamp: Option<String>,
    #[serde(alias = "rfqEnabled")]
    pub rfq_enabled: Option<bool>,
    #[serde(alias = "eventStartTime")]
    pub event_start_time: Option<String>,
}

/// Request parameters for listing markets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetMarketsRequest<'a> {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order: Option<&'a str>,
    pub ascending: Option<bool>,
    pub id: Option<Vec<String>>,
    pub slug: Option<Vec<String>>,
    pub clob_token_ids: Option<Vec<String>>,
    pub condition_ids: Option<Vec<String>>,
    pub market_maker_address: Option<Vec<String>>,
    pub liquidity_num_min: Option<f64>,
    pub liquidity_num_max: Option<f64>,
    pub volume_num_min: Option<f64>,
    pub volume_num_max: Option<f64>,
    pub start_date_min: Option<&'a str>,
    pub start_date_max: Option<&'a str>,
    pub end_date_min: Option<&'a str>,
    pub end_date_max: Option<&'a str>,
    pub tag_id: Option<&'a str>,
    pub related_tags: Option<bool>,
    pub cyom: Option<bool>,
    pub uma_resolution_status: Option<&'a str>,
    pub game_id: Option<&'a str>,
    pub sports_market_types: Option<Vec<String>>,
    pub rewards_min_size: Option<f64>,
    pub question_ids: Option<Vec<String>>,
    pub include_tag: Option<bool>,
    pub closed: Option<bool>,
}

impl<'a> GetMarketsRequest<'a> {
    /// Validates request parameters before sending.
    pub fn validate(&self) -> Result<()> {
        if let Some(limit) = self.limit
            && !(1..=1000).contains(&limit)
        {
            return Err(PolymarketError::bad_request(
                "limit must be between 1 and 1000",
            ));
        }

        validate_tag_id(self.tag_id)?;
        Ok(())
    }

    /// Builds the request URL using the provided base URL.
    pub(crate) fn build_url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();
        url.set_path("markets");
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(limit) = self.limit {
                pairs.append_pair("limit", &limit.to_string());
            }
            if let Some(offset) = self.offset {
                pairs.append_pair("offset", &offset.to_string());
            }
            if let Some(order) = self.order {
                pairs.append_pair("order", order);
            }
            if let Some(ascending) = self.ascending {
                pairs.append_pair("ascending", &ascending.to_string());
            }
            if let Some(ids) = &self.id {
                for id in ids {
                    pairs.append_pair("id", id);
                }
            }
            if let Some(slugs) = &self.slug {
                for slug in slugs {
                    pairs.append_pair("slug", slug);
                }
            }
            if let Some(clob_token_ids) = &self.clob_token_ids {
                for token in clob_token_ids {
                    pairs.append_pair("clob_token_ids", token);
                }
            }
            if let Some(condition_ids) = &self.condition_ids {
                for condition_id in condition_ids {
                    pairs.append_pair("condition_ids", condition_id);
                }
            }
            if let Some(market_maker_addresses) = &self.market_maker_address {
                for addr in market_maker_addresses {
                    pairs.append_pair("market_maker_address", addr);
                }
            }
            if let Some(liquidity_num_min) = self.liquidity_num_min {
                pairs.append_pair("liquidity_num_min", &liquidity_num_min.to_string());
            }
            if let Some(liquidity_num_max) = self.liquidity_num_max {
                pairs.append_pair("liquidity_num_max", &liquidity_num_max.to_string());
            }
            if let Some(volume_num_min) = self.volume_num_min {
                pairs.append_pair("volume_num_min", &volume_num_min.to_string());
            }
            if let Some(volume_num_max) = self.volume_num_max {
                pairs.append_pair("volume_num_max", &volume_num_max.to_string());
            }
            if let Some(start_date_min) = self.start_date_min {
                pairs.append_pair("start_date_min", start_date_min);
            }
            if let Some(start_date_max) = self.start_date_max {
                pairs.append_pair("start_date_max", start_date_max);
            }
            if let Some(end_date_min) = self.end_date_min {
                pairs.append_pair("end_date_min", end_date_min);
            }
            if let Some(end_date_max) = self.end_date_max {
                pairs.append_pair("end_date_max", end_date_max);
            }
            if let Some(tag_id) = self.tag_id {
                pairs.append_pair("tag_id", tag_id);
            }
            if let Some(related_tags) = self.related_tags {
                pairs.append_pair("related_tags", &related_tags.to_string());
            }
            if let Some(cyom) = self.cyom {
                pairs.append_pair("cyom", &cyom.to_string());
            }
            if let Some(uma_resolution_status) = self.uma_resolution_status {
                pairs.append_pair("uma_resolution_status", uma_resolution_status);
            }
            if let Some(game_id) = self.game_id {
                pairs.append_pair("game_id", game_id);
            }
            if let Some(sports_market_types) = &self.sports_market_types {
                for market_type in sports_market_types {
                    pairs.append_pair("sports_market_types", market_type);
                }
            }
            if let Some(rewards_min_size) = self.rewards_min_size {
                pairs.append_pair("rewards_min_size", &rewards_min_size.to_string());
            }
            if let Some(question_ids) = &self.question_ids {
                for question_id in question_ids {
                    pairs.append_pair("question_ids", question_id);
                }
            }
            if let Some(include_tag) = self.include_tag {
                pairs.append_pair("include_tag", &include_tag.to_string());
            }
            if let Some(closed) = self.closed {
                pairs.append_pair("closed", &closed.to_string());
            }
        }
        url
    }
}

// -----------------------------------------------------------------------------
// Client implementation
// -----------------------------------------------------------------------------

impl Client {
    /// Lists markets with optional filters.
    #[instrument(skip(self, request), level = "trace")]
    pub async fn get_markets(&self, request: GetMarketsRequest<'_>) -> Result<Vec<Market>> {
        request.validate()?;
        let url = request.build_url(&self.base_url);
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let markets: Vec<Market> = response.json().await?;
        trace!(count = markets.len(), "received markets");
        Ok(markets)
    }

    /// Gets a market by its ID.
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_market_by_id(&self, id: &str, include_tag: Option<bool>) -> Result<Market> {
        let mut url = self.build_url(&format!("markets/{}", id));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(include_tag) = include_tag {
                pairs.append_pair("include_tag", &include_tag.to_string());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let market: Market = response.json().await?;
        trace!(market_id = %market.id, "received market");
        Ok(market)
    }

    /// Lists tags attached to a market by ID.
    #[instrument(skip(self), fields(id = %id), level = "trace")]
    pub async fn get_market_tags(&self, id: &str) -> Result<Vec<Tag>> {
        let url = self.build_url(&format!("markets/{}/tags", id));
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let tags: Vec<Tag> = response.json().await?;
        trace!(count = tags.len(), "received tags");
        Ok(tags)
    }

    /// Gets a market by its slug.
    #[instrument(skip(self), fields(slug = %slug), level = "trace")]
    pub async fn get_market_by_slug(
        &self,
        slug: &str,
        include_tag: Option<bool>,
    ) -> Result<Market> {
        let mut url = self.build_url(&format!("markets/slug/{}", slug));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(include_tag) = include_tag {
                pairs.append_pair("include_tag", &include_tag.to_string());
            }
        }
        trace!(url = %url, method = "GET", "sending HTTP request");
        let response = self.http_client.get(url).send().await?;
        let response = self.check_response(response).await?;
        let market: Market = response.json().await?;
        trace!(market_id = %market.id, "received market");
        Ok(market)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_invalid_tag_id() {
        let req = GetMarketsRequest {
            tag_id: Some("abc"),
            ..Default::default()
        };

        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("digits"));
    }

    #[test]
    fn build_url_includes_tag_id() {
        let base = Url::parse("https://example.com").unwrap();
        let url = GetMarketsRequest {
            limit: Some(2),
            offset: Some(1),
            tag_id: Some("123"),
            related_tags: Some(true),
            closed: Some(false),
            ..Default::default()
        }
        .build_url(&base);

        let query = url.query().unwrap_or_default();
        for expected in [
            "limit=2",
            "offset=1",
            "tag_id=123",
            "related_tags=true",
            "closed=false",
        ] {
            assert!(
                query.contains(expected),
                "missing '{expected}' in query: {query}"
            );
        }
    }

    #[test]
    fn validate_rejects_invalid_limit() {
        let req = GetMarketsRequest {
            limit: Some(0),
            ..Default::default()
        };

        let err = req.validate().unwrap_err();
        assert!(err.to_string().contains("limit"));
    }
}
