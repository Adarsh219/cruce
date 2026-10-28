use crate::dll_overrides::build_dll_overrides;
use crate::paths::{find_app_id_in_game_dir, find_game_root, steam_id_for_name, Paths};
use crate::types::GameConfig;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

const DESKTOP_LAUNCH_KEYS: &[&str] = &[
    "DESKTOP_STARTUP_ID",
    "XDG_ACTIVATION_TOKEN",
    "BAMF_DESKTOP_FILE_HINT",
    "GIO_LAUNCHED_DESKTOP_FILE",
    "GIO_LAUNCHED_DESKTOP_FILE_PID",
];

const APPIMAGE_IDENTITY_KEYS: &[&str] = &["APPIMAGE", "APPDIR", "OWD", "ARGV0"];

const APPIMAGE_KEYS: &[&str] = &[
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "PYTHONHOME",
    "PYTHONPATH",
    "QT_PLUGIN_PATH",
    "QT_QPA_PLATFORM_PLUGIN_PATH",
    "QT_QPA_FONTDIR",
    "QTDIR",
    "QT_XKB_CONFIG_ROOT",
];

const APPIMAGE_SAVED_KEYS: &[&str] = &[
    "LD_LIBRARY_PATH",
    "PYTHONHOME",
    "PYTHONPATH",
    "QT_PLUGIN_PATH",
    "PATH",
];

/// Strips AppImage and desktop launch environment variables from `env`.
/// Restores any backed up `CRUCIBLE_ORIG_*` variables so child processes inherit
/// the user's pristine host environment.
pub fn strip_launch_env(env: &mut HashMap<String, String>) {
    let is_appimage = env.contains_key("APPDIR");

    for &k in DESKTOP_LAUNCH_KEYS {
        env.remove(k);
    }
    for &k in APPIMAGE_IDENTITY_KEYS {
        env.remove(k);
    }

    if is_appimage {
        for &key in APPIMAGE_SAVED_KEYS {
            let orig_key = format!("CRUCIBLE_ORIG_{}", key);
            if let Some(orig_val) = env.remove(&orig_key) {
                if !orig_val.is_empty() {
                    env.insert(key.to_string(), orig_val);
                } else {
                    env.remove(key);
                }
            }
        }

        for &key in APPIMAGE_KEYS {
            if !APPIMAGE_SAVED_KEYS.contains(&key) {
                env.remove(key);
            }
        }
    }

    // Clean up any remaining CRUCIBLE_ORIG_ keys
    let orig_keys: Vec<String> = env
        .keys()
        .filter(|k| k.starts_with("CRUCIBLE_ORIG_"))
        .cloned()
        .collect();
    for k in orig_keys {
        env.remove(&k);
    }
}

/// Resolves the Steam App ID for a game config, looking up `steam_appid.txt` in the game root directory.
pub fn resolve_appid(game: &GameConfig) -> String {
    if !game.exe_path.is_empty() {
        let exe = Path::new(&game.exe_path);
        let game_root = find_game_root(exe).or_else(|| exe.parent().map(|p| p.to_path_buf()));
        if let Some(root) = game_root {
            if let Some(app_id) = find_app_id_in_game_dir(&root) {
                return app_id;
            }
        }
    }
    "umu-default".to_string()
}

/// Assembles the complete environment variable map for a game launch.
pub fn build_env(
    game: &GameConfig,
    proton_path: &str,
    prefix_path: &Path,
) -> (HashMap<String, String>, String) {
    let mut env: HashMap<String, String> = std::env::vars()
        .filter(|(k, _)| !k.starts_with("BASH_FUNC"))
        .collect();

    strip_launch_env(&mut env);

    // Apply game overrides
    for (k, v) in &game.env_vars {
        if !k.is_empty() {
            env.insert(k.clone(), v.clone());
        }
    }

    let raw_appid = resolve_appid(game);
    let steam_id = steam_id_for_name(&game.name, &raw_appid);

    env.insert("GAMEID".to_string(), steam_id.clone());
    env.insert("SteamAppId".to_string(), steam_id.clone());
    env.insert("SteamGameId".to_string(), steam_id);

    if env.get("PROTON_LOG").map(|s| s.as_str()) == Some("1") {
        let log_dir = Paths::game_logs_dir(&game.name);
        env.insert("UMU_LOG".to_string(), "1".to_string());
        env.insert(
            "PROTON_LOG_DIR".to_string(),
            log_dir.to_string_lossy().to_string(),
        );
    } else {
        env.remove("UMU_LOG");
        env.remove("PROTON_LOG_DIR");
    }

    env.insert(
        "WINEPREFIX".to_string(),
        prefix_path.to_string_lossy().to_string(),
    );
    env.insert("PROTONPATH".to_string(), proton_path.to_string());
    env.insert("WINEARCH".to_string(), "win64".to_string());
    env.insert("PROTON_VERB".to_string(), "waitforexitandrun".to_string());

    if env.contains_key("LC_ALL") && !env.contains_key("HOST_LC_ALL") {
        if let Some(lc) = env.get("LC_ALL").cloned() {
            env.insert("HOST_LC_ALL".to_string(), lc);
        }
    }

    env.insert(
        "WINEDLLOVERRIDES".to_string(),
        build_dll_overrides(&game.custom_overrides),
    );

    if !env.contains_key("WINEDEBUG") {
        let val = if env.get("PROTON_LOG").map(|s| s.as_str()) == Some("1") {
            ""
        } else {
            "-all"
        };
        env.insert("WINEDEBUG".to_string(), val.to_string());
    }

    if !env.contains_key("DXVK_LOG_LEVEL") {
        let val = if env.get("PROTON_LOG").map(|s| s.as_str()) == Some("1") {
            "info"
        } else {
            "error"
        };
        env.insert("DXVK_LOG_LEVEL".to_string(), val.to_string());
    }

    if !env.contains_key("PULSE_LATENCY_MSEC") {
        env.insert("PULSE_LATENCY_MSEC".to_string(), "60".to_string());
    }

    if env.get("PROTON_ENABLE_NVAPI").map(|s| s.as_str()) == Some("1") {
        env.entry("DXVK_ENABLE_NVAPI".to_string())
            .or_insert_with(|| "1".to_string());
        env.entry("DXVK_NVAPI_ALLOW_OTHER_DRIVERS".to_string())
            .or_insert_with(|| "1".to_string());
    }

    let session_uuid = Uuid::new_v4().to_string();
    env.insert("CRUCIBLE_GAME_ID".to_string(), session_uuid.clone());

    (env, session_uuid)
}
