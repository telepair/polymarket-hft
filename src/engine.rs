//! Data engine for metrics collection and state management.
//!
//! This module provides:
//! - Core types (`Metric`, `StateEntry`)
//! - Conversion traits (`ToMetrics`, `ToState`)
//! - Scrape configuration
//! - Scheduler, Dispatcher

pub mod config;
pub mod dispatcher;
pub mod scheduler;
pub mod types;

pub use config::*;
pub use types::*;
