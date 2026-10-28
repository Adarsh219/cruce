use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use cruce_core::paths::{safe_name, Paths};
use cruce_core::types::{GameConfig, GamescopeSettings, GlobalConfigData};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub fn image_to_data_url(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() < 100 {
        return None;
    }
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jpg")
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "image/jpeg",
    };
    let encoded = STANDARD.encode(&bytes);
    Some(format!("data:{};base64,{}", mime, encoded))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVar {
    pub id: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamescopeConfig {
    pub render_w: String,
    pub render_h: String,
    pub out_w: String,
    pub out_h: String,
    pub method: String,
    pub window_type: String,
    pub grab_cursor: bool,
    pub fps: String,
    pub fps_unfocused: String,
    pub extra: String,
}

impl Default for GamescopeConfig {
    fn default() -> Self {
        Self {
            render_w: "1080".to_string(),
            render_h: "1920".to_string(),
            out_w: "1440".to_string(),
            out_h: "2560".to_string(),
            method: "fsr".to_string(),
            window_type: "borderless".to_string(),
            grab_cursor: false,
            fps: "144".to_string(),
            fps_unfocused: "60".to_string(),
            extra: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphicsConfig {
    pub gamemode: bool,
    pub mangohud: bool,
    pub gamescope: bool,
    pub dlss: bool,
    pub fsr3: bool,
    pub fsr4: bool,
    pub fsr4rdna3: bool,
    pub xess: bool,
    pub gs: GamescopeConfig,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            gamemode: true,
            mangohud: false,
            gamescope: false,
            dlss: false,
            fsr3: false,
            fsr4: false,
            fsr4rdna3: false,
            xess: false,
            gs: GamescopeConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WineConfig {
    pub nvapi: bool,
    pub wayland: bool,
    pub hdr: bool,
    pub wow64: bool,
    pub wined3d: bool,
    pub log: bool,
    pub no_steam_client: bool,
    pub skip_runtime_update: bool,
    pub bwrap_lock: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchConfig {
    pub args: String,
    pub dll_overrides: String,
    pub wrapper: String,
    pub env: Vec<EnvVar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub title: String,
    pub exe: String,
    pub install_dir: String,
    pub prefix: String,
    pub runner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hero: Option<String>,
    pub hue: u32,
    pub playtime_min: u32,
    pub last_played_days_ago: Option<u32>,
    pub status: String, // "idle" | "launching" | "running"
    pub shortcut: bool,
    pub prefix_size: Option<String>,
    pub launch: LaunchConfig,
    pub graphics: GraphicsConfig,
    pub wine: WineConfig,
}

impl Game {
    pub fn from_config(
        cfg: &GameConfig,
        status: &str,
        shortcut: bool,
        prefix_size: Option<String>,
    ) -> Self {
        let sname = safe_name(&cfg.name);

        let prefix = if cfg.prefix_path.is_empty() {
            Paths::prefixes_dir()
                .join(format!("{}prefix", sname))
                .to_string_lossy()
                .to_string()
        } else {
            cfg.prefix_path.clone()
        };

        let mut hasher = Md5::new();
        hasher.update(cfg.name.as_bytes());
        let digest = hasher.finalize();
        let hue = ((digest[0] as u32) * 360) / 256;

        let playtime_min = (cfg.playtime_seconds / 60) as u32;

        let last_played_days_ago = if !cfg.last_played.is_empty() {
            // Simple heuristic or default to 0/None
            Some(0)
        } else {
            None
        };

        let mut env_list = Vec::new();
        let mut idx = 0;
        for (k, v) in &cfg.env_vars {
            if !k.starts_with("PROTON_")
                && !k.starts_with("UMU_")
                && k != "WINEPREFIX"
                && k != "GAMEID"
            {
                idx += 1;
                env_list.push(EnvVar {
                    id: format!("env-{}", idx),
                    key: k.clone(),
                    value: v.clone(),
                });
            }
        }

        let is_nvapi = cfg
            .env_vars
            .get("PROTON_ENABLE_NVAPI")
            .map(|v| v == "1")
            .unwrap_or(false);
        let is_wayland = cfg
            .env_vars
            .get("PROTON_ENABLE_WAYLAND")
            .map(|v| v == "1")
            .unwrap_or(false);
        let is_hdr = cfg
            .env_vars
            .get("PROTON_ENABLE_HDR")
            .map(|v| v == "1")
            .unwrap_or(false);
        let is_wow64 = cfg
            .env_vars
            .get("PROTON_USE_WOW64")
            .map(|v| v == "1")
            .unwrap_or(false);
        let is_wined3d = cfg
            .env_vars
            .get("PROTON_USE_WINED3D")
            .map(|v| v == "1")
            .unwrap_or(false);
        let is_log = cfg
            .env_vars
            .get("PROTON_LOG")
            .map(|v| v == "1")
            .unwrap_or(false);
        let is_no_steam = cfg
            .env_vars
            .get("PROTON_NO_STEAMCLIENT")
            .map(|v| v == "1")
            .unwrap_or(false);
        let is_skip_runtime = cfg
            .env_vars
            .get("UMU_RUNTIME_UPDATE")
            .map(|v| v == "0")
            .unwrap_or(false);

        let gs = GamescopeConfig {
            render_w: if cfg.gamescope_settings.game_width.is_empty() {
                "1080".into()
            } else {
                cfg.gamescope_settings.game_width.clone()
            },
            render_h: if cfg.gamescope_settings.game_height.is_empty() {
                "1920".into()
            } else {
                cfg.gamescope_settings.game_height.clone()
            },
            out_w: if cfg.gamescope_settings.upscale_width.is_empty() {
                "1440".into()
            } else {
                cfg.gamescope_settings.upscale_width.clone()
            },
            out_h: if cfg.gamescope_settings.upscale_height.is_empty() {
                "2560".into()
            } else {
                cfg.gamescope_settings.upscale_height.clone()
            },
            method: if cfg.gamescope_settings.upscale_method.is_empty() {
                "fsr".into()
            } else {
                cfg.gamescope_settings.upscale_method.clone()
            },
            window_type: if cfg.gamescope_settings.window_type.is_empty() {
                "borderless".into()
            } else {
                cfg.gamescope_settings.window_type.clone()
            },
            grab_cursor: cfg.gamescope_settings.enable_force_grab_cursor,
            fps: if cfg.gamescope_settings.fps_limiter.is_empty() {
                "144".into()
            } else {
                cfg.gamescope_settings.fps_limiter.clone()
            },
            fps_unfocused: if cfg.gamescope_settings.fps_limiter_no_focus.is_empty() {
                "60".into()
            } else {
                cfg.gamescope_settings.fps_limiter_no_focus.clone()
            },
            extra: cfg.gamescope_settings.additional_options.clone(),
        };

        // Artwork candidates: sha1 key, safe name key, game name, or fallback to header / icon
        let artwork_dir = Paths::artwork_dir();
        let safe = safe_name(&cfg.name);
        let exe_key =
            cruce_core::artwork::ArtworkManager::artwork_key(&cfg.exe_path, None, &cfg.name);

        let cover_candidates = [
            artwork_dir.join(&exe_key).join("portrait.jpg"),
            artwork_dir
                .join(format!("exe_{}", safe))
                .join("portrait.jpg"),
            artwork_dir.join(&safe).join("portrait.jpg"),
            artwork_dir.join(&exe_key).join("header.jpg"),
            artwork_dir.join(format!("exe_{}", safe)).join("header.jpg"),
            artwork_dir.join(&safe).join("header.jpg"),
            artwork_dir.join(&exe_key).join("icon.png"),
            artwork_dir.join(format!("exe_{}", safe)).join("icon.png"),
        ];

        let cover = cover_candidates.iter().find_map(|p| image_to_data_url(p));

        let hero_candidates = [
            artwork_dir.join(&exe_key).join("hero.jpg"),
            artwork_dir.join(format!("exe_{}", safe)).join("hero.jpg"),
            artwork_dir.join(&safe).join("hero.jpg"),
            artwork_dir.join(&exe_key).join("header.jpg"),
            artwork_dir.join(format!("exe_{}", safe)).join("header.jpg"),
            artwork_dir.join(&safe).join("header.jpg"),
        ];

        let hero = hero_candidates.iter().find_map(|p| image_to_data_url(p));

        Self {
            id: sname,
            title: cfg.name.clone(),
            exe: cfg.exe_path.clone(),
            install_dir: cfg.install_dir.clone(),
            prefix,
            runner: cfg.proton_version.clone(),
            cover,
            hero,
            hue,
            playtime_min,
            last_played_days_ago,
            status: status.to_string(),
            shortcut,
            prefix_size,
            launch: LaunchConfig {
                args: cfg.launch_args.clone(),
                dll_overrides: cfg.custom_overrides.clone(),
                wrapper: cfg.wrapper_command.clone(),
                env: env_list,
            },
            graphics: GraphicsConfig {
                gamemode: cfg.enable_gamemode,
                mangohud: cfg.enable_mangohud,
                gamescope: cfg.enable_gamescope,
                dlss: is_nvapi,
                fsr3: false,
                fsr4: false,
                fsr4rdna3: false,
                xess: false,
                gs,
            },
            wine: WineConfig {
                nvapi: is_nvapi,
                wayland: is_wayland,
                hdr: is_hdr,
                wow64: is_wow64,
                wined3d: is_wined3d,
                log: is_log,
                no_steam_client: is_no_steam,
                skip_runtime_update: is_skip_runtime,
                bwrap_lock: cfg.fingerprint_lock,
            },
        }
    }

    pub fn to_game_config(&self, existing: Option<&GameConfig>) -> GameConfig {
        let mut cfg = existing.cloned().unwrap_or_default();
        cfg.name = self.title.clone();
        cfg.exe_path = self.exe.clone();
        cfg.install_dir = self.install_dir.clone();
        cfg.prefix_path = self.prefix.clone();
        cfg.proton_version = self.runner.clone();
        cfg.launch_args = self.launch.args.clone();
        cfg.custom_overrides = self.launch.dll_overrides.clone();
        cfg.wrapper_command = self.launch.wrapper.clone();

        cfg.enable_gamemode = self.graphics.gamemode;
        cfg.enable_mangohud = self.graphics.mangohud;
        cfg.enable_gamescope = self.graphics.gamescope;
        cfg.fingerprint_lock = self.wine.bwrap_lock;

        cfg.gamescope_settings = GamescopeSettings {
            enable_force_grab_cursor: self.graphics.gs.grab_cursor,
            window_type: self.graphics.gs.window_type.clone(),
            game_width: self.graphics.gs.render_w.clone(),
            game_height: self.graphics.gs.render_h.clone(),
            upscale_width: self.graphics.gs.out_w.clone(),
            upscale_height: self.graphics.gs.out_h.clone(),
            upscale_method: self.graphics.gs.method.clone(),
            fps_limiter: self.graphics.gs.fps.clone(),
            fps_limiter_no_focus: self.graphics.gs.fps_unfocused.clone(),
            additional_options: self.graphics.gs.extra.clone(),
        };

        let mut env_map = HashMap::new();
        for item in &self.launch.env {
            if !item.key.trim().is_empty() {
                env_map.insert(item.key.trim().to_string(), item.value.clone());
            }
        }

        if self.wine.nvapi {
            env_map.insert("PROTON_ENABLE_NVAPI".into(), "1".into());
        }
        if self.wine.wayland {
            env_map.insert("PROTON_ENABLE_WAYLAND".into(), "1".into());
        }
        if self.wine.hdr {
            env_map.insert("PROTON_ENABLE_HDR".into(), "1".into());
        }
        if self.wine.wow64 {
            env_map.insert("PROTON_USE_WOW64".into(), "1".into());
        }
        if self.wine.wined3d {
            env_map.insert("PROTON_USE_WINED3D".into(), "1".into());
        }
        if self.wine.log {
            env_map.insert("PROTON_LOG".into(), "1".into());
        }
        if self.wine.no_steam_client {
            env_map.insert("PROTON_NO_STEAMCLIENT".into(), "1".into());
        }
        if self.wine.skip_runtime_update {
            env_map.insert("UMU_RUNTIME_UPDATE".into(), "0".into());
        }

        cfg.env_vars = env_map;
        cfg.playtime_seconds = (self.playtime_min as u64) * 60;
        cfg
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Runner {
    pub id: String,
    pub name: String,
    pub source: String,
    pub version: String,
    pub size: String,
    pub date: String,
    pub installed: bool,
    #[serde(default)]
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub minimize_to_tray: bool,
    pub remember_geometry: bool,
    pub auto_update_umu: bool,
    pub font: String,
    pub theme: String,
    pub proton_dir: String,
    pub default_runner: String,
    pub default_args: String,
    pub default_dll: String,
    pub default_wrapper: String,
    pub default_gamemode: bool,
    pub default_mangohud: bool,
    pub default_gamescope: bool,
    pub log_level: String,
}

impl From<&GlobalConfigData> for AppSettings {
    fn from(data: &GlobalConfigData) -> Self {
        Self {
            minimize_to_tray: true,
            remember_geometry: true,
            auto_update_umu: true,
            font: "geist".to_string(),
            theme: "crucible".to_string(),
            proton_dir: Paths::steam_compat_dir().to_string_lossy().to_string(),
            default_runner: data.proton_version.clone(),
            default_args: data.launch_args.clone(),
            default_dll: data.custom_overrides.clone(),
            default_wrapper: data.wrapper_command.clone(),
            default_gamemode: data.enable_gamemode,
            default_mangohud: data.enable_mangohud,
            default_gamescope: data.enable_gamescope,
            log_level: "info".to_string(),
        }
    }
}

impl AppSettings {
    pub fn to_global_config_data(&self, existing: Option<&GlobalConfigData>) -> GlobalConfigData {
        let mut data = existing.cloned().unwrap_or_default();
        data.proton_version = self.default_runner.clone();
        data.launch_args = self.default_args.clone();
        data.custom_overrides = self.default_dll.clone();
        data.wrapper_command = self.default_wrapper.clone();
        data.enable_gamemode = self.default_gamemode;
        data.enable_mangohud = self.default_mangohud;
        data.enable_gamescope = self.default_gamescope;
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_roundtrip_conversion() {
        let mut cfg = GameConfig {
            name: "Cyberpunk 2077".to_string(),
            exe_path: "/games/cyberpunk/bin/x64/Cyberpunk2077.exe".to_string(),
            install_dir: "/games/cyberpunk".to_string(),
            prefix_path: "/prefixes/cyberpunk".to_string(),
            proton_version: "GE-Proton9-23".to_string(),
            launch_args: "-novid".to_string(),
            custom_overrides: "dxgi=n,b".to_string(),
            wrapper_command: "gamemoderun".to_string(),
            enable_gamemode: true,
            enable_mangohud: true,
            enable_gamescope: true,
            fingerprint_lock: true,
            playtime_seconds: 7200,
            last_played: "2026-10-24T12:00:00Z".to_string(),
            ..Default::default()
        };
        cfg.env_vars
            .insert("PROTON_ENABLE_NVAPI".to_string(), "1".to_string());
        cfg.env_vars
            .insert("CUSTOM_VAR".to_string(), "VALUE".to_string());
        cfg.gamescope_settings.upscale_method = "fsr".to_string();
        cfg.gamescope_settings.fps_limiter = "120".to_string();

        let game = Game::from_config(&cfg, "idle", true, Some("15.2 GB".to_string()));
        assert_eq!(game.id, "cyberpunk_2077");
        assert_eq!(game.title, "Cyberpunk 2077");
        assert_eq!(game.playtime_min, 120);
        assert!(game.shortcut);
        assert_eq!(game.prefix_size.as_deref(), Some("15.2 GB"));
        assert!(game.wine.bwrap_lock);
        assert!(game.wine.nvapi);
        assert_eq!(game.graphics.gs.method, "fsr");
        assert_eq!(game.graphics.gs.fps, "120");

        let converted_back = game.to_game_config(Some(&cfg));
        assert_eq!(converted_back.name, cfg.name);
        assert_eq!(converted_back.exe_path, cfg.exe_path);
        assert_eq!(converted_back.launch_args, cfg.launch_args);
        assert_eq!(converted_back.custom_overrides, cfg.custom_overrides);
        assert_eq!(converted_back.enable_gamemode, cfg.enable_gamemode);
        assert_eq!(converted_back.fingerprint_lock, cfg.fingerprint_lock);
        assert_eq!(
            converted_back
                .env_vars
                .get("PROTON_ENABLE_NVAPI")
                .map(|s| s.as_str()),
            Some("1")
        );
        assert_eq!(
            converted_back
                .env_vars
                .get("CUSTOM_VAR")
                .map(|s| s.as_str()),
            Some("VALUE")
        );
    }

    #[test]
    fn test_app_settings_roundtrip() {
        let global = GlobalConfigData {
            proton_version: "GE-Proton9-23".to_string(),
            launch_args: "-skipintro".to_string(),
            custom_overrides: "d3d11=n".to_string(),
            wrapper_command: "mangohud".to_string(),
            enable_gamemode: true,
            enable_mangohud: true,
            enable_gamescope: false,
            ..Default::default()
        };

        let app_settings = AppSettings::from(&global);
        assert_eq!(app_settings.default_runner, "GE-Proton9-23");
        assert_eq!(app_settings.default_args, "-skipintro");
        assert!(app_settings.default_gamemode);
        assert!(app_settings.default_mangohud);
        assert!(!app_settings.default_gamescope);

        let roundtrip = app_settings.to_global_config_data(Some(&global));
        assert_eq!(roundtrip.proton_version, global.proton_version);
        assert_eq!(roundtrip.launch_args, global.launch_args);
        assert_eq!(roundtrip.custom_overrides, global.custom_overrides);
        assert_eq!(roundtrip.enable_gamemode, global.enable_gamemode);
        assert_eq!(roundtrip.enable_mangohud, global.enable_mangohud);
    }
}
