use md5::{Digest, Md5};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Common game subdirectory names used by `find_game_root()` to walk up past nested binary directories.
pub const GAME_SUBDIR_NAMES: &[&str] = &[
    "binaries",
    "bin",
    "win64",
    "win32",
    "x64",
    "x86",
    "x86_64",
    "windows",
    "engine",
    "game",
    "content",
    "app",
    "apps",
    "build",
    "shipping",
    "development",
    "programs",
];

/// Superset of `GAME_SUBDIR_NAMES` including OS-level and Steam library directory names.
pub const GENERIC_DIRS: &[&str] = &[
    "binaries",
    "bin",
    "win64",
    "win32",
    "x64",
    "x86",
    "x86_64",
    "windows",
    "engine",
    "game",
    "content",
    "app",
    "apps",
    "build",
    "shipping",
    "development",
    "programs",
    "program files",
    "program files (x86)",
    "steamapps",
    "common",
    "games",
    "steam",
    "library",
];

const MAX_GAME_ROOT_DEPTH: usize = 16;
const MAX_APPID_SEARCH_DEPTH: usize = 10;

pub struct Paths;

impl Paths {
    /// Return `~/.local/share/crucible-launcher/`, creating it if needed.
    pub fn data_dir() -> PathBuf {
        let dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("crucible-launcher");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return the game configurations directory (`~/.local/share/crucible-launcher/games/`).
    pub fn games_dir() -> PathBuf {
        let dir = Self::data_dir().join("games");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return the Wine prefixes directory (`~/.local/share/crucible-launcher/Prefix/`).
    pub fn prefixes_dir() -> PathBuf {
        let dir = Self::data_dir().join("Prefix");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return the backups directory (`~/.local/share/crucible-launcher/backups/`).
    pub fn backups_dir() -> PathBuf {
        let dir = Self::data_dir().join("backups");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return the top-level logs directory (`~/.local/share/crucible-launcher/logs/`).
    pub fn logs_dir() -> PathBuf {
        let dir = Self::data_dir().join("logs");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return the application log directory (`logs/app/`).
    pub fn app_logs_dir() -> PathBuf {
        let dir = Self::logs_dir().join("app");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return the per-game log directory (`logs/games/<safe_name>/`).
    pub fn game_logs_dir(game_name: &str) -> PathBuf {
        let dir = Self::logs_dir().join("games").join(safe_name(game_name));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return the artwork cache directory (`~/.local/share/crucible-launcher/artwork/`).
    pub fn artwork_dir() -> PathBuf {
        let dir = Self::data_dir().join("artwork");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return the runner storage directory (`~/.local/share/crucible-launcher/runner/`).
    pub fn runner_dir() -> PathBuf {
        let dir = Self::data_dir().join("runner");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return the hardware fingerprint snapshot directory (`~/.local/share/crucible-launcher/fingerprints/`).
    pub fn fingerprints_dir() -> PathBuf {
        let dir = Self::data_dir().join("fingerprints");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return the user's Steam compatibility tools directory (`~/.steam/steam/compatibilitytools.d/`).
    pub fn steam_compat_dir() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let dir = home
            .join(".steam")
            .join("steam")
            .join("compatibilitytools.d");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Return FreeDesktop applications directory (`~/.local/share/applications/`).
    pub fn applications_dir() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let dir = home.join(".local").join("share").join("applications");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }
}

/// Convert `name` to a lowercase identifier with non-alphanumeric runs replaced by underscores.
/// Matches Python `re.sub(r'[^a-zA-Z0-9]+', '_', name.lower()).strip('_')`.
pub fn safe_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut last_was_underscore = false;

    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            last_was_underscore = false;
        } else if !last_was_underscore && !result.is_empty() {
            result.push('_');
            last_was_underscore = true;
        }
    }

    // Trim trailing underscore
    if result.ends_with('_') {
        result.pop();
    }
    result
}

/// Convert `name` to an artwork-safe filename by replacing spaces/slashes and stripping punctuation.
pub fn artwork_safe_name(name: &str) -> String {
    name.replace([' ', '/'], "_").replace([':', '"', '?'], "")
}

/// Derive a human-readable display name from an executable path.
/// Replaces underscores and hyphens with spaces, then capitalizes each word.
pub fn display_name_from_exe(exe_path: &Path) -> String {
    let stem = exe_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Game");

    stem.replace(['_', '-'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Walk up from `exe_path`, skipping known build/engine subdirs, to find the game root.
pub fn find_game_root(exe_path: &Path) -> Option<PathBuf> {
    if !exe_path.exists() {
        return None;
    }

    let mut current = exe_path.parent()?;
    for _ in 0..MAX_GAME_ROOT_DEPTH {
        let parent = match current.parent() {
            Some(p) if p != current => p,
            _ => break,
        };

        if let Some(name) = current.file_name().and_then(|s| s.to_str()) {
            let lower = name.to_ascii_lowercase();
            if GAME_SUBDIR_NAMES.contains(&lower.as_str()) {
                current = parent;
                continue;
            }
        }

        return Some(current.to_path_buf());
    }

    None
}

/// Search `game_root` for a `steam_appid.txt` file and return its numeric contents.
/// Walks up to 10 levels deep.
pub fn find_app_id_in_game_dir(game_root: &Path) -> Option<String> {
    if !game_root.exists() {
        return None;
    }

    for entry in WalkDir::new(game_root)
        .max_depth(MAX_APPID_SEARCH_DEPTH)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "steam_appid.txt" {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let trimmed = content.trim();
                if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    None
}

/// Derive a stable numeric Steam app ID from the game name if raw_appid is "umu-default".
/// Uses MD5 hash matching Crucible: `int(hashlib.md5(name).hexdigest()[:8], 16) % 900000 + 100000`.
pub fn steam_id_for_name(game_name: &str, raw_appid: &str) -> String {
    if raw_appid != "umu-default" && !raw_appid.is_empty() {
        return raw_appid.to_string();
    }

    let mut hasher = Md5::new();
    hasher.update(game_name.as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    if hex.len() >= 8 {
        if let Ok(val) = u32::from_str_radix(&hex[..8], 16) {
            let id = (val % 900_000) + 100_000;
            return id.to_string();
        }
    }

    "100000".to_string()
}

fn normalize_lexical(path: &Path) -> crate::error::Result<PathBuf> {
    let mut parts = Vec::new();
    let is_absolute = path.is_absolute();

    for comp in path.components() {
        match comp {
            std::path::Component::Prefix(p) => parts.push(p.as_os_str().to_os_string()),
            std::path::Component::RootDir => {
                parts.clear();
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if parts.pop().is_none() {
                    return Err(crate::error::CruceError::Archive(format!(
                        "Path traversal detected: {} escapes directory root",
                        path.display()
                    )));
                }
            }
            std::path::Component::Normal(c) => parts.push(c.to_os_string()),
        }
    }

    let mut result = if is_absolute {
        PathBuf::from("/")
    } else {
        PathBuf::new()
    };
    for p in parts {
        result.push(p);
    }
    Ok(result)
}

/// Ensure `target_path` is safely contained inside `base_dir`, preventing directory traversal (Zip-Slip).
pub fn ensure_within_dir(base_dir: &Path, target_path: &Path) -> crate::error::Result<()> {
    let norm_base = normalize_lexical(base_dir)?;
    let norm_target = if target_path.is_absolute() {
        normalize_lexical(target_path)?
    } else {
        let combined = norm_base.join(target_path);
        normalize_lexical(&combined)?
    };

    if !norm_target.starts_with(&norm_base) {
        return Err(crate::error::CruceError::Archive(format!(
            "Path traversal detected: {} is outside {}",
            target_path.display(),
            base_dir.display()
        )));
    }

    Ok(())
}
