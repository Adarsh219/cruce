use crate::config::GlobalConfig;
use crate::error::{CruceError, Result};
use crate::paths::{find_game_root, safe_name, Paths};
use crate::storage::{atomic_write_json, load_json};
use crate::types::GameConfig;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::error;

/// High-performance store for game configurations located in `~/.local/share/crucible-launcher/games/`.
#[derive(Debug)]
pub struct GameStore {
    games_dir: PathBuf,
    prefixes_dir: PathBuf,
    pub global_config: GlobalConfig,
    games: Vec<GameConfig>,
}

impl GameStore {
    /// Initialize with custom directories (for testing) or system defaults.
    pub fn new(
        games_dir: Option<PathBuf>,
        prefixes_dir: Option<PathBuf>,
        global_config_path: Option<PathBuf>,
    ) -> Self {
        let g_dir = games_dir.unwrap_or_else(Paths::games_dir);
        let p_dir = prefixes_dir.unwrap_or_else(Paths::prefixes_dir);
        let _ = fs::create_dir_all(&g_dir);
        let _ = fs::create_dir_all(&p_dir);

        let mut store = Self {
            games_dir: g_dir,
            prefixes_dir: p_dir,
            global_config: GlobalConfig::new(global_config_path),
            games: Vec::new(),
        };
        store.scan_games();
        store
    }

    /// Read all game JSON files from `games_dir`, populate in-memory games, and sort alphabetically.
    pub fn scan_games(&mut self) -> &[GameConfig] {
        let mut loaded_games = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.games_dir) {
            let mut paths: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|ext| ext.to_str()) == Some("json"))
                .collect();

            paths.sort();

            for game_file in paths {
                match load_json::<GameConfig>(&game_file) {
                    Ok(mut game) => {
                        // Infer install_dir if missing
                        if game.install_dir.is_empty() && !game.exe_path.is_empty() {
                            let exe = Path::new(&game.exe_path);
                            if exe.exists() {
                                let inferred = find_game_root(exe)
                                    .or_else(|| exe.parent().map(|p| p.to_path_buf()));
                                if let Some(root) = inferred {
                                    game.install_dir = root.to_string_lossy().to_string();
                                }
                            }
                        }

                        game.game_file = Some(game_file.to_string_lossy().to_string());
                        loaded_games.push(game);
                    }
                    Err(err) => {
                        error!("Failed to parse game file {}: {}", game_file.display(), err);
                    }
                }
            }
        }

        // Sort alphabetically by name (case-insensitive)
        loaded_games.sort_by_key(|a| a.name.to_lowercase());
        self.games = loaded_games;
        &self.games
    }

    /// Return all cached games in alphabetical order.
    pub fn games(&self) -> &[GameConfig] {
        &self.games
    }

    /// Find a game by its exact display name.
    pub fn get_game(&self, name: &str) -> Option<&GameConfig> {
        self.games.iter().find(|g| g.name == name)
    }

    /// Return game configuration resolved with global defaults.
    pub fn get_resolved_game(&self, name: &str) -> Option<GameConfig> {
        self.get_game(name).map(|g| self.global_config.resolve(g))
    }

    /// Path to the JSON configuration file for a given game name.
    pub fn game_file_path(&self, name: &str) -> PathBuf {
        self.games_dir.join(format!("{}.json", safe_name(name)))
    }

    /// Path to the default Wine prefix for a given game name.
    pub fn default_prefix_path(&self, name: &str) -> PathBuf {
        self.prefixes_dir.join(format!("{}prefix", safe_name(name)))
    }

    /// Add a new game or update an existing one if the name matches.
    pub fn add_game(&mut self, mut game: GameConfig) -> Result<()> {
        let game_file = self.game_file_path(&game.name);

        if game_file.exists() {
            if let Ok(existing) = load_json::<GameConfig>(&game_file) {
                if existing.name != game.name {
                    return Err(CruceError::NameCollision {
                        new_name: game.name,
                        existing_name: existing.name,
                    });
                }
            }
        }

        // Infer install_dir if not set
        if game.install_dir.is_empty() && !game.exe_path.is_empty() {
            let exe = Path::new(&game.exe_path);
            if exe.exists() {
                if let Some(root) =
                    find_game_root(exe).or_else(|| exe.parent().map(|p| p.to_path_buf()))
                {
                    game.install_dir = root.to_string_lossy().to_string();
                }
            }
        }

        atomic_write_json(&game_file, &game)?;
        self.scan_games();
        Ok(())
    }

    /// Update fields on an existing game record and save.
    pub fn update_game<F>(&mut self, name: &str, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut GameConfig),
    {
        let game = self
            .get_game(name)
            .ok_or_else(|| CruceError::GameNotFound(name.to_string()))?;

        let file_path = game
            .game_file
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.game_file_path(name));

        let mut current: GameConfig = load_json(&file_path)?;
        update_fn(&mut current);

        atomic_write_json(&file_path, &current)?;
        self.scan_games();
        Ok(())
    }

    /// Rename a game, moving its JSON file if the `safe_name` changed.
    pub fn rename_game(&mut self, old_name: &str, new_name: &str) -> Result<()> {
        if old_name == new_name {
            return Ok(());
        }

        let game = self
            .get_game(old_name)
            .ok_or_else(|| CruceError::GameNotFound(old_name.to_string()))?;

        let old_file = game
            .game_file
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.game_file_path(old_name));

        let new_file = self.game_file_path(new_name);

        if new_file.exists() && old_file != new_file {
            if let Ok(existing) = load_json::<GameConfig>(&new_file) {
                if existing.name != new_name {
                    return Err(CruceError::NameCollision {
                        new_name: new_name.to_string(),
                        existing_name: existing.name,
                    });
                }
            }
        }

        let mut data: GameConfig = load_json(&old_file)?;
        data.name = new_name.to_string();

        atomic_write_json(&new_file, &data)?;

        if old_file != new_file && old_file.exists() {
            let _ = fs::remove_file(&old_file);
        }

        self.scan_games();
        Ok(())
    }

    /// Remove a game's JSON config and optionally its Wine prefix.
    pub fn remove_game(&mut self, name: &str, remove_prefix: bool) -> Result<()> {
        let game = self
            .get_game(name)
            .ok_or_else(|| CruceError::GameNotFound(name.to_string()))?;

        let file_path = game
            .game_file
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.game_file_path(name));

        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| CruceError::Io {
                path: file_path.clone(),
                source: e,
            })?;
        }

        if remove_prefix {
            let prefix = if !game.prefix_path.is_empty() {
                PathBuf::from(&game.prefix_path)
            } else {
                self.default_prefix_path(name)
            };

            if prefix.exists() {
                let _ = fs::remove_dir_all(&prefix);
            }
        }

        self.scan_games();
        Ok(())
    }

    /// Reset/delete the Wine prefix for a game.
    pub fn reset_game_prefix(&self, name: &str) -> Result<()> {
        let game = self
            .get_game(name)
            .ok_or_else(|| CruceError::GameNotFound(name.to_string()))?;

        let prefix = if !game.prefix_path.is_empty() {
            PathBuf::from(&game.prefix_path)
        } else {
            self.default_prefix_path(name)
        };

        if prefix.exists() {
            fs::remove_dir_all(&prefix).map_err(|e| CruceError::Io {
                path: prefix.clone(),
                source: e,
            })?;
        }

        Ok(())
    }

    /// Record playtime elapsed in seconds and update `last_played` ISO 8601 timestamp.
    pub fn record_playtime(&mut self, name: &str, elapsed_seconds: u64) -> Result<()> {
        let now_iso = chrono_or_simple_timestamp();
        self.update_game(name, |g| {
            g.playtime_seconds += elapsed_seconds;
            g.last_played = now_iso;
        })
    }
}

fn chrono_or_simple_timestamp() -> String {
    // Generate ISO 8601 string: YYYY-MM-DDTHH:MM:SS
    // Using standard time or libc to avoid heavy chrono crate if not needed
    use std::time::SystemTime;
    let now = SystemTime::now();
    let duration = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Approximate ISO 8601 date string
    let tm_sec = secs % 60;
    let tm_min = (secs / 60) % 60;
    let tm_hour = (secs / 3600) % 24;
    let days = secs / 86400;

    // Convert days since 1970-01-01
    let mut year = 1970;
    let mut d = days as i64;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        year += 1;
    }

    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_in_months = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &dim in &days_in_months {
        if d < dim {
            break;
        }
        d -= dim;
        month += 1;
    }
    let day = d + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, tm_hour, tm_min, tm_sec
    )
}
