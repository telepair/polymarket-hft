//! Storage layer module.
//!
//! This module provides data models and utilities for the storage layer,
//! including Redis (hot data) and TimescaleDB (cold data) integration.

pub mod archiver;
pub mod model;

pub use archiver::Archiver;
pub use model::{DataSource, Metric};
