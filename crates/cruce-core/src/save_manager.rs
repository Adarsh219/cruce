use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tar::{Archive, Builder};
use tracing::info;

use crate::error::{CruceError, Result};
use crate::paths::{ensure_within_dir, safe_name, Paths};
use crate::storage::format_bytes;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupInfo {
    pub file_path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub size_formatted: String,
    pub timestamp: String,
}

pub struct SaveManager;

impl SaveManager {
    /// Back up save game directories from a Wine prefix into a compressed tar.gz archive.
    /// Common save directories in a Wine prefix:
    /// - drive_c/users/steamuser/Saved Games
    /// - drive_c/users/steamuser/AppData/Local
    /// - drive_c/users/steamuser/AppData/LocalLow
    /// - drive_c/users/steamuser/AppData/Roaming
    /// - drive_c/users/steamuser/Documents
    pub fn backup_saves(prefix_path: &Path, game_name: &str) -> Result<BackupInfo> {
        let safe = safe_name(game_name);
        let backups_dir = Paths::backups_dir().join(&safe);
        fs::create_dir_all(&backups_dir).map_err(CruceError::IoGeneric)?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let archive_filename = format!("{}_{}.tar.gz", safe, now);
        let archive_path = backups_dir.join(&archive_filename);

        let tar_gz = File::create(&archive_path).map_err(CruceError::IoGeneric)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);

        let users_dir = prefix_path.join("drive_c").join("users");
        let mut found_any = false;

        if users_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&users_dir) {
                for entry in entries.flatten() {
                    let user_folder = entry.path();
                    if !user_folder.is_dir() {
                        continue;
                    }

                    // Subdirectories where Windows games store saves
                    let subdirs = [
                        "Saved Games",
                        "AppData/Local",
                        "AppData/LocalLow",
                        "AppData/Roaming",
                        "Documents",
                    ];

                    for sub in &subdirs {
                        let target = user_folder.join(sub);
                        if target.is_dir() {
                            if let Ok(rel) = target.strip_prefix(&users_dir) {
                                if tar.append_dir_all(rel, &target).is_ok() {
                                    found_any = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        if !found_any && users_dir.is_dir() {
            let _ = tar.append_dir_all("users", &users_dir);
        }

        let enc = tar.into_inner().map_err(CruceError::IoGeneric)?;
        enc.finish().map_err(CruceError::IoGeneric)?;

        let metadata = fs::metadata(&archive_path).map_err(CruceError::IoGeneric)?;
        let size_bytes = metadata.len();
        let size_formatted = format_bytes(size_bytes);

        info!(
            "Created save backup for '{}': {} ({})",
            game_name,
            archive_path.display(),
            size_formatted
        );

        Ok(BackupInfo {
            file_path: archive_path.to_string_lossy().to_string(),
            file_name: archive_filename,
            size_bytes,
            size_formatted,
            timestamp: format!("{}", now),
        })
    }

    /// Restore saves from a compressed tar.gz archive into a Wine prefix.
    pub fn restore_saves(prefix_path: &Path, archive_path: &Path) -> Result<()> {
        if !archive_path.is_file() {
            return Err(CruceError::Validation(format!(
                "Backup archive not found: {}",
                archive_path.display()
            )));
        }

        let dest_users_dir = prefix_path.join("drive_c").join("users");
        fs::create_dir_all(&dest_users_dir).map_err(CruceError::IoGeneric)?;

        let file = File::open(archive_path).map_err(CruceError::IoGeneric)?;
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);

        for entry in archive.entries().map_err(CruceError::IoGeneric)? {
            let mut entry = entry.map_err(CruceError::IoGeneric)?;
            let path = entry.path().map_err(CruceError::IoGeneric)?.into_owned();

            let out_path = dest_users_dir.join(&path);
            ensure_within_dir(&dest_users_dir, &out_path)?;

            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(CruceError::IoGeneric)?;
            }
            entry.unpack(&out_path).map_err(CruceError::IoGeneric)?;
        }

        info!(
            "Restored save backup into prefix '{}'",
            prefix_path.display()
        );
        Ok(())
    }

    /// Retrieve the most recent backup info for a game.
    pub fn get_latest_backup(game_name: &str) -> Option<BackupInfo> {
        let safe = safe_name(game_name);
        let backups_dir = Paths::data_dir().join("backups").join(&safe);

        if !backups_dir.is_dir() {
            return None;
        }

        let mut archives: Vec<(PathBuf, u64)> = fs::read_dir(&backups_dir)
            .ok()?
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                if p.is_file() && p.to_string_lossy().ends_with(".tar.gz") {
                    let mtime = e
                        .metadata()
                        .ok()?
                        .modified()
                        .ok()?
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .ok()?
                        .as_secs();
                    Some((p, mtime))
                } else {
                    None
                }
            })
            .collect();

        archives.sort_by_key(|(_p, mtime)| std::cmp::Reverse(*mtime));

        if let Some((latest_path, timestamp)) = archives.first() {
            let size_bytes = fs::metadata(latest_path).map(|m| m.len()).unwrap_or(0);
            let size_formatted = format_bytes(size_bytes);
            Some(BackupInfo {
                file_path: latest_path.to_string_lossy().to_string(),
                file_name: latest_path.file_name()?.to_string_lossy().to_string(),
                size_bytes,
                size_formatted,
                timestamp: format!("{}", timestamp),
            })
        } else {
            None
        }
    }
}
