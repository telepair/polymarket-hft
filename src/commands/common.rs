use std::io::{self, Write};

use thiserror::Error;

/// Shared CLI error type used by command modules.
#[derive(Error, Debug)]
pub enum CliError {
    #[error("{0}")]
    Sdk(#[from] polymarket_hft::error::PolymarketError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Write pretty JSON to stdout using a streaming writer.
pub fn write_json_output<T: serde::Serialize>(value: &T) -> Result<(), CliError> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(&mut handle, value)?;
    writeln!(handle)?;
    Ok(())
}
