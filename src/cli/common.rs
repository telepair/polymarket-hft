use std::io::{self, Write};

/// Write pretty JSON to stdout using a streaming writer.
pub fn write_json_output<T: serde::Serialize>(value: &T) -> anyhow::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(&mut handle, value)?;
    writeln!(handle)?;
    Ok(())
}
