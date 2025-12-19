//! Order utilities for creating and signing Polymarket CLOB orders.
//!
//! This module provides EIP-712 typed data signing for creating exchange orders,
//! referencing:
//! - [Polymarket/go-order-utils](https://github.com/Polymarket/go-order-utils)
//! - [tdergouzi/rs-clob-client](https://github.com/tdergouzi/rs-clob-client)

pub mod builder;
pub mod constants;
pub mod eip712;
pub mod helpers;
pub mod types;

pub use builder::ExchangeOrderBuilder;
pub use types::{Order, OrderData, Side as OrderSide, SignatureType, SignedOrder};
