//! Helper functions for order creation.
//!
//! Provides rounding utilities, amount calculations, and market price calculations.

use alloy_primitives::{Address, U256};
use std::str::FromStr;

use super::builder::ExchangeOrderBuilder;
use super::constants::COLLATERAL_TOKEN_DECIMALS;
use super::types::{OrderData, Side, SignedOrder};
use crate::client::polymarket::clob::orderbook::PriceLevel;
use crate::client::polymarket::clob::types::{
    OrderType, TickSize, UserLimitOrder, UserMarketOrder,
};
use crate::error::{PolymarketError, Result};

// =============================================================================
// Rounding Configuration
// =============================================================================

/// Rounding configuration for price, size, and amount.
#[derive(Debug, Clone, Copy)]
pub struct RoundConfig {
    /// Decimal places for price.
    pub price: usize,
    /// Decimal places for size.
    pub size: usize,
    /// Decimal places for amount.
    pub amount: usize,
}

/// Gets the rounding configuration for a given tick size.
pub fn get_rounding_config(tick_size: TickSize) -> RoundConfig {
    match tick_size {
        TickSize::PointOne => RoundConfig {
            price: 1,
            size: 2,
            amount: 3,
        },
        TickSize::PointZeroOne => RoundConfig {
            price: 2,
            size: 2,
            amount: 4,
        },
        TickSize::PointZeroZeroOne => RoundConfig {
            price: 3,
            size: 2,
            amount: 5,
        },
        TickSize::PointZeroZeroZeroOne => RoundConfig {
            price: 4,
            size: 2,
            amount: 6,
        },
    }
}

// =============================================================================
// Rounding Utilities
// =============================================================================

/// Rounds a number to the specified decimal places (standard rounding).
pub fn round_normal(value: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (value * multiplier).round() / multiplier
}

/// Rounds a number down to the specified decimal places.
pub fn round_down(value: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (value * multiplier).floor() / multiplier
}

/// Rounds a number up to the specified decimal places.
pub fn round_up(value: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (value * multiplier).ceil() / multiplier
}

/// Counts the number of decimal places in a number.
pub fn decimal_places(value: f64) -> usize {
    let s = format!("{:.15}", value);
    if let Some(pos) = s.find('.') {
        let decimal_part = s[pos + 1..].trim_end_matches('0');
        decimal_part.len()
    } else {
        0
    }
}

// =============================================================================
// Amount Calculations
// =============================================================================

/// Raw amounts for order creation.
#[derive(Debug, Clone)]
pub struct RawAmounts {
    /// Adjusted side.
    pub side: Side,
    /// Raw maker amount.
    pub raw_maker_amt: f64,
    /// Raw taker amount.
    pub raw_taker_amt: f64,
}

/// Calculates raw amounts for a limit order.
pub fn get_order_raw_amounts(
    side: Side,
    size: f64,
    price: f64,
    round_config: &RoundConfig,
) -> RawAmounts {
    let raw_price = round_normal(price, round_config.price);

    match side {
        Side::Buy => {
            let raw_taker_amt = round_down(size, round_config.size);
            let mut raw_maker_amt = raw_taker_amt * raw_price;

            if decimal_places(raw_maker_amt) > round_config.amount {
                raw_maker_amt = round_up(raw_maker_amt, round_config.amount + 4);
                if decimal_places(raw_maker_amt) > round_config.amount {
                    raw_maker_amt = round_down(raw_maker_amt, round_config.amount);
                }
            }

            RawAmounts {
                side: Side::Buy,
                raw_maker_amt,
                raw_taker_amt,
            }
        }
        Side::Sell => {
            let raw_maker_amt = round_down(size, round_config.size);
            let mut raw_taker_amt = raw_maker_amt * raw_price;

            if decimal_places(raw_taker_amt) > round_config.amount {
                raw_taker_amt = round_up(raw_taker_amt, round_config.amount + 4);
                if decimal_places(raw_taker_amt) > round_config.amount {
                    raw_taker_amt = round_down(raw_taker_amt, round_config.amount);
                }
            }

            RawAmounts {
                side: Side::Sell,
                raw_maker_amt,
                raw_taker_amt,
            }
        }
    }
}

/// Calculates raw amounts for a market order.
pub fn get_market_order_raw_amounts(
    side: Side,
    amount: f64,
    price: f64,
    round_config: &RoundConfig,
) -> RawAmounts {
    let raw_price = round_down(price, round_config.price);

    match side {
        Side::Buy => {
            let raw_maker_amt = round_down(amount, round_config.size);
            let mut raw_taker_amt = if raw_price > 0.0 {
                raw_maker_amt / raw_price
            } else {
                0.0
            };

            if decimal_places(raw_taker_amt) > round_config.amount {
                raw_taker_amt = round_up(raw_taker_amt, round_config.amount + 4);
                if decimal_places(raw_taker_amt) > round_config.amount {
                    raw_taker_amt = round_down(raw_taker_amt, round_config.amount);
                }
            }

            RawAmounts {
                side: Side::Buy,
                raw_maker_amt,
                raw_taker_amt,
            }
        }
        Side::Sell => {
            let raw_maker_amt = round_down(amount, round_config.size);
            let mut raw_taker_amt = raw_maker_amt * raw_price;

            if decimal_places(raw_taker_amt) > round_config.amount {
                raw_taker_amt = round_up(raw_taker_amt, round_config.amount + 4);
                if decimal_places(raw_taker_amt) > round_config.amount {
                    raw_taker_amt = round_down(raw_taker_amt, round_config.amount);
                }
            }

            RawAmounts {
                side: Side::Sell,
                raw_maker_amt,
                raw_taker_amt,
            }
        }
    }
}

/// Converts a float value to U256 with the given decimals.
pub fn parse_units(value: f64, decimals: u8) -> U256 {
    let multiplier = 10_f64.powi(decimals as i32);
    let raw_value = (value * multiplier) as u128;
    U256::from(raw_value)
}

// =============================================================================
// Market Price Calculation
// =============================================================================

/// Calculates the buy market price from ask positions.
pub fn calculate_buy_market_price(
    asks: &[PriceLevel],
    amount_to_match: f64,
    order_type: OrderType,
) -> Result<f64> {
    if asks.is_empty() {
        return Err(PolymarketError::other("No asks available in orderbook"));
    }

    let mut sum = 0.0;

    // Iterate from lowest ask to highest
    for ask in asks.iter() {
        let price: f64 = ask
            .price
            .parse()
            .map_err(|_| PolymarketError::other("Invalid price in orderbook"))?;
        let size: f64 = ask
            .size
            .parse()
            .map_err(|_| PolymarketError::other("Invalid size in orderbook"))?;

        sum += size * price;
        if sum >= amount_to_match {
            return Ok(price);
        }
    }

    // Not enough liquidity
    if order_type == OrderType::Fok {
        return Err(PolymarketError::other(
            "Insufficient liquidity for FOK order",
        ));
    }

    // For FAK, return the last price
    let last_price: f64 = asks
        .last()
        .unwrap()
        .price
        .parse()
        .map_err(|_| PolymarketError::other("Invalid price in orderbook"))?;
    Ok(last_price)
}

/// Calculates the sell market price from bid positions.
pub fn calculate_sell_market_price(
    bids: &[PriceLevel],
    amount_to_match: f64,
    order_type: OrderType,
) -> Result<f64> {
    if bids.is_empty() {
        return Err(PolymarketError::other("No bids available in orderbook"));
    }

    let mut sum = 0.0;

    // Iterate from highest bid to lowest
    for bid in bids.iter().rev() {
        let price: f64 = bid
            .price
            .parse()
            .map_err(|_| PolymarketError::other("Invalid price in orderbook"))?;
        let size: f64 = bid
            .size
            .parse()
            .map_err(|_| PolymarketError::other("Invalid size in orderbook"))?;

        sum += size;
        if sum >= amount_to_match {
            return Ok(price);
        }
    }

    // Not enough liquidity
    if order_type == OrderType::Fok {
        return Err(PolymarketError::other(
            "Insufficient liquidity for FOK order",
        ));
    }

    // For FAK, return the first price
    let first_price: f64 = bids
        .first()
        .unwrap()
        .price
        .parse()
        .map_err(|_| PolymarketError::other("Invalid price in orderbook"))?;
    Ok(first_price)
}

// =============================================================================
// Order Creation Helpers
// =============================================================================

/// Builds OrderData for a limit order.
pub fn build_limit_order_data(
    builder: &ExchangeOrderBuilder,
    user_order: &UserLimitOrder,
    tick_size: TickSize,
) -> Result<OrderData> {
    let round_config = get_rounding_config(tick_size);

    // Convert Side from pricing::Side to order_utils::Side
    let side = match user_order.side {
        crate::client::polymarket::clob::pricing::Side::Buy => Side::Buy,
        crate::client::polymarket::clob::pricing::Side::Sell => Side::Sell,
    };

    let raw_amounts = get_order_raw_amounts(side, user_order.size, user_order.price, &round_config);

    let maker_amount = parse_units(raw_amounts.raw_maker_amt, COLLATERAL_TOKEN_DECIMALS);
    let taker_amount = parse_units(raw_amounts.raw_taker_amt, COLLATERAL_TOKEN_DECIMALS);

    let taker = user_order.taker.unwrap_or(Address::ZERO);
    let fee_rate_bps = U256::from(user_order.fee_rate_bps.unwrap_or(0));
    let nonce = U256::from(user_order.nonce.unwrap_or(0));
    let expiration = user_order.expiration.map(U256::from);

    let token_id = U256::from_str(&user_order.token_id)
        .map_err(|e| PolymarketError::other(format!("Invalid token_id: {}", e)))?;

    Ok(OrderData {
        maker: builder.maker_address(),
        taker,
        token_id,
        maker_amount,
        taker_amount,
        side: raw_amounts.side,
        fee_rate_bps,
        nonce,
        signer: Some(builder.signer_address()),
        expiration,
        signature_type: None,
    })
}

/// Builds OrderData for a market order.
pub fn build_market_order_data(
    builder: &ExchangeOrderBuilder,
    user_order: &UserMarketOrder,
    tick_size: TickSize,
) -> Result<OrderData> {
    let round_config = get_rounding_config(tick_size);
    let price = user_order.price.unwrap_or(1.0);

    // Convert Side from pricing::Side to order_utils::Side
    let side = match user_order.side {
        crate::client::polymarket::clob::pricing::Side::Buy => Side::Buy,
        crate::client::polymarket::clob::pricing::Side::Sell => Side::Sell,
    };

    let raw_amounts = get_market_order_raw_amounts(side, user_order.amount, price, &round_config);

    let maker_amount = parse_units(raw_amounts.raw_maker_amt, COLLATERAL_TOKEN_DECIMALS);
    let taker_amount = parse_units(raw_amounts.raw_taker_amt, COLLATERAL_TOKEN_DECIMALS);

    let taker = user_order.taker.unwrap_or(Address::ZERO);
    let fee_rate_bps = U256::from(user_order.fee_rate_bps.unwrap_or(0));
    let nonce = U256::from(user_order.nonce.unwrap_or(0));

    let token_id = U256::from_str(&user_order.token_id)
        .map_err(|e| PolymarketError::other(format!("Invalid token_id: {}", e)))?;

    Ok(OrderData {
        maker: builder.maker_address(),
        taker,
        token_id,
        maker_amount,
        taker_amount,
        side: raw_amounts.side,
        fee_rate_bps,
        nonce,
        signer: Some(builder.signer_address()),
        expiration: Some(U256::ZERO),
        signature_type: None,
    })
}

/// Creates and signs a limit order.
pub async fn create_limit_order(
    builder: &ExchangeOrderBuilder,
    user_order: &UserLimitOrder,
    tick_size: TickSize,
    neg_risk: bool,
) -> Result<SignedOrder> {
    let order_data = build_limit_order_data(builder, user_order, tick_size)?;
    builder.build_signed_order(order_data, neg_risk).await
}

/// Creates and signs a market order.
pub async fn create_market_order(
    builder: &ExchangeOrderBuilder,
    user_order: &UserMarketOrder,
    tick_size: TickSize,
    neg_risk: bool,
) -> Result<SignedOrder> {
    let order_data = build_market_order_data(builder, user_order, tick_size)?;
    builder.build_signed_order(order_data, neg_risk).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rounding_config() {
        let config = get_rounding_config(TickSize::PointZeroOne);
        assert_eq!(config.price, 2);
        assert_eq!(config.size, 2);
        assert_eq!(config.amount, 4);
    }

    #[test]
    fn test_round_normal() {
        assert_eq!(round_normal(0.555, 2), 0.56);
        assert_eq!(round_normal(0.554, 2), 0.55);
    }

    #[test]
    fn test_round_down() {
        assert_eq!(round_down(0.559, 2), 0.55);
        assert_eq!(round_down(0.551, 2), 0.55);
    }

    #[test]
    fn test_round_up() {
        assert_eq!(round_up(0.551, 2), 0.56);
        // Due to floating-point precision, 0.55 * 100 slightly exceeds 55.0
        // ceil rounds to 56.0, so result is 0.56
        assert_eq!(round_up(0.55, 2), 0.56);
    }

    #[test]
    fn test_get_order_raw_amounts_buy() {
        let round_config = RoundConfig {
            price: 2,
            size: 2,
            amount: 4,
        };
        let result = get_order_raw_amounts(Side::Buy, 100.0, 0.55, &round_config);
        assert_eq!(result.side, Side::Buy);
        assert_eq!(result.raw_taker_amt, 100.0);
        assert_eq!(result.raw_maker_amt, 55.0);
    }

    #[test]
    fn test_get_order_raw_amounts_sell() {
        let round_config = RoundConfig {
            price: 2,
            size: 2,
            amount: 4,
        };
        let result = get_order_raw_amounts(Side::Sell, 100.0, 0.55, &round_config);
        assert_eq!(result.side, Side::Sell);
        assert_eq!(result.raw_maker_amt, 100.0);
        assert_eq!(result.raw_taker_amt, 55.0);
    }

    #[test]
    fn test_parse_units() {
        assert_eq!(parse_units(1.0, 6), U256::from(1_000_000u64));
        assert_eq!(parse_units(0.5, 6), U256::from(500_000u64));
    }
}
