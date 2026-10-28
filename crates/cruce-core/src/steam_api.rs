use crate::error::Result;
use crate::paths::GENERIC_DIRS;
use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::Duration;

const STEAMCMD_URL: &str = "https://api.steamcmd.net/v1/info/{app_id}";
const STORE_SEARCH_URL: &str = "https://store.steampowered.com/api/storesearch/";
const APPDETAILS_URL: &str = "https://store.steampowered.com/api/appdetails";
const MIN_CONFIDENCE: u32 = 3;
const PERFECT_SCORE: u32 = 16;

/// Parsed metadata from the SteamCMD PICS API.
#[derive(Debug, Clone, Default)]
pub struct SteamCmdRecord {
    pub installdir: Option<String>,
    pub launch_executables: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct StoreSearchResponse {
    #[serde(default)]
    items: Vec<StoreSearchItem>,
}

#[derive(Debug, Deserialize)]
struct StoreSearchItem {
    id: serde_json::Value,
    #[serde(default)]
    name: String,
}

/// Client for Steam Store and SteamCMD API lookup, identification, and install-dir resolution.
pub struct SteamApi {
    client: reqwest::blocking::Client,
    steamcmd_cache: RwLock<HashMap<String, Option<SteamCmdRecord>>>,
    appdetails_cache: RwLock<HashMap<String, Option<serde_json::Value>>>,
}

impl Default for SteamApi {
    fn default() -> Self {
        Self::new()
    }
}

impl SteamApi {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent("Crucible Launcher/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self {
            client,
            steamcmd_cache: RwLock::new(HashMap::new()),
            appdetails_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Collapse `value` to lowercase alphanumeric tokens separated by single spaces.
    pub fn normalize_name(value: &str) -> String {
        let mut result = String::with_capacity(value.len());
        let mut last_was_space = false;

        for c in value.chars() {
            if c.is_ascii_alphanumeric() {
                result.push(c.to_ascii_lowercase());
                last_was_space = false;
            } else if !last_was_space && !result.is_empty() {
                result.push(' ');
                last_was_space = true;
            }
        }

        if result.ends_with(' ') {
            result.pop();
        }
        result
    }

    /// Collect non-generic ancestor directory names as candidate search terms.
    pub fn ancestor_search_terms(exe_path: &Path, max_depth: usize) -> Vec<String> {
        let mut terms = Vec::new();
        let mut current = match exe_path.parent() {
            Some(p) => p,
            None => return terms,
        };

        let non_word_re = Regex::new(r"[_.-]+").unwrap();

        for _ in 0..max_depth {
            let parent = match current.parent() {
                Some(p) if p != current => p,
                _ => break,
            };

            if let Some(name) = current.file_name().and_then(|s| s.to_str()) {
                let name_lower = name.to_ascii_lowercase();
                if !GENERIC_DIRS.contains(&name_lower.as_str()) {
                    let readable = non_word_re.replace_all(name, " ").trim().to_string();
                    if readable.len() > 2 {
                        terms.push(readable);
                    }
                }
            }

            current = parent;
        }

        terms
    }

    /// Derive de-duplicated search terms and executable name from `exe_path` and `game_name`.
    pub fn exe_candidates(exe_path: &Path, game_name: &str) -> (Vec<String>, Option<String>) {
        let mut exe_name = None;
        let mut raw_terms = Vec::new();

        if !exe_path.as_os_str().is_empty() {
            if let Some(name) = exe_path.file_name().and_then(|s| s.to_str()) {
                exe_name = Some(name.to_string());
            }

            if let Some(stem) = exe_path.file_stem().and_then(|s| s.to_str()) {
                raw_terms.push(stem.to_string());
                let normalized = Self::normalize_name(stem);
                if !normalized.is_empty() && !normalized.eq_ignore_ascii_case(stem) {
                    raw_terms.push(normalized);
                }
            }
        }

        if !game_name.is_empty() {
            raw_terms.push(game_name.to_string());
        }

        if !exe_path.as_os_str().is_empty() {
            raw_terms.extend(Self::ancestor_search_terms(exe_path, 8));
        }

        let mut deduped = Vec::new();
        let mut seen = HashSet::new();

        for term in raw_terms {
            let cleaned = term.trim().to_string();
            if cleaned.is_empty() {
                continue;
            }
            let key = cleaned.to_ascii_lowercase();
            if seen.insert(key) {
                deduped.push(cleaned);
            }
        }

        (deduped, exe_name)
    }

    /// Search the Steam store for `query`, returning `(app_id, title)` pairs.
    pub fn search_candidates(&self, query: &str, limit: usize) -> Result<Vec<(String, String)>> {
        let resp = self
            .client
            .get(STORE_SEARCH_URL)
            .query(&[("term", query), ("l", "english"), ("cc", "US")])
            .send()?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let search_res: StoreSearchResponse = match resp.json() {
            Ok(res) => res,
            Err(_) => return Ok(Vec::new()),
        };

        let mut candidates = Vec::with_capacity(limit);
        for item in search_res.items.into_iter().take(limit) {
            let app_id_str = match item.id {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s,
                _ => continue,
            };
            let name = item.name.trim().to_string();
            if !app_id_str.is_empty() && !name.is_empty() {
                candidates.push((app_id_str, name));
            }
        }

        Ok(candidates)
    }

    /// Fetch SteamCMD PICS configuration for an app ID (cached).
    pub fn fetch_steamcmd_config(&self, app_id: &str) -> Option<SteamCmdRecord> {
        {
            let cache = self.steamcmd_cache.read().unwrap();
            if let Some(cached) = cache.get(app_id) {
                return cached.clone();
            }
        }

        let url = STEAMCMD_URL.replace("{app_id}", app_id);
        let record = self.query_steamcmd_network(&url, app_id);

        let mut cache = self.steamcmd_cache.write().unwrap();
        cache.insert(app_id.to_string(), record.clone());
        record
    }

    fn query_steamcmd_network(&self, url: &str, app_id: &str) -> Option<SteamCmdRecord> {
        let resp = self.client.get(url).send().ok()?;
        if !resp.status().is_success() {
            return None;
        }

        let json: serde_json::Value = resp.json().ok()?;
        let app_data = json.get("data")?.get(app_id)?;

        let config = app_data.get("config");
        let installdir = config
            .and_then(|c| c.get("installdir"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut launch_executables = Vec::new();
        if let Some(launch) = config.and_then(|c| c.get("launch")) {
            let entries: Vec<&serde_json::Value> = match launch {
                serde_json::Value::Array(arr) => arr.iter().collect(),
                serde_json::Value::Object(map) => map.values().collect(),
                _ => Vec::new(),
            };

            for entry in entries {
                if let Some(exe) = entry.get("executable").and_then(|v| v.as_str()) {
                    let normalized_path = exe.replace('\\', "/");
                    if let Some(file_name) = normalized_path.rsplit('/').next() {
                        let lower = file_name.trim().to_ascii_lowercase();
                        if !lower.is_empty() {
                            launch_executables.push(lower);
                        }
                    }
                }
            }
        }

        Some(SteamCmdRecord {
            installdir,
            launch_executables,
        })
    }

    /// Score a candidate: +10 launch executable match, +6 installdir match, +4 exact name, +2 substring.
    pub fn score_candidate(
        &self,
        app_id: &str,
        app_name: &str,
        exe_name_lower: &str,
        exe_path_lower: &str,
        normalized_stem: &str,
        normalized_game: &str,
    ) -> u32 {
        let mut score = 0;

        if let Some(record) = self.fetch_steamcmd_config(app_id) {
            if record
                .launch_executables
                .iter()
                .any(|e| e == exe_name_lower)
            {
                score += 10;
            }

            if let Some(ref installdir) = record.installdir {
                let idir_lower = installdir.trim().to_ascii_lowercase();
                if !idir_lower.is_empty() {
                    let needle = format!("/{}/", idir_lower);
                    if exe_path_lower.contains(&needle) {
                        score += 6;
                    }
                }
            }
        }

        let title_norm = Self::normalize_name(app_name);
        let is_addon = title_norm.contains(" dlc")
            || title_norm.contains(" demo")
            || title_norm.contains(" soundtrack")
            || title_norm.contains(" ost")
            || title_norm.contains(" season pass");

        if title_norm == normalized_stem
            || (!normalized_game.is_empty() && title_norm == normalized_game)
        {
            score += 4;
        } else if !is_addon && !normalized_stem.is_empty() && title_norm.contains(normalized_stem) {
            score += 2;
        }

        score
    }

    /// Search and find the best Steam app match for an executable path and game name.
    pub fn find_app_match_by_exe_name(
        &self,
        exe_path: &Path,
        game_name: &str,
    ) -> Result<Option<(String, String)>> {
        let (terms, exe_name) = Self::exe_candidates(exe_path, game_name);
        let exe_name = match exe_name {
            Some(n) => n,
            None => return Ok(None),
        };

        if terms.is_empty() {
            return Ok(None);
        }

        let exe_name_lower = exe_name.to_ascii_lowercase();
        let exe_path_str = exe_path
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase();
        let exe_path_lower = format!("/{}/", exe_path_str.trim_matches('/'));

        let stem_str = exe_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&exe_name);
        let normalized_stem = Self::normalize_name(stem_str);
        let normalized_game = Self::normalize_name(game_name);

        let mut all_candidates = Vec::new();
        let mut seen_ids = HashSet::new();

        for term in &terms {
            if let Ok(candidates) = self.search_candidates(term, 8) {
                for (id, name) in candidates {
                    if seen_ids.insert(id.clone()) {
                        all_candidates.push((id, name));
                    }
                }
            }
        }

        if all_candidates.is_empty() {
            return Ok(None);
        }

        let mut best_score = 0;
        let mut best = None;

        for (app_id, app_name) in all_candidates {
            let score = self.score_candidate(
                &app_id,
                &app_name,
                &exe_name_lower,
                &exe_path_lower,
                &normalized_stem,
                &normalized_game,
            );

            if score > best_score {
                best_score = score;
                best = Some((app_id, app_name));
            }

            if best_score >= PERFECT_SCORE {
                break;
            }
        }

        if best_score >= MIN_CONFIDENCE {
            Ok(best)
        } else {
            Ok(None)
        }
    }

    /// Fetch Steam store `appdetails` for an app ID (cached).
    pub fn fetch_appdetails(&self, app_id: &str) -> Option<serde_json::Value> {
        {
            let cache = self.appdetails_cache.read().unwrap();
            if let Some(cached) = cache.get(app_id) {
                return cached.clone();
            }
        }

        let resp = self
            .client
            .get(APPDETAILS_URL)
            .query(&[("appids", app_id)])
            .send()
            .ok()?;

        let mut result = None;
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                if let Some(app_obj) = json.get(app_id) {
                    if app_obj.get("success").and_then(|v| v.as_bool()) == Some(true) {
                        result = app_obj.get("data").cloned();
                    }
                }
            }
        }

        let mut cache = self.appdetails_cache.write().unwrap();
        cache.insert(app_id.to_string(), result.clone());
        result
    }

    /// Return `(header_image_url, steam_name)` from Steam store `appdetails`.
    pub fn fetch_header_url(&self, app_id: &str) -> (Option<String>, Option<String>) {
        if let Some(data) = self.fetch_appdetails(app_id) {
            let header = data
                .get("header_image")
                .and_then(|v| v.as_str())
                .map(|url| url.split('?').next().unwrap_or(url).to_string());
            let name = data
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            (header, name)
        } else {
            (None, None)
        }
    }

    /// Walk up from `actual_exe_path` to find an ancestor directory matching `folder_name` (case-insensitive).
    pub fn find_ancestor_by_name(exe_path: &Path, folder_name: &str) -> Option<PathBuf> {
        let target = folder_name.to_ascii_lowercase();
        let mut current = exe_path.parent()?;

        while let Some(parent) = current.parent() {
            if parent == current {
                break;
            }

            if let Some(name) = current.file_name().and_then(|s| s.to_str()) {
                if name.eq_ignore_ascii_case(&target) {
                    return Some(current.to_path_buf());
                }
            }

            current = parent;
        }

        None
    }

    /// Resolve the game installation root directory using the Steam `installdir` value.
    pub fn resolve_install_dir(&self, app_id: &str, actual_exe_path: &Path) -> Option<PathBuf> {
        let record = self.fetch_steamcmd_config(app_id)?;
        let installdir = record.installdir?;
        let root = Self::find_ancestor_by_name(actual_exe_path, installdir.trim())?;
        if root.is_dir() {
            Some(root)
        } else {
            None
        }
    }
}
