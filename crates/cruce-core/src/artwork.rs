use crate::paths::{artwork_safe_name, Paths};
use crate::steam_api::SteamApi;
use sha1::{Digest, Sha1};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const VARIANT_HEADER: &str = "header";
pub const VARIANT_PORTRAIT: &str = "portrait";
pub const VARIANT_HERO: &str = "hero";

#[derive(Debug, Clone, Default)]
pub struct ArtworkFetchResult {
    pub app_id: Option<String>,
    pub steam_name: Option<String>,
    pub install_dir: Option<PathBuf>,
    pub header_path: Option<PathBuf>,
    pub portrait_path: Option<PathBuf>,
    pub hero_path: Option<PathBuf>,
}

/// Helper for artwork paths, folder indexing, and Steam CDN image downloads.
pub struct ArtworkManager {
    artwork_root: PathBuf,
    client: reqwest::blocking::Client,
}

impl Default for ArtworkManager {
    fn default() -> Self {
        Self::new(Paths::artwork_dir())
    }
}

impl ArtworkManager {
    pub fn new(artwork_root: PathBuf) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(6))
            .user_agent("Crucible Launcher/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self {
            artwork_root,
            client,
        }
    }

    /// Return the artwork cache directory key matching Crucible's convention.
    /// `exe_{sha1[:16]}` for executable paths, `app_{id}` for App IDs, or sanitized game name.
    pub fn artwork_key(exe_path: &str, app_id: Option<&str>, game_name: &str) -> String {
        let trimmed_exe = exe_path.trim();
        if !trimmed_exe.is_empty() {
            let mut hasher = Sha1::new();
            hasher.update(trimmed_exe.to_ascii_lowercase().as_bytes());
            let hex = format!("{:x}", hasher.finalize());
            return format!("exe_{}", &hex[..16]);
        }

        if let Some(id) = app_id {
            let trimmed_id = id.trim();
            if !trimmed_id.is_empty() {
                return format!("app_{}", trimmed_id);
            }
        }

        artwork_safe_name(game_name)
    }

    /// Return the directory where a game's artwork is stored on the filesystem.
    pub fn get_game_artwork_dir(
        &self,
        exe_path: &str,
        app_id: Option<&str>,
        game_name: &str,
    ) -> PathBuf {
        let key = Self::artwork_key(exe_path, app_id, game_name);
        self.artwork_root.join(key)
    }

    /// Return the local filepath for a specific artwork variant (`header`, `portrait`, `hero`).
    pub fn get_artwork_path(
        &self,
        exe_path: &str,
        app_id: Option<&str>,
        game_name: &str,
        variant: &str,
    ) -> PathBuf {
        let dir = self.get_game_artwork_dir(exe_path, app_id, game_name);
        let filename = if variant.is_empty() {
            VARIANT_HEADER
        } else {
            variant
        };
        dir.join(format!("{}.jpg", filename))
    }

    /// Check if a valid cached artwork image file exists (> 500 bytes).
    pub fn has_cached_artwork(
        &self,
        exe_path: &str,
        app_id: Option<&str>,
        game_name: &str,
        variant: &str,
    ) -> Option<PathBuf> {
        let path = self.get_artwork_path(exe_path, app_id, game_name, variant);
        if path.exists() {
            if let Ok(metadata) = fs::metadata(&path) {
                if metadata.len() > 500 {
                    return Some(path);
                }
            }
        }
        None
    }

    /// Download an image from `url` and save to `dest_path`.
    /// Validates status 200 and file size > 500 bytes.
    pub fn download_image(&self, url: &str, dest_path: &Path) -> bool {
        let resp = match self.client.get(url).send() {
            Ok(r) => r,
            Err(_) => return false,
        };

        if !resp.status().is_success() {
            return false;
        }

        let bytes = match resp.bytes() {
            Ok(b) => b,
            Err(_) => return false,
        };

        if bytes.len() <= 500 {
            return false;
        }

        if let Some(parent) = dest_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        fs::write(dest_path, bytes).is_ok()
    }

    /// Rename the cached artwork directory when a game's `exe_path` changes.
    pub fn migrate_artwork(&self, old_exe_path: &str, new_exe_path: &str) -> bool {
        if old_exe_path.is_empty()
            || new_exe_path.is_empty()
            || old_exe_path.eq_ignore_ascii_case(new_exe_path)
        {
            return false;
        }

        let old_dir = self.get_game_artwork_dir(old_exe_path, None, "");
        let new_dir = self.get_game_artwork_dir(new_exe_path, None, "");

        if old_dir.exists() && !new_dir.exists() {
            return fs::rename(&old_dir, &new_dir).is_ok();
        }

        false
    }

    /// Fetch all artwork variants (header, portrait, hero) and resolve metadata for a game.
    pub fn fetch_all_artwork(
        &self,
        steam_api: &SteamApi,
        exe_path: &str,
        game_name: &str,
        initial_app_id: Option<&str>,
    ) -> ArtworkFetchResult {
        let mut result = ArtworkFetchResult::default();
        let mut resolved_id = initial_app_id.map(|s| s.to_string());
        let mut resolved_name = None;

        let exe_p = Path::new(exe_path);

        // 1. Identify Steam App ID if missing
        if resolved_id.is_none() && !exe_path.is_empty() {
            if let Ok(Some((matched_id, matched_name))) =
                steam_api.find_app_match_by_exe_name(exe_p, game_name)
            {
                resolved_id = Some(matched_id);
                resolved_name = Some(matched_name);
            }
        }

        result.app_id = resolved_id.clone();
        result.steam_name = resolved_name;

        // 2. Resolve install dir
        if let Some(ref id) = resolved_id {
            if !exe_path.is_empty() {
                result.install_dir = steam_api.resolve_install_dir(id, exe_p);
            }
        }

        let app_id_str = resolved_id.as_deref();

        // 3. Header Artwork
        let header_path = self.get_artwork_path(exe_path, app_id_str, game_name, VARIANT_HEADER);
        if self
            .has_cached_artwork(exe_path, app_id_str, game_name, VARIANT_HEADER)
            .is_some()
        {
            result.header_path = Some(header_path);
        } else if let Some(ref id) = resolved_id {
            let mut fetched = false;
            let (store_header_url, steam_name) = steam_api.fetch_header_url(id);
            if result.steam_name.is_none() && steam_name.is_some() {
                result.steam_name = steam_name;
            }

            if let Some(ref url) = store_header_url {
                if self.download_image(url, &header_path) {
                    fetched = true;
                    result.header_path = Some(header_path.clone());
                }
            }

            if !fetched {
                let fallbacks = [
                    format!(
                        "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/header.jpg",
                        id
                    ),
                    format!("https://cdn.akamai.steamstatic.com/steam/apps/{}/header.jpg", id),
                ];
                for url in &fallbacks {
                    if self.download_image(url, &header_path) {
                        result.header_path = Some(header_path.clone());
                        break;
                    }
                }
            }
        }

        // 4. Portrait Artwork (600x900)
        let portrait_path =
            self.get_artwork_path(exe_path, app_id_str, game_name, VARIANT_PORTRAIT);
        if self
            .has_cached_artwork(exe_path, app_id_str, game_name, VARIANT_PORTRAIT)
            .is_some()
        {
            result.portrait_path = Some(portrait_path);
        } else if let Some(ref id) = resolved_id {
            let portrait_urls = [
                format!(
                    "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
                    id
                ),
                format!("https://cdn.akamai.steamstatic.com/steam/apps/{}/library_600x900.jpg", id),
                format!(
                    "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900_2x.jpg",
                    id
                ),
            ];
            for url in &portrait_urls {
                if self.download_image(url, &portrait_path) {
                    result.portrait_path = Some(portrait_path.clone());
                    break;
                }
            }
        }

        // 5. Hero Banner Artwork
        let hero_path = self.get_artwork_path(exe_path, app_id_str, game_name, VARIANT_HERO);
        if self
            .has_cached_artwork(exe_path, app_id_str, game_name, VARIANT_HERO)
            .is_some()
        {
            result.hero_path = Some(hero_path);
        } else if let Some(ref id) = resolved_id {
            let hero_urls = [
                format!(
                    "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_hero.jpg",
                    id
                ),
                format!("https://cdn.akamai.steamstatic.com/steam/apps/{}/library_hero.jpg", id),
            ];
            for url in &hero_urls {
                if self.download_image(url, &hero_path) {
                    result.hero_path = Some(hero_path.clone());
                    break;
                }
            }
        }

        result
    }
}
