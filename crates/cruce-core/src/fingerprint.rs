use crate::error::{CruceError, Result};
use crate::paths::{safe_name, Paths};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;

/// Manages hardware fingerprint snapshots to freeze `/proc/cpuinfo` and `/proc/version`
/// using bubblewrap (`bwrap`) to prevent Denuvo and anti-tamper triggers across kernel/microcode updates.
pub struct FingerprintManager {
    fingerprints_dir: PathBuf,
}

impl FingerprintManager {
    pub fn new(custom_dir: Option<PathBuf>) -> Self {
        let dir = custom_dir.unwrap_or_else(Paths::fingerprints_dir);
        let _ = fs::create_dir_all(&dir);
        Self {
            fingerprints_dir: dir,
        }
    }

    /// Return the game's snapshot directory.
    pub fn snapshot_dir(&self, game_name: &str) -> PathBuf {
        self.fingerprints_dir.join(safe_name(game_name))
    }

    /// Snapshots `/proc/cpuinfo` and `/proc/version` into the game's snapshot folder.
    pub fn snapshot(&self, game_name: &str) -> Result<()> {
        let snap_dir = self.snapshot_dir(game_name);
        fs::create_dir_all(&snap_dir).map_err(|e| CruceError::Io {
            path: snap_dir.clone(),
            source: e,
        })?;

        let cpuinfo_src = Path::new("/proc/cpuinfo");
        let version_src = Path::new("/proc/version");

        if cpuinfo_src.exists() {
            let data = fs::read(cpuinfo_src).map_err(|e| CruceError::Io {
                path: cpuinfo_src.to_path_buf(),
                source: e,
            })?;
            fs::write(snap_dir.join("cpuinfo"), data).map_err(|e| CruceError::Io {
                path: snap_dir.join("cpuinfo"),
                source: e,
            })?;
        }

        if version_src.exists() {
            let data = fs::read(version_src).map_err(|e| CruceError::Io {
                path: version_src.to_path_buf(),
                source: e,
            })?;
            fs::write(snap_dir.join("version"), data).map_err(|e| CruceError::Io {
                path: snap_dir.join("version"),
                source: e,
            })?;
        }

        Ok(())
    }

    /// Removes the snapshot directory for a game.
    pub fn clear(&self, game_name: &str) {
        let snap_dir = self.snapshot_dir(game_name);
        if snap_dir.exists() {
            let _ = fs::remove_dir_all(snap_dir);
        }
    }

    /// Assembles the bubblewrap (`bwrap`) argument list to bind-mount the snapshot over `/proc`.
    ///
    /// Returns an empty `Vec` if `bwrap` is not installed on the system.
    pub fn get_bwrap_args(&self, game_name: &str) -> Vec<String> {
        let bwrap_bin = match which::which("bwrap") {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => {
                warn!("fingerprint_lock enabled but 'bwrap' not found on PATH — launching without isolation");
                return Vec::new();
            }
        };

        let snap_dir = self.snapshot_dir(game_name);
        let cpuinfo = snap_dir.join("cpuinfo");
        let version = snap_dir.join("version");

        // Take snapshot automatically if absent
        if !cpuinfo.exists() || !version.exists() {
            let _ = self.snapshot(game_name);
        }

        let home_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .to_string_lossy()
            .to_string();

        let mut args = vec![
            bwrap_bin,
            "--ro-bind".to_string(),
            "/".to_string(),
            "/".to_string(),
            "--dev-bind".to_string(),
            "/dev".to_string(),
            "/dev".to_string(),
            "--proc".to_string(),
            "/proc".to_string(),
            "--bind".to_string(),
            "/tmp".to_string(),
            "/tmp".to_string(),
            "--bind".to_string(),
            "/run".to_string(),
            "/run".to_string(),
            "--bind".to_string(),
            home_dir.clone(),
            home_dir,
        ];

        if cpuinfo.exists() {
            args.push("--ro-bind".to_string());
            args.push(cpuinfo.to_string_lossy().to_string());
            args.push("/proc/cpuinfo".to_string());
        }

        if version.exists() {
            args.push("--ro-bind".to_string());
            args.push(version.to_string_lossy().to_string());
            args.push("/proc/version".to_string());
        }

        args.push("--".to_string());
        args
    }
}
