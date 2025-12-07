//! Data API CLI module.
//!
//! This module provides CLI commands for interacting with the Polymarket Data API.

mod commands;
mod handlers;

pub use commands::DataCommands;
pub use handlers::handle;
