use crate::error::Result;
use image::ImageFormat;
use pelite::PeFile;
use std::fs;
use std::path::Path;
use tracing::debug;

/// Extracts the highest-resolution icon embedded in a Windows `.exe` binary
/// and writes it to `dest_png_path` as a clean PNG image.
///
/// Uses in-memory PE resource extraction with zero external CLI dependencies.
pub fn extract_exe_icon(exe_path: &Path, dest_png_path: &Path) -> Result<bool> {
    if !exe_path.exists() {
        return Ok(false);
    }

    let bytes = match fs::read(exe_path) {
        Ok(b) => b,
        Err(e) => {
            debug!("Failed to read executable for icon extraction: {}", e);
            return Ok(false);
        }
    };

    if let Some(parent) = dest_png_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Try PE file parsing (works for PE32 and PE64)
    if let Ok(pe) = PeFile::from_bytes(&bytes) {
        if let Ok(resources) = pe.resources() {
            let mut best_icon: Option<Vec<u8>> = None;
            let mut best_size: usize = 0;

            for (_name, group) in resources.icons().flatten() {
                for entry in group.entries() {
                    if let Ok(img_data) = group.image(entry.nId) {
                        if img_data.len() > best_size {
                            best_size = img_data.len();
                            best_icon = Some(img_data.to_vec());
                        }
                    }
                }
            }

            if let Some(raw_icon_bytes) = best_icon {
                // Try decoding as PNG directly (many modern Windows icons are embedded PNGs)
                if let Ok(img) =
                    image::load_from_memory_with_format(&raw_icon_bytes, ImageFormat::Png)
                {
                    if img
                        .save_with_format(dest_png_path, ImageFormat::Png)
                        .is_ok()
                    {
                        return Ok(true);
                    }
                }

                // Try decoding as standard ICO format
                if let Ok(img) =
                    image::load_from_memory_with_format(&raw_icon_bytes, ImageFormat::Ico)
                {
                    if img
                        .save_with_format(dest_png_path, ImageFormat::Png)
                        .is_ok()
                    {
                        return Ok(true);
                    }
                }

                // Generic format guess
                if let Ok(img) = image::load_from_memory(&raw_icon_bytes) {
                    if img
                        .save_with_format(dest_png_path, ImageFormat::Png)
                        .is_ok()
                    {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}
