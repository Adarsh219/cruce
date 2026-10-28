use crate::error::{CruceError, Result};
use crate::paths::ensure_within_dir;
use std::fs::{self, File};
use std::path::Path;

const MAX_TOTAL_EXTRACT_SIZE: u64 = 20 * 1024 * 1024 * 1024; // 20 GB
const MAX_FILE_COUNT: usize = 50_000;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExtractedZipContents {
    pub dlls: Vec<String>,
    pub exes: Vec<String>,
}

/// Safely extract a mod or patch ZIP archive into a destination directory with Zip-Slip path traversal protection.
pub fn extract_zip_mod(zip_path: &Path, dest_dir: &Path) -> Result<ExtractedZipContents> {
    fs::create_dir_all(dest_dir).map_err(|e| CruceError::Io {
        path: dest_dir.to_path_buf(),
        source: e,
    })?;

    let canonical_dest = dest_dir
        .canonicalize()
        .unwrap_or_else(|_| dest_dir.to_path_buf());
    let file = File::open(zip_path).map_err(|e| CruceError::Io {
        path: zip_path.to_path_buf(),
        source: e,
    })?;

    let mut archive = zip::ZipArchive::new(file).map_err(|e| CruceError::Archive(e.to_string()))?;

    if archive.len() > MAX_FILE_COUNT {
        return Err(CruceError::Archive(format!(
            "Archive contains {} entries, exceeding limit of {}",
            archive.len(),
            MAX_FILE_COUNT
        )));
    }

    let mut total_size: u64 = 0;
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| CruceError::Archive(e.to_string()))?;
        total_size += entry.size();
        if total_size > MAX_TOTAL_EXTRACT_SIZE {
            return Err(CruceError::Archive(
                "Archive uncompressed size exceeds limit of 20 GB".to_string(),
            ));
        }
        if let Some(enclosed) = entry.enclosed_name() {
            ensure_within_dir(&canonical_dest, &enclosed)?;
        } else {
            return Err(CruceError::Archive(
                "Archive member has forbidden or absolute path".to_string(),
            ));
        }
    }

    let mut dlls = std::collections::BTreeSet::new();
    let mut exes = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| CruceError::Archive(e.to_string()))?;
        let entry_name = entry
            .name()
            .map_err(|e| CruceError::Archive(e.to_string()))?
            .to_string();
        let target_path = canonical_dest.join(&entry_name);

        if entry.is_dir() {
            let _ = fs::create_dir_all(&target_path);
        } else {
            if let Some(parent) = target_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let mut outfile = File::create(&target_path).map_err(|e| CruceError::Io {
                path: target_path.clone(),
                source: e,
            })?;
            std::io::copy(&mut entry, &mut outfile).map_err(CruceError::IoGeneric)?;

            let lower = entry_name.to_ascii_lowercase();
            if lower.ends_with(".dll") {
                if let Some(stem) = Path::new(&entry_name).file_stem().and_then(|s| s.to_str()) {
                    dlls.insert(stem.to_ascii_lowercase());
                }
            } else if lower.ends_with(".exe") {
                exes.push(target_path.to_string_lossy().to_string());
            }
        }
    }

    exes.sort();

    Ok(ExtractedZipContents {
        dlls: dlls.into_iter().collect(),
        exes,
    })
}
