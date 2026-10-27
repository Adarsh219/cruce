use crate::error::{CruceError, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

/// Format a byte count into a human-readable string (B, KB, MB, GB).
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Atomically writes JSON data to a file by first writing to a temporary file in the
/// same directory and renaming it into place.
pub fn atomic_write_json<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| CruceError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let temp_file = tempfile::Builder::new()
        .prefix(&format!(
            ".{}.",
            path.file_name().and_then(|f| f.to_str()).unwrap_or("tmp")
        ))
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|e| CruceError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;

    {
        let mut writer = BufWriter::new(&temp_file);
        serde_json::to_writer_pretty(&mut writer, data)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }

    temp_file.persist(path).map_err(|e| CruceError::Io {
        path: path.to_path_buf(),
        source: e.error,
    })?;

    Ok(())
}

/// Reads and deserializes a JSON file into type `T`.
pub fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let file = File::open(path).map_err(|e| CruceError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}
