use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Structured configuration for the Gamescope micro-compositor.
/// Mirrors Heroic's and Crucible's Gamescope settings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GamescopeSettings {
    #[serde(default)]
    pub enable_force_grab_cursor: bool,
    #[serde(default)]
    pub window_type: String, // "fullscreen" | "borderless"
    #[serde(default)]
    pub game_width: String, // inner render width (e.g. "1280")
    #[serde(default)]
    pub game_height: String, // inner render height
    #[serde(default)]
    pub upscale_width: String, // gamescope output width
    #[serde(default)]
    pub upscale_height: String, // gamescope output height
    #[serde(default)]
    pub upscale_method: String, // "fsr" | "nis" | "integer" | "stretch"
    #[serde(default)]
    pub fps_limiter: String, // max FPS when focused
    #[serde(default)]
    pub fps_limiter_no_focus: String, // max FPS when unfocused
    #[serde(default)]
    pub additional_options: String, // raw CLI flags appended verbatim
}

/// Schema for a game configuration record.
/// Persisted to `~/.local/share/crucible-launcher/games/{safe_name}.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub exe_path: String,
    #[serde(default)]
    pub proton_path: String,
    #[serde(default)]
    pub proton_version: String,
    #[serde(default)]
    pub launch_args: String,
    #[serde(default)]
    pub custom_overrides: String,
    #[serde(default)]
    pub install_dir: String,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    #[serde(default)]
    pub disabled_env_vars: Vec<String>,
    #[serde(default)]
    pub prefix_path: String,
    #[serde(default)]
    pub fingerprint_lock: bool,
    #[serde(default)]
    pub wrapper_command: String,
    #[serde(default = "default_exe_match_mode")]
    pub exe_match_mode: String, // "auto" | "custom"

    #[serde(default)]
    pub enable_gamemode: bool,
    #[serde(default)]
    pub enable_mangohud: bool,
    #[serde(default)]
    pub enable_gamescope: bool,
    #[serde(default)]
    pub disabled_global_flags: Vec<String>,
    #[serde(default)]
    pub gamescope_settings: GamescopeSettings,

    #[serde(default)]
    pub playtime_seconds: u64,
    #[serde(default)]
    pub last_played: String, // ISO 8601 timestamp

    /// Runtime-only path pointing to the source JSON file on disk. Never serialized.
    #[serde(skip)]
    pub game_file: Option<String>,
}

fn default_exe_match_mode() -> String {
    "auto".to_string()
}

/// Global configuration with defaults that per-game configs inherit from.
/// Persisted to `~/.local/share/crucible-launcher/global_config.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfigData {
    #[serde(default)]
    pub proton_version: String,
    #[serde(default)]
    pub launch_args: String,
    #[serde(default)]
    pub custom_overrides: String,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    #[serde(default)]
    pub wrapper_command: String,
    #[serde(default)]
    pub fingerprint_lock: bool,

    #[serde(default)]
    pub enable_gamemode: bool,
    #[serde(default)]
    pub enable_mangohud: bool,
    #[serde(default)]
    pub enable_gamescope: bool,
    #[serde(default)]
    pub gamescope_settings: GamescopeSettings,
}

/// Lifecycle states for an individual game process session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameState {
    Idle,
    Launching,
    Running,
    Stopping,
}

impl GameState {
    /// Validates whether a state transition from `self` to `target` is allowed.
    pub fn can_transition_to(self, target: GameState) -> bool {
        match self {
            GameState::Idle => matches!(target, GameState::Launching),
            GameState::Launching => matches!(target, GameState::Running | GameState::Idle),
            GameState::Running => matches!(target, GameState::Stopping | GameState::Idle),
            GameState::Stopping => matches!(target, GameState::Idle),
        }
    }
}
