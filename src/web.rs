//! Web server module for the metrics dashboard.
//!
//! Provides HTTP endpoints for viewing metrics via a browser interface
//! using htmx for reactivity and TailwindCSS for styling.

pub mod handlers;
pub mod templates;

pub use handlers::create_router;
