use crate::error::Result;
use crate::paths::Paths;
use crate::storage::{atomic_write_json, load_json};
use crate::types::{GameConfig, GamescopeSettings, GlobalConfigData};
use std::collections::HashSet;
use std::path::PathBuf;

/// Global configuration controller with atomic persistence and 2-tier inheritance resolution.
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    path: PathBuf,
    data: GlobalConfigData,
}

impl GlobalConfig {
    /// Initialize with a custom path or the default `~/.local/share/crucible-launcher/global_config.json`.
    pub fn new(custom_path: Option<PathBuf>) -> Self {
        let path = custom_path.unwrap_or_else(|| Paths::data_dir().join("global_config.json"));
        let mut config = Self {
            path,
            data: GlobalConfigData::default(),
        };
        config.load();
        config
    }

    /// Load the configuration from disk if it exists.
    pub fn load(&mut self) {
        if self.path.exists() {
            if let Ok(loaded) = load_json::<GlobalConfigData>(&self.path) {
                self.data = loaded;
            }
        }
    }

    /// Save the current configuration to disk atomically.
    pub fn save(&self) -> Result<()> {
        atomic_write_json(&self.path, &self.data)
    }

    /// Borrow the underlying config data.
    pub fn data(&self) -> &GlobalConfigData {
        &self.data
    }

    /// Mutably borrow the underlying config data.
    pub fn data_mut(&mut self) -> &mut GlobalConfigData {
        &mut self.data
    }

    /// Set a new configuration data and persist immediately.
    pub fn update(&mut self, data: GlobalConfigData) -> Result<()> {
        self.data = data;
        self.save()
    }

    /// Resolves a `GameConfig` by overlaying it on top of `GlobalConfigData`.
    ///
    /// Rules matching Crucible:
    /// 1. Scalar strings: If game value is empty, falls through to global default.
    /// 2. Bool flags:
    ///    - If key is listed in `game.disabled_global_flags`, it is explicitly disabled (`false`).
    ///    - Else if game has `true`, it is enabled (`true`).
    ///    - Else, falls through to global default.
    /// 3. `env_vars`: Global map provides base, keys in `disabled_env_vars` are stripped,
    ///    and game's `env_vars` overwrite on top.
    /// 4. `gamescope_settings`: Global provides base, game overrides non-empty fields.
    pub fn resolve(&self, game: &GameConfig) -> GameConfig {
        let mut resolved = game.clone();

        // 1. Scalar strings
        if resolved.proton_version.is_empty() && !self.data.proton_version.is_empty() {
            resolved.proton_version = self.data.proton_version.clone();
        }
        if resolved.launch_args.is_empty() && !self.data.launch_args.is_empty() {
            resolved.launch_args = self.data.launch_args.clone();
        }
        if resolved.custom_overrides.is_empty() && !self.data.custom_overrides.is_empty() {
            resolved.custom_overrides = self.data.custom_overrides.clone();
        }
        if resolved.wrapper_command.is_empty() && !self.data.wrapper_command.is_empty() {
            resolved.wrapper_command = self.data.wrapper_command.clone();
        }

        // 2. Boolean flags
        let disabled_flags: HashSet<&str> = resolved
            .disabled_global_flags
            .iter()
            .map(|s| s.as_str())
            .collect();

        resolved.fingerprint_lock = if disabled_flags.contains("fingerprint_lock") {
            false
        } else if game.fingerprint_lock {
            true
        } else {
            self.data.fingerprint_lock
        };

        resolved.enable_gamemode = if disabled_flags.contains("enable_gamemode") {
            false
        } else if game.enable_gamemode {
            true
        } else {
            self.data.enable_gamemode
        };

        resolved.enable_mangohud = if disabled_flags.contains("enable_mangohud") {
            false
        } else if game.enable_mangohud {
            true
        } else {
            self.data.enable_mangohud
        };

        resolved.enable_gamescope = if disabled_flags.contains("enable_gamescope") {
            false
        } else if game.enable_gamescope {
            true
        } else {
            self.data.enable_gamescope
        };

        // 3. Environment Variables (Base from global, minus disabled, plus game overrides)
        let disabled_envs: HashSet<&str> = resolved
            .disabled_env_vars
            .iter()
            .map(|s| s.as_str())
            .collect();

        let mut merged_envs = self.data.env_vars.clone();
        merged_envs.retain(|k, _| !disabled_envs.contains(k.as_str()));
        for (k, v) in &game.env_vars {
            merged_envs.insert(k.clone(), v.clone());
        }
        resolved.env_vars = merged_envs;

        // 4. Gamescope Settings (merge non-empty fields)
        resolved.gamescope_settings =
            merge_gamescope_settings(&self.data.gamescope_settings, &game.gamescope_settings);

        resolved
    }
}

fn merge_gamescope_settings(
    base: &GamescopeSettings,
    overlay: &GamescopeSettings,
) -> GamescopeSettings {
    let pick_str = |b: &str, o: &str| -> String {
        if !o.is_empty() {
            o.to_string()
        } else {
            b.to_string()
        }
    };

    GamescopeSettings {
        enable_force_grab_cursor: overlay.enable_force_grab_cursor || base.enable_force_grab_cursor,
        window_type: pick_str(&base.window_type, &overlay.window_type),
        game_width: pick_str(&base.game_width, &overlay.game_width),
        game_height: pick_str(&base.game_height, &overlay.game_height),
        upscale_width: pick_str(&base.upscale_width, &overlay.upscale_width),
        upscale_height: pick_str(&base.upscale_height, &overlay.upscale_height),
        upscale_method: pick_str(&base.upscale_method, &overlay.upscale_method),
        fps_limiter: pick_str(&base.fps_limiter, &overlay.fps_limiter),
        fps_limiter_no_focus: pick_str(&base.fps_limiter_no_focus, &overlay.fps_limiter_no_focus),
        additional_options: pick_str(&base.additional_options, &overlay.additional_options),
    }
}
