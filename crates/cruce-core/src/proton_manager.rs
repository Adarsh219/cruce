use crate::error::{CruceError, Result};
use crate::paths::{ensure_within_dir, Paths};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tempfile::tempdir;
use xz2::read::XzDecoder;

const GITHUB_API_TIMEOUT: Duration = Duration::from_secs(10);
const DOWNLOAD_STREAM_TIMEOUT: Duration = Duration::from_secs(120);
const DOWNLOAD_CHUNK_SIZE: usize = 32768;

pub type ProgressCallback<'a> = &'a dyn Fn(u8, &str);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledRunner {
    pub name: String,
    pub path: String,
    pub version: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtonRelease {
    pub tag: String,
    pub name: String,
    pub url: String,
    pub published: String,
    pub body: String,
    pub size: u64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct RunnerSourceConfig {
    pub repo: &'static str,
    pub asset_suffix: &'static str,
    pub prefix: &'static str,
}

pub fn get_runner_source(source_id: &str) -> Option<RunnerSourceConfig> {
    match source_id {
        "ge" => Some(RunnerSourceConfig {
            repo: "GloriousEggroll/proton-ge-custom",
            asset_suffix: ".tar.gz",
            prefix: "GE-Proton",
        }),
        "umu" => Some(RunnerSourceConfig {
            repo: "Open-Wine-Components/umu-proton",
            asset_suffix: ".tar.gz",
            prefix: "UMU-Proton",
        }),
        "cachy" => Some(RunnerSourceConfig {
            repo: "CachyOS/proton-cachyos",
            asset_suffix: "-x86_64.tar.xz",
            prefix: "cachyos-",
        }),
        _ => None,
    }
}

pub fn guess_runner_source(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.starts_with("ge-proton") {
        "ge"
    } else if lower.starts_with("umu-proton") {
        "umu"
    } else if lower.contains("cachyos") || lower.starts_with("proton-cachyos") {
        "cachy"
    } else {
        "ge"
    }
}

/// Natural sort key for runner version strings (e.g. `GE-Proton9-25` > `GE-Proton9-9`).
pub fn natural_sort_key(s: &str) -> Vec<(bool, u64, String)> {
    let mut parts = Vec::new();
    let mut current_digits = String::new();
    let mut current_nondigits = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() {
            if !current_nondigits.is_empty() {
                parts.push((false, 0, current_nondigits.clone()));
                current_nondigits.clear();
            }
            current_digits.push(c);
        } else {
            if !current_digits.is_empty() {
                let val = current_digits.parse::<u64>().unwrap_or(0);
                parts.push((true, val, String::new()));
                current_digits.clear();
            }
            current_nondigits.push(c.to_ascii_lowercase());
        }
    }

    if !current_digits.is_empty() {
        let val = current_digits.parse::<u64>().unwrap_or(0);
        parts.push((true, val, String::new()));
    }
    if !current_nondigits.is_empty() {
        parts.push((false, 0, current_nondigits));
    }

    parts
}

/// Safely extract `.tar.gz` or `.tar.xz` archives with directory-traversal / Zip-Slip protection.
pub fn extract_tarball_safe(tar_path: &Path, dest_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(dest_dir).map_err(|e| CruceError::Io {
        path: dest_dir.to_path_buf(),
        source: e,
    })?;

    let file = File::open(tar_path).map_err(|e| CruceError::Io {
        path: tar_path.to_path_buf(),
        source: e,
    })?;

    let filename = tar_path.to_string_lossy().to_ascii_lowercase();
    let reader: Box<dyn Read> = if filename.ends_with(".tar.xz") || filename.ends_with(".xz") {
        Box::new(XzDecoder::new(file))
    } else if filename.ends_with(".tar.gz")
        || filename.ends_with(".tgz")
        || filename.ends_with(".gz")
    {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };

    let mut archive = tar::Archive::new(reader);
    let canonical_dest = dest_dir
        .canonicalize()
        .unwrap_or_else(|_| dest_dir.to_path_buf());

    for entry_result in archive
        .entries()
        .map_err(|e| CruceError::Archive(e.to_string()))?
    {
        let mut entry = entry_result.map_err(|e| CruceError::Archive(e.to_string()))?;
        let entry_path = entry
            .path()
            .map_err(|e| CruceError::Archive(e.to_string()))?
            .into_owned();

        // 1. Path traversal check on entry name
        ensure_within_dir(&canonical_dest, &entry_path)?;

        // 2. Symlink / hardlink target check
        if let Some(link_name) = entry
            .link_name()
            .map_err(|e| CruceError::Archive(e.to_string()))?
        {
            let target = link_name.into_owned();
            if target.is_absolute() {
                return Err(CruceError::Archive(format!(
                    "Tarball contains forbidden absolute symlink target: {}",
                    target.display()
                )));
            }
        }

        entry.unpack_in(&canonical_dest).map_err(|e| {
            CruceError::Archive(format!("Failed to unpack {}: {}", entry_path.display(), e))
        })?;
    }

    // If destination contains exactly 1 directory and no files, return that inner directory
    let mut entries = Vec::new();
    if let Ok(read_dir) = fs::read_dir(&canonical_dest) {
        for e in read_dir.flatten() {
            entries.push(e.path());
        }
    }

    if entries.len() == 1 && entries[0].is_dir() {
        Ok(entries.remove(0))
    } else {
        Ok(canonical_dest)
    }
}

pub struct ProtonManager {
    compat_dir: PathBuf,
    extra_dirs: Vec<PathBuf>,
    client: reqwest::blocking::Client,
}

impl Default for ProtonManager {
    fn default() -> Self {
        Self::new(Paths::steam_compat_dir())
    }
}

impl ProtonManager {
    pub fn new(compat_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&compat_dir);

        let client = reqwest::blocking::Client::builder()
            .timeout(GITHUB_API_TIMEOUT)
            .user_agent("Crucible Launcher/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self {
            compat_dir,
            extra_dirs: Vec::new(),
            client,
        }
    }

    pub fn compat_dir(&self) -> &Path {
        &self.compat_dir
    }

    pub fn add_search_dir(&mut self, path: PathBuf) {
        if !self.extra_dirs.contains(&path) {
            self.extra_dirs.push(path);
        }
    }

    /// Scan `compatibilitytools.d` and extra directories for installed Proton runners.
    pub fn scan_installed(&self) -> Vec<InstalledRunner> {
        let mut installed = Vec::new();
        let mut scan_dirs = vec![self.compat_dir.clone()];
        scan_dirs.extend(self.extra_dirs.clone());

        for scan_dir in scan_dirs {
            let read_dir = match fs::read_dir(&scan_dir) {
                Ok(rd) => rd,
                Err(_) => continue,
            };

            for entry in read_dir.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                // Must contain executable/script named "proton"
                if !path.join("proton").exists() {
                    continue;
                }

                let name = match path.file_name().and_then(|s| s.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };

                let mut version = "unknown".to_string();
                let version_file = path.join("version");
                let alt_version_file = path.join("VERSION");

                if version_file.exists() {
                    if let Ok(content) = fs::read_to_string(&version_file) {
                        version = content.trim().to_string();
                    }
                } else if alt_version_file.exists() {
                    if let Ok(content) = fs::read_to_string(&alt_version_file) {
                        version = content.trim().to_string();
                    }
                }

                let source = guess_runner_source(&name).to_string();

                installed.push(InstalledRunner {
                    name,
                    path: path.to_string_lossy().to_string(),
                    version,
                    source,
                });
            }
        }

        // Natural sort in reverse order (newest versions first)
        installed.sort_by(|a, b| {
            let ka = natural_sort_key(&a.name);
            let kb = natural_sort_key(&b.name);
            kb.cmp(&ka)
        });

        installed
    }

    /// Query GitHub API for available releases from the specified source ("ge", "umu", "cachy").
    pub fn fetch_releases(&self, source_id: &str) -> Result<Vec<ProtonRelease>> {
        let source = get_runner_source(source_id).ok_or_else(|| {
            CruceError::Validation(format!("Unknown Proton runner source: {}", source_id))
        })?;

        let url = format!("https://api.github.com/repos/{}/releases", source.repo);
        let resp = self.client.get(&url).send()?;

        if !resp.status().is_success() {
            return Err(CruceError::Validation(format!(
                "GitHub API returned HTTP {}",
                resp.status()
            )));
        }

        let releases_json: serde_json::Value = resp.json()?;
        let releases_arr = match releases_json.as_array() {
            Some(arr) => arr,
            None => return Ok(Vec::new()),
        };

        let mut results = Vec::new();
        for rel in releases_arr {
            let tag = match rel.get("tag_name").and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => continue,
            };

            let name = rel
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(&tag)
                .to_string();
            let published = rel
                .get("published_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let body = rel
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let mut tar_url = None;
            let mut tar_size = 0;

            if let Some(assets) = rel.get("assets").and_then(|v| v.as_array()) {
                for asset in assets {
                    if let Some(aname) = asset.get("name").and_then(|v| v.as_str()) {
                        if aname.ends_with(source.asset_suffix) {
                            tar_url = asset
                                .get("browser_download_url")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            tar_size = asset.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                            break;
                        }
                    }
                }
            }

            if let Some(url) = tar_url {
                results.push(ProtonRelease {
                    tag,
                    name,
                    url,
                    published,
                    body,
                    size: tar_size,
                    source: source_id.to_string(),
                });
            }
        }

        Ok(results)
    }

    /// Download and install a Proton runner from a GitHub release tag.
    /// Uses an atomic staging directory `.<tag>.<uuid>.installing` to prevent races.
    pub fn download_and_install(
        &self,
        tag: &str,
        source_id: &str,
        cancel_token: Option<&AtomicBool>,
        progress_cb: Option<ProgressCallback>,
    ) -> Result<()> {
        let is_cancelled = || {
            if let Some(token) = cancel_token {
                token.load(Ordering::Relaxed)
            } else {
                false
            }
        };

        if is_cancelled() {
            return Err(CruceError::Cancelled);
        }

        let source = get_runner_source(source_id).ok_or_else(|| {
            CruceError::Validation(format!("Unknown runner source: {}", source_id))
        })?;

        // 1. Fetch release details by tag
        let release_url = format!(
            "https://api.github.com/repos/{}/releases/tags/{}",
            source.repo, tag
        );
        let resp = self.client.get(&release_url).send()?;
        if !resp.status().is_success() {
            return Err(CruceError::Validation(format!(
                "Failed to fetch release tag {}: HTTP {}",
                tag,
                resp.status()
            )));
        }

        let release_json: serde_json::Value = resp.json()?;
        let mut download_url = None;
        let mut asset_size = 0u64;

        if let Some(assets) = release_json.get("assets").and_then(|v| v.as_array()) {
            for asset in assets {
                if let Some(aname) = asset.get("name").and_then(|v| v.as_str()) {
                    if aname.ends_with(source.asset_suffix) {
                        download_url = asset
                            .get("browser_download_url")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        asset_size = asset.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                        break;
                    }
                }
            }
        }

        let url = download_url.ok_or_else(|| {
            CruceError::Validation(format!(
                "No {} asset found for tag {}",
                source.asset_suffix, tag
            ))
        })?;

        // 2. Download archive to temp file with progress callback (0%..70%)
        let tmp_dir = tempdir().map_err(CruceError::IoGeneric)?;
        let suffix = if source.asset_suffix.ends_with(".xz") {
            ".tar.xz"
        } else {
            ".tar.gz"
        };
        let tar_path = tmp_dir.path().join(format!("runner_{}{}", tag, suffix));

        let stream_client = reqwest::blocking::Client::builder()
            .timeout(DOWNLOAD_STREAM_TIMEOUT)
            .user_agent("Crucible Launcher/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        let mut stream_resp = stream_client.get(&url).send()?;
        if !stream_resp.status().is_success() {
            return Err(CruceError::Validation(format!(
                "Download failed with HTTP {}",
                stream_resp.status()
            )));
        }

        let total_size = stream_resp.content_length().unwrap_or(asset_size);

        let mut out_file = File::create(&tar_path).map_err(|e| CruceError::Io {
            path: tar_path.clone(),
            source: e,
        })?;

        let mut downloaded: u64 = 0;
        let mut buf = [0u8; DOWNLOAD_CHUNK_SIZE];

        loop {
            if is_cancelled() {
                let _ = fs::remove_file(&tar_path);
                return Err(CruceError::Cancelled);
            }

            let n = stream_resp.read(&mut buf).map_err(CruceError::IoGeneric)?;
            if n == 0 {
                break;
            }

            out_file.write_all(&buf[..n]).map_err(|e| CruceError::Io {
                path: tar_path.clone(),
                source: e,
            })?;

            downloaded += n as u64;

            if let Some(cb) = progress_cb {
                if total_size > 0 {
                    let percent = ((downloaded as f64 / total_size as f64) * 70.0) as u8;
                    cb(percent.min(70), &format!("Downloading {}...", tag));
                }
            }
        }

        drop(out_file);

        if is_cancelled() {
            let _ = fs::remove_file(&tar_path);
            return Err(CruceError::Cancelled);
        }

        // 3. Extract into unique temporary staging directory (70%..85%)
        if let Some(cb) = progress_cb {
            cb(70, &format!("Extracting {}...", tag));
        }

        let stage_uuid = uuid::Uuid::new_v4().to_string();
        let staging_name = format!(".{}.{}.installing", tag, &stage_uuid[..8]);
        let staged_dir = self.compat_dir.join(&staging_name);

        let temp_extract_dir = tmp_dir.path().join("extract");
        let extracted_root = extract_tarball_safe(&tar_path, &temp_extract_dir)?;

        if is_cancelled() {
            let _ = fs::remove_dir_all(&temp_extract_dir);
            return Err(CruceError::Cancelled);
        }

        // 4. Move extracted content to staging directory (85%..99%)
        if let Some(cb) = progress_cb {
            cb(85, &format!("Installing {}...", tag));
        }

        if extracted_root != temp_extract_dir {
            fs::rename(&extracted_root, &staged_dir).map_err(|e| CruceError::Io {
                path: staged_dir.clone(),
                source: e,
            })?;
        } else {
            fs::create_dir_all(&staged_dir).map_err(|e| CruceError::Io {
                path: staged_dir.clone(),
                source: e,
            })?;
            for entry in fs::read_dir(&temp_extract_dir)
                .map_err(|e| CruceError::Io {
                    path: temp_extract_dir.clone(),
                    source: e,
                })?
                .flatten()
            {
                let dest = staged_dir.join(entry.file_name());
                fs::rename(entry.path(), dest).ok();
            }
        }

        // 5. Atomic replacement into compatibilitytools.d/<tag>
        let final_dir = self.compat_dir.join(tag);
        if final_dir.exists() {
            let _ = fs::remove_dir_all(&final_dir);
        }

        fs::rename(&staged_dir, &final_dir).map_err(|e| CruceError::Io {
            path: final_dir.clone(),
            source: e,
        })?;

        if let Some(cb) = progress_cb {
            cb(100, &format!("Installed {}", tag));
        }

        Ok(())
    }

    /// Delete an installed Proton runner version from `compatibilitytools.d/`.
    pub fn delete_version(&self, tag_or_name: &str) -> Result<()> {
        let target = self.compat_dir.join(tag_or_name);
        if !target.exists() {
            return Err(CruceError::ProtonNotFound(tag_or_name.to_string()));
        }

        fs::remove_dir_all(&target).map_err(|e| CruceError::Io {
            path: target,
            source: e,
        })
    }
}
