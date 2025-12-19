//! Trading module for CLOB API.
//!
//! Provides order creation, submission, querying, and cancellation.

use alloy_signer_local::PrivateKeySigner;
use serde_json::json;
use tracing::{instrument, trace};

use super::Client;
use super::auth::{create_l2_headers, get_current_timestamp};
use super::types::{
    ApiKeyCreds, Chain, END_CURSOR, INITIAL_CURSOR, OpenOrder, OpenOrderParams, OpenOrdersResponse,
    OrderType, Trade, TradeParams, TradesPaginatedResponse,
};
use crate::error::Result;

// =============================================================================
// Order Submission Endpoints
// =============================================================================

/// Endpoints for trading operations.
mod endpoints {
    pub const POST_ORDER: &str = "/order";
    pub const POST_ORDERS: &str = "/orders";
    pub const GET_ORDER: &str = "/order/";
    pub const GET_OPEN_ORDERS: &str = "/orders";
    pub const GET_TRADES: &str = "/trades";
    pub const CANCEL_ORDER: &str = "/order";
    pub const CANCEL_ORDERS: &str = "/orders";
    pub const CANCEL_ALL: &str = "/cancel-all";
    pub const CANCEL_MARKET_ORDERS: &str = "/cancel-market-orders";
}

// =============================================================================
// Trading Client Extension
// =============================================================================

/// Trading client with wallet and credentials.
#[derive(Debug, Clone)]
pub struct TradingClient {
    /// Inner CLOB client.
    inner: Client,
    /// Chain ID.
    #[allow(dead_code)]
    chain_id: Chain,
    /// Wallet for signing.
    wallet: PrivateKeySigner,
    /// API credentials.
    creds: ApiKeyCreds,
    /// Whether to use server time for signatures.
    use_server_time: bool,
}

impl TradingClient {
    /// Creates a new trading client.
    ///
    /// # Arguments
    ///
    /// * `wallet` - Private key signer for authentication.
    /// * `creds` - API credentials (key, secret, passphrase).
    /// * `chain` - Blockchain network (Polygon or Amoy).
    pub fn new(wallet: PrivateKeySigner, creds: ApiKeyCreds, chain: Chain) -> Self {
        Self {
            inner: Client::new(),
            chain_id: chain,
            wallet,
            creds,
            use_server_time: false,
        }
    }

    /// Creates a trading client with custom base URL.
    pub fn with_base_url(
        base_url: &str,
        wallet: PrivateKeySigner,
        creds: ApiKeyCreds,
        chain: Chain,
    ) -> Result<Self> {
        Ok(Self {
            inner: Client::with_base_url(base_url)?,
            chain_id: chain,
            wallet,
            creds,
            use_server_time: false,
        })
    }

    /// Sets whether to use server time for signatures.
    pub fn with_server_time(mut self, use_server_time: bool) -> Self {
        self.use_server_time = use_server_time;
        self
    }

    /// Returns a reference to the inner client for public API access.
    pub fn client(&self) -> &Client {
        &self.inner
    }

    /// Gets the timestamp for signing (server time or local time).
    async fn get_timestamp(&self) -> Result<String> {
        if self.use_server_time {
            // Could fetch from server, for now use local time
            Ok(get_current_timestamp().to_string())
        } else {
            Ok(get_current_timestamp().to_string())
        }
    }

    // =========================================================================
    // API Key Management (L1)
    // =========================================================================

    /// Creates a new API key using L1 authentication.
    ///
    /// This requires wallet signature but no existing API credentials.
    ///
    /// # Arguments
    ///
    /// * `nonce` - Optional nonce for the signature (defaults to 0).
    ///
    /// # Returns
    ///
    /// Returns new API credentials that can be used for L2 authentication.
    #[instrument(skip(self), level = "trace")]
    pub async fn create_api_key(&self, nonce: Option<u64>) -> Result<super::types::ApiKeyCreds> {
        use super::auth::create_l1_headers;

        let timestamp = self.get_timestamp().await?;
        let headers = create_l1_headers(
            &self.wallet,
            self.chain_id.chain_id(),
            nonce,
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url("/auth/api-key");
        trace!(url = %url, method = "POST", "sending HTTP request");

        let mut request = self.inner.http_client.post(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;
        let raw: super::types::ApiKeyRaw = response.json().await?;
        trace!("API key created successfully");
        Ok(raw.into())
    }

    /// Derives an existing API key using L1 authentication.
    ///
    /// If an API key was previously created with the same nonce, this will
    /// return the same key. This is useful for recovering lost credentials.
    ///
    /// # Arguments
    ///
    /// * `nonce` - Optional nonce for the signature (defaults to 0).
    ///
    /// # Returns
    ///
    /// Returns the derived API credentials.
    #[instrument(skip(self), level = "trace")]
    pub async fn derive_api_key(&self, nonce: Option<u64>) -> Result<super::types::ApiKeyCreds> {
        use super::auth::create_l1_headers;

        let timestamp = self.get_timestamp().await?;
        let headers = create_l1_headers(
            &self.wallet,
            self.chain_id.chain_id(),
            nonce,
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url("/auth/derive-api-key");
        trace!(url = %url, method = "GET", "sending HTTP request");

        let mut request = self.inner.http_client.get(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;
        let raw: super::types::ApiKeyRaw = response.json().await?;
        trace!("API key derived successfully");
        Ok(raw.into())
    }

    /// Creates or derives an API key.
    ///
    /// First attempts to create a new API key. If that fails (key already exists),
    /// it will derive the existing key instead.
    ///
    /// # Arguments
    ///
    /// * `nonce` - Optional nonce for the signature (defaults to 0).
    ///
    /// # Returns
    ///
    /// Returns API credentials (either newly created or derived).
    pub async fn create_or_derive_api_key(
        &self,
        nonce: Option<u64>,
    ) -> Result<super::types::ApiKeyCreds> {
        match self.create_api_key(nonce).await {
            Ok(creds) => Ok(creds),
            Err(_) => self.derive_api_key(nonce).await,
        }
    }

    // =========================================================================
    // API Key Management (L2)
    // =========================================================================

    /// Gets all API keys for the current user.
    ///
    /// Requires L2 authentication.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_api_keys(&self) -> Result<Vec<String>> {
        let endpoint = "/auth/api-keys";
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "GET",
            endpoint,
            None,
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoint);
        trace!(url = %url, method = "GET", "sending HTTP request");

        let mut request = self.inner.http_client.get(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;
        let result: super::types::ApiKeysResponse = response.json().await?;
        trace!(count = result.api_keys.len(), "received API keys");
        Ok(result.api_keys)
    }

    /// Deletes the current API key.
    ///
    /// Requires L2 authentication.
    #[instrument(skip(self), level = "trace")]
    pub async fn delete_api_key(&self) -> Result<()> {
        let endpoint = "/auth/api-key";
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "DELETE",
            endpoint,
            None,
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoint);
        trace!(url = %url, method = "DELETE", "sending HTTP request");

        let mut request = self.inner.http_client.delete(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let _ = self.inner.check_response(response).await?;
        trace!("API key deleted successfully");
        Ok(())
    }

    /// Gets the closed-only (ban) status for the current user.
    ///
    /// Requires L2 authentication.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_closed_only_mode(&self) -> Result<bool> {
        let endpoint = "/auth/ban-status/closed-only";
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "GET",
            endpoint,
            None,
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoint);
        trace!(url = %url, method = "GET", "sending HTTP request");

        let mut request = self.inner.http_client.get(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;
        let result: super::types::BanStatus = response.json().await?;
        trace!(
            closed_only_mode = result.closed_only_mode,
            "received ban status"
        );
        Ok(result.closed_only_mode)
    }

    // =========================================================================
    // Order Submission
    // =========================================================================

    /// Posts a signed order to the exchange.
    ///
    /// # Arguments
    ///
    /// * `order` - Signed order JSON.
    /// * `order_type` - Order type (GTC, FOK, GTD, FAK).
    ///
    /// # Returns
    ///
    /// Returns the API response with order status.
    #[instrument(skip(self, order), level = "trace")]
    pub async fn post_order(
        &self,
        order: serde_json::Value,
        order_type: OrderType,
    ) -> Result<serde_json::Value> {
        let order_payload = json!({
            "order": order,
            "owner": self.creds.key,
            "orderType": order_type,
        });
        let body = serde_json::to_string(&order_payload)?;

        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "POST",
            endpoints::POST_ORDER,
            Some(&body),
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoints::POST_ORDER);
        trace!(url = %url, method = "POST", "sending HTTP request");

        let mut request = self.inner.http_client.post(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        let response = self.inner.check_response(response).await?;
        let result: serde_json::Value = response.json().await?;
        trace!("order posted successfully");
        Ok(result)
    }

    /// Posts multiple signed orders to the exchange.
    #[instrument(skip(self, orders), level = "trace")]
    pub async fn post_orders(
        &self,
        orders: Vec<(serde_json::Value, OrderType)>,
    ) -> Result<serde_json::Value> {
        let payloads: Vec<_> = orders
            .iter()
            .map(|(order, order_type)| {
                json!({
                    "order": order,
                    "owner": self.creds.key,
                    "orderType": order_type,
                    "deferExec": false
                })
            })
            .collect();
        let body = serde_json::to_string(&payloads)?;

        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "POST",
            endpoints::POST_ORDERS,
            Some(&body),
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoints::POST_ORDERS);
        trace!(url = %url, method = "POST", count = orders.len(), "sending HTTP request");

        let mut request = self.inner.http_client.post(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        let response = self.inner.check_response(response).await?;
        let result: serde_json::Value = response.json().await?;
        trace!("orders posted successfully");
        Ok(result)
    }

    // =========================================================================
    // Order Queries
    // =========================================================================

    /// Gets an open order by ID.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_open_order(&self, order_id: &str) -> Result<OpenOrder> {
        let endpoint_path = format!("{}{}", endpoints::GET_ORDER, order_id);
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "GET",
            &endpoint_path,
            None,
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(&endpoint_path);
        trace!(url = %url, method = "GET", "sending HTTP request");

        let mut request = self.inner.http_client.get(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;
        let order: OpenOrder = response.json().await?;
        trace!(order_id = %order.id, "received open order");
        Ok(order)
    }

    /// Gets all open orders.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_open_orders(
        &self,
        params: Option<OpenOrderParams>,
    ) -> Result<OpenOrdersResponse> {
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "GET",
            endpoints::GET_OPEN_ORDERS,
            None,
            Some(timestamp),
        )
        .await?;

        let mut url = self.inner.build_url(endpoints::GET_OPEN_ORDERS);

        if let Some(p) = params {
            if let Some(id) = p.id {
                url.query_pairs_mut().append_pair("id", &id);
            }
            if let Some(market) = p.market {
                url.query_pairs_mut().append_pair("market", &market);
            }
            if let Some(asset_id) = p.asset_id {
                url.query_pairs_mut().append_pair("asset_id", &asset_id);
            }
        }

        trace!(url = %url, method = "GET", "sending HTTP request");

        let mut request = self.inner.http_client.get(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;
        let orders: OpenOrdersResponse = response.json().await?;
        trace!(count = orders.len(), "received open orders");
        Ok(orders)
    }

    /// Gets all trade history with automatic pagination.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_trades(&self, params: Option<TradeParams>) -> Result<Vec<Trade>> {
        let mut results = Vec::new();
        let mut next_cursor = INITIAL_CURSOR.to_string();

        while next_cursor != END_CURSOR {
            let response = self
                .get_trades_paginated(params.clone(), Some(&next_cursor))
                .await?;
            next_cursor = response.next_cursor;
            results.extend(response.data);
        }

        Ok(results)
    }

    /// Gets trades with pagination.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_trades_paginated(
        &self,
        params: Option<TradeParams>,
        cursor: Option<&str>,
    ) -> Result<TradesPaginatedResponse> {
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "GET",
            endpoints::GET_TRADES,
            None,
            Some(timestamp),
        )
        .await?;

        let mut url = self.inner.build_url(endpoints::GET_TRADES);
        url.query_pairs_mut()
            .append_pair("next_cursor", cursor.unwrap_or(INITIAL_CURSOR));

        if let Some(p) = params {
            if let Some(id) = p.id {
                url.query_pairs_mut().append_pair("id", &id);
            }
            if let Some(market) = p.market {
                url.query_pairs_mut().append_pair("market", &market);
            }
            if let Some(asset_id) = p.asset_id {
                url.query_pairs_mut().append_pair("asset_id", &asset_id);
            }
            if let Some(maker) = p.maker_address {
                url.query_pairs_mut().append_pair("maker_address", &maker);
            }
            if let Some(before) = p.before {
                url.query_pairs_mut()
                    .append_pair("before", &before.to_string());
            }
            if let Some(after) = p.after {
                url.query_pairs_mut()
                    .append_pair("after", &after.to_string());
            }
        }

        trace!(url = %url, method = "GET", "sending HTTP request");

        let mut request = self.inner.http_client.get(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;
        let trades: TradesPaginatedResponse = response.json().await?;
        trace!(count = trades.data.len(), "received trades");
        Ok(trades)
    }

    // =========================================================================
    // Order Cancellation
    // =========================================================================

    /// Cancels a single order by ID.
    #[instrument(skip(self), level = "trace")]
    pub async fn cancel_order(&self, order_id: &str) -> Result<serde_json::Value> {
        let payload = json!({ "orderId": order_id });
        let body = serde_json::to_string(&payload)?;

        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "DELETE",
            endpoints::CANCEL_ORDER,
            Some(&body),
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoints::CANCEL_ORDER);
        trace!(url = %url, method = "DELETE", "sending HTTP request");

        let mut request = self.inner.http_client.delete(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        let response = self.inner.check_response(response).await?;
        let result: serde_json::Value = response.json().await?;
        trace!("order cancelled");
        Ok(result)
    }

    /// Cancels multiple orders by IDs.
    #[instrument(skip(self, order_ids), level = "trace")]
    pub async fn cancel_orders(&self, order_ids: Vec<String>) -> Result<serde_json::Value> {
        let payload = json!({ "orderIds": order_ids });
        let body = serde_json::to_string(&payload)?;

        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "DELETE",
            endpoints::CANCEL_ORDERS,
            Some(&body),
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoints::CANCEL_ORDERS);
        trace!(url = %url, method = "DELETE", count = order_ids.len(), "sending HTTP request");

        let mut request = self.inner.http_client.delete(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        let response = self.inner.check_response(response).await?;
        let result: serde_json::Value = response.json().await?;
        trace!("orders cancelled");
        Ok(result)
    }

    /// Cancels all open orders.
    #[instrument(skip(self), level = "trace")]
    pub async fn cancel_all(&self) -> Result<serde_json::Value> {
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "DELETE",
            endpoints::CANCEL_ALL,
            None,
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoints::CANCEL_ALL);
        trace!(url = %url, method = "DELETE", "sending HTTP request");

        let mut request = self.inner.http_client.delete(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;
        let result: serde_json::Value = response.json().await?;
        trace!("all orders cancelled");
        Ok(result)
    }

    /// Cancels orders for a specific market or asset.
    #[instrument(skip(self), level = "trace")]
    pub async fn cancel_market_orders(
        &self,
        market: Option<&str>,
        asset_id: Option<&str>,
    ) -> Result<serde_json::Value> {
        let payload = json!({
            "market": market,
            "asset_id": asset_id,
        });
        let body = serde_json::to_string(&payload)?;

        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "DELETE",
            endpoints::CANCEL_MARKET_ORDERS,
            Some(&body),
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoints::CANCEL_MARKET_ORDERS);
        trace!(url = %url, method = "DELETE", "sending HTTP request");

        let mut request = self.inner.http_client.delete(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        let response = self.inner.check_response(response).await?;
        let result: serde_json::Value = response.json().await?;
        trace!("market orders cancelled");
        Ok(result)
    }

    // =========================================================================
    // Order Creation (using order_utils)
    // =========================================================================

    /// Creates an order builder for this trading client.
    ///
    /// The builder can be used to create multiple orders with the same configuration.
    pub fn order_builder(&self) -> super::order_utils::ExchangeOrderBuilder {
        super::order_utils::ExchangeOrderBuilder::new(
            self.wallet.clone(),
            self.chain_id.chain_id(),
            None,
            None,
        )
    }

    /// Creates a signed limit order.
    ///
    /// # Arguments
    ///
    /// * `order` - User limit order parameters.
    /// * `tick_size` - Tick size for the market.
    /// * `neg_risk` - Whether this is a negative risk market.
    ///
    /// # Returns
    ///
    /// Returns the signed order as JSON ready for submission.
    #[instrument(skip(self, order), level = "trace")]
    pub async fn create_limit_order(
        &self,
        order: &super::types::UserLimitOrder,
        tick_size: super::types::TickSize,
        neg_risk: bool,
    ) -> Result<serde_json::Value> {
        let builder = self.order_builder();
        let signed_order =
            super::order_utils::helpers::create_limit_order(&builder, order, tick_size, neg_risk)
                .await?;
        trace!("created signed limit order");
        Ok(signed_order.to_json())
    }

    /// Creates a signed market order.
    ///
    /// # Arguments
    ///
    /// * `order` - User market order parameters.
    /// * `tick_size` - Tick size for the market.
    /// * `neg_risk` - Whether this is a negative risk market.
    ///
    /// # Returns
    ///
    /// Returns the signed order as JSON ready for submission.
    #[instrument(skip(self, order), level = "trace")]
    pub async fn create_market_order(
        &self,
        order: &super::types::UserMarketOrder,
        tick_size: super::types::TickSize,
        neg_risk: bool,
    ) -> Result<serde_json::Value> {
        let builder = self.order_builder();
        let signed_order =
            super::order_utils::helpers::create_market_order(&builder, order, tick_size, neg_risk)
                .await?;
        trace!("created signed market order");
        Ok(signed_order.to_json())
    }

    /// Creates and posts a limit order in one call.
    ///
    /// # Arguments
    ///
    /// * `order` - User limit order parameters.
    /// * `tick_size` - Tick size for the market.
    /// * `neg_risk` - Whether this is a negative risk market.
    /// * `order_type` - Order type (GTC, FOK, GTD, FAK).
    ///
    /// # Returns
    ///
    /// Returns the API response with order status.
    #[instrument(skip(self, order), level = "trace")]
    pub async fn create_and_post_limit_order(
        &self,
        order: &super::types::UserLimitOrder,
        tick_size: super::types::TickSize,
        neg_risk: bool,
        order_type: OrderType,
    ) -> Result<serde_json::Value> {
        let signed_order = self.create_limit_order(order, tick_size, neg_risk).await?;
        self.post_order(signed_order, order_type).await
    }

    /// Creates and posts a market order in one call.
    ///
    /// # Arguments
    ///
    /// * `order` - User market order parameters.
    /// * `tick_size` - Tick size for the market.
    /// * `neg_risk` - Whether this is a negative risk market.
    /// * `order_type` - Order type (GTC, FOK, GTD, FAK).
    ///
    /// # Returns
    ///
    /// Returns the API response with order status.
    #[instrument(skip(self, order), level = "trace")]
    pub async fn create_and_post_market_order(
        &self,
        order: &super::types::UserMarketOrder,
        tick_size: super::types::TickSize,
        neg_risk: bool,
        order_type: OrderType,
    ) -> Result<serde_json::Value> {
        let signed_order = self.create_market_order(order, tick_size, neg_risk).await?;
        self.post_order(signed_order, order_type).await
    }

    // =========================================================================
    // Balance & Allowance
    // =========================================================================

    /// Gets balance and allowance for the user.
    ///
    /// # Arguments
    ///
    /// * `params` - Balance allowance parameters (asset type and optional token ID).
    ///
    /// # Returns
    ///
    /// Returns the balance and allowance information.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_balance_allowance(
        &self,
        params: super::types::BalanceAllowanceParams,
    ) -> Result<super::types::BalanceAllowance> {
        let endpoint = "/balance-allowance";
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "GET",
            endpoint,
            None,
            Some(timestamp),
        )
        .await?;

        let mut url = self.inner.build_url(endpoint);
        url.query_pairs_mut().append_pair(
            "asset_type",
            &format!("{:?}", params.asset_type).to_uppercase(),
        );
        if let Some(token_id) = &params.token_id {
            url.query_pairs_mut().append_pair("token_id", token_id);
        }

        trace!(url = %url, method = "GET", "sending HTTP request");

        let mut request = self.inner.http_client.get(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;
        let result: super::types::BalanceAllowance = response.json().await?;
        trace!(balance = %result.balance, allowance = %result.allowance, "received balance allowance");
        Ok(result)
    }

    /// Updates balance and allowance cache for the user.
    ///
    /// # Arguments
    ///
    /// * `params` - Balance allowance parameters.
    #[instrument(skip(self), level = "trace")]
    pub async fn update_balance_allowance(
        &self,
        params: super::types::BalanceAllowanceParams,
    ) -> Result<()> {
        let endpoint = "/balance-allowance/update";
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "GET",
            endpoint,
            None,
            Some(timestamp),
        )
        .await?;

        let mut url = self.inner.build_url(endpoint);
        url.query_pairs_mut().append_pair(
            "asset_type",
            &format!("{:?}", params.asset_type).to_uppercase(),
        );
        if let Some(token_id) = &params.token_id {
            url.query_pairs_mut().append_pair("token_id", token_id);
        }

        trace!(url = %url, method = "GET", "sending HTTP request");

        let mut request = self.inner.http_client.get(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let _ = self.inner.check_response(response).await?;
        trace!("balance allowance updated");
        Ok(())
    }

    // =========================================================================
    // Order Scoring
    // =========================================================================

    /// Checks if a specific order is currently scoring (earning rewards).
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID to check.
    ///
    /// # Returns
    ///
    /// Returns true if the order is scoring.
    #[instrument(skip(self), level = "trace")]
    pub async fn is_order_scoring(&self, order_id: &str) -> Result<bool> {
        let endpoint = "/order-scoring";
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "GET",
            endpoint,
            None,
            Some(timestamp),
        )
        .await?;

        let mut url = self.inner.build_url(endpoint);
        url.query_pairs_mut().append_pair("order_id", order_id);

        trace!(url = %url, method = "GET", "sending HTTP request");

        let mut request = self.inner.http_client.get(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;

        #[derive(serde::Deserialize)]
        struct OrderScoringResponse {
            scoring: bool,
        }

        let result: OrderScoringResponse = response.json().await?;
        trace!(scoring = result.scoring, "received order scoring status");
        Ok(result.scoring)
    }

    /// Checks if multiple orders are currently scoring.
    ///
    /// # Arguments
    ///
    /// * `order_ids` - The order IDs to check.
    ///
    /// # Returns
    ///
    /// Returns a map of order_id to scoring status.
    #[instrument(skip(self, order_ids), level = "trace")]
    pub async fn are_orders_scoring(
        &self,
        order_ids: &[String],
    ) -> Result<std::collections::HashMap<String, bool>> {
        let endpoint = "/orders-scoring";
        let body = serde_json::to_string(order_ids)?;
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "POST",
            endpoint,
            Some(&body),
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoint);
        trace!(url = %url, method = "POST", count = order_ids.len(), "sending HTTP request");

        let mut request = self.inner.http_client.post(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        let response = self.inner.check_response(response).await?;
        let result: std::collections::HashMap<String, bool> = response.json().await?;
        trace!(count = result.len(), "received orders scoring status");
        Ok(result)
    }

    // =========================================================================
    // Notifications
    // =========================================================================

    /// Gets notifications for the user.
    ///
    /// # Returns
    ///
    /// Returns a list of notifications.
    #[instrument(skip(self), level = "trace")]
    pub async fn get_notifications(&self) -> Result<Vec<serde_json::Value>> {
        let endpoint = "/notifications";
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "GET",
            endpoint,
            None,
            Some(timestamp),
        )
        .await?;

        let url = self.inner.build_url(endpoint);
        trace!(url = %url, method = "GET", "sending HTTP request");

        let mut request = self.inner.http_client.get(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let response = self.inner.check_response(response).await?;
        let result: Vec<serde_json::Value> = response.json().await?;
        trace!(count = result.len(), "received notifications");
        Ok(result)
    }

    /// Drops (deletes) notifications for the user.
    ///
    /// # Arguments
    ///
    /// * `ids` - Optional list of notification IDs to drop. If None, drops all.
    #[instrument(skip(self), level = "trace")]
    pub async fn drop_notifications(&self, ids: Option<&[String]>) -> Result<()> {
        let endpoint = "/notifications";
        let timestamp = self.get_timestamp().await?;
        let headers = create_l2_headers(
            &self.wallet,
            &self.creds,
            "DELETE",
            endpoint,
            None,
            Some(timestamp),
        )
        .await?;

        let mut url = self.inner.build_url(endpoint);
        if let Some(notification_ids) = ids {
            for id in notification_ids {
                url.query_pairs_mut().append_pair("ids", id);
            }
        }

        trace!(url = %url, method = "DELETE", "sending HTTP request");

        let mut request = self.inner.http_client.delete(url);
        for (key, value) in headers.to_headers() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;
        let _ = self.inner.check_response(response).await?;
        trace!("notifications dropped");
        Ok(())
    }

    // =========================================================================
    // Market Price Calculation
    // =========================================================================

    /// Calculates the optimal market price for a given amount based on the order book.
    ///
    /// This is a client-side calculation that simulates matching against the order book.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The token ID of the market.
    /// * `side` - Buy or Sell side.
    /// * `amount` - The amount to trade (in USDC for buy, shares for sell).
    ///
    /// # Returns
    ///
    /// Returns the calculated price that would fill the order.
    #[instrument(skip(self), level = "trace")]
    pub async fn calculate_market_price(
        &self,
        token_id: &str,
        side: super::pricing::Side,
        amount: f64,
    ) -> Result<f64> {
        use super::pricing::Side;

        let order_book = self.inner.get_order_book(token_id).await?;

        let levels = match side {
            Side::Buy => &order_book.asks,
            Side::Sell => &order_book.bids,
        };

        if levels.is_empty() {
            return Err(crate::error::PolymarketError::bad_request(
                "no liquidity available".to_string(),
            ));
        }

        let mut remaining = amount;
        let mut total_cost = 0.0;
        let mut total_size = 0.0;

        for level in levels {
            let price: f64 = level.price.parse().unwrap_or(0.0);
            let size: f64 = level.size.parse().unwrap_or(0.0);

            if remaining <= 0.0 {
                break;
            }

            match side {
                Side::Buy => {
                    // For buy orders, amount is in USDC
                    let cost_at_level = size * price;
                    if cost_at_level >= remaining {
                        // Partial fill at this level
                        let fill_size = remaining / price;
                        total_size += fill_size;
                        total_cost += remaining;
                        remaining = 0.0;
                    } else {
                        // Full fill at this level
                        total_size += size;
                        total_cost += cost_at_level;
                        remaining -= cost_at_level;
                    }
                }
                Side::Sell => {
                    // For sell orders, amount is in shares
                    if size >= remaining {
                        // Partial fill at this level
                        total_cost += remaining * price;
                        total_size += remaining;
                        remaining = 0.0;
                    } else {
                        // Full fill at this level
                        total_cost += size * price;
                        total_size += size;
                        remaining -= size;
                    }
                }
            }
        }

        if remaining > 0.0 {
            return Err(crate::error::PolymarketError::bad_request(
                "insufficient liquidity".to_string(),
            ));
        }

        // Calculate average price
        let avg_price = if total_size > 0.0 {
            total_cost / total_size
        } else {
            0.0
        };

        trace!(avg_price = avg_price, "calculated market price");
        Ok(avg_price)
    }
}
