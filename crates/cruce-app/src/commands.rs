use crate::models::{AppSettings, Game, Runner};
use crate::state::{ActiveSession, AppState};
use cruce_core::desktop_shortcuts::{
    create_game_shortcut, has_game_shortcut, remove_game_shortcut,
};
use cruce_core::paths::{safe_name, Paths};
use cruce_core::pe_icon::extract_exe_icon;
use cruce_core::process::{is_pid_alive, scan_uuid_pids, stop_game_session};
use cruce_core::types::{GameConfig, GameState};
use cruce_core::wine_tools::{
    calculate_prefix_size, launch_regedit, launch_taskmgr, launch_winecfg, launch_winetricks,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};
use tracing::{error, info};

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn find_umu_binary() -> String {
    let bundled = Paths::runner_dir().join("umu-run");
    if bundled.exists() {
        return bundled.to_string_lossy().to_string();
    }
    if let Ok(paths) = std::env::var("PATH") {
        for p in std::env::split_paths(&paths) {
            let candidate = p.join("umu-run");
            if candidate.is_file() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }
    "umu-run".to_string()
}

#[tauri::command]
pub fn get_games(state: State<'_, AppState>) -> Result<Vec<Game>, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    store.scan_games();

    let mut result = Vec::new();
    for cfg in store.games() {
        let status = match state.state_tracker.get(&cfg.name) {
            GameState::Running => "running",
            GameState::Launching => "launching",
            _ => "idle",
        };

        let shortcut = has_game_shortcut(&cfg.name);
        let prefix_path = if !cfg.prefix_path.is_empty() {
            PathBuf::from(&cfg.prefix_path)
        } else {
            Paths::prefixes_dir().join(format!("{}prefix", safe_name(&cfg.name)))
        };

        let prefix_size = if prefix_path.exists() {
            let sz = calculate_prefix_size(&prefix_path);
            if sz > 0 {
                Some(format_bytes(sz))
            } else {
                None
            }
        } else {
            None
        };

        result.push(Game::from_config(cfg, status, shortcut, prefix_size));
    }

    Ok(result)
}

#[tauri::command]
pub fn get_game(state: State<'_, AppState>, id: String) -> Result<Game, String> {
    let games = get_games(state)?;
    games
        .into_iter()
        .find(|g| g.id == id || safe_name(&g.title) == id)
        .ok_or_else(|| format!("Game '{}' not found", id))
}

#[tauri::command]
pub fn save_game(state: State<'_, AppState>, game: Game) -> Result<Game, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;

    let existing_name = if store.get_game(&game.title).is_some() {
        game.title.clone()
    } else {
        let old = store
            .games()
            .iter()
            .find(|g| safe_name(&g.name) == game.id || g.name == game.id)
            .map(|g| g.name.clone());
        if let Some(old_name) = old {
            store
                .rename_game(&old_name, &game.title)
                .map_err(|e| e.to_string())?;
            game.title.clone()
        } else {
            game.title.clone()
        }
    };

    let existing = store.get_game(&existing_name).cloned();
    let updated_cfg = game.to_game_config(existing.as_ref());

    store
        .update_game(&existing_name, |cfg| *cfg = updated_cfg)
        .map_err(|e| e.to_string())?;

    let shortcut = has_game_shortcut(&game.title);
    let prefix_path = PathBuf::from(&game.prefix);
    let prefix_size = if prefix_path.exists() {
        let sz = calculate_prefix_size(&prefix_path);
        if sz > 0 {
            Some(format_bytes(sz))
        } else {
            None
        }
    } else {
        None
    };

    let cfg = store.get_game(&game.title).cloned().unwrap_or_default();
    Ok(Game::from_config(&cfg, &game.status, shortcut, prefix_size))
}

#[tauri::command]
pub fn add_game(
    app: AppHandle,
    state: State<'_, AppState>,
    title: String,
    exe: String,
    install_dir: String,
    hue: u32,
    runner: String,
) -> Result<Game, String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;

    let new_cfg = GameConfig {
        name: title.clone(),
        exe_path: exe.clone(),
        install_dir,
        proton_version: runner,
        ..Default::default()
    };

    store.add_game(new_cfg.clone()).map_err(|e| e.to_string())?;

    // Try in-memory PE icon extraction
    let exe_key = cruce_core::artwork::ArtworkManager::artwork_key(&exe, None, &title);
    let artwork_dir = Paths::artwork_dir().join(&exe_key);
    let _ = fs::create_dir_all(&artwork_dir);
    let icon_png = artwork_dir.join("icon.png");
    let _ = extract_exe_icon(Path::new(&exe), &icon_png);

    // Background Steam artwork fetcher
    let app_clone = app.clone();
    let title_clone = title.clone();
    let exe_clone = exe.clone();
    thread::spawn(move || {
        let artwork_mgr = cruce_core::artwork::ArtworkManager::default();
        let steam_api = cruce_core::steam_api::SteamApi::default();
        let result = artwork_mgr.fetch_all_artwork(&steam_api, &exe_clone, &title_clone, None);
        if result.portrait_path.is_some()
            || result.hero_path.is_some()
            || result.header_path.is_some()
        {
            let _ = app_clone.emit(
                "artwork_updated",
                serde_json::json!({
                    "id": safe_name(&title_clone),
                    "title": title_clone
                }),
            );
        }
    });

    let shortcut = has_game_shortcut(&title);
    let mut g = Game::from_config(&new_cfg, "idle", shortcut, None);
    if hue > 0 {
        g.hue = hue;
    }
    Ok(g)
}

#[tauri::command]
pub fn remove_game(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let game = store
        .games()
        .iter()
        .find(|g| safe_name(&g.name) == id || g.name == id)
        .cloned();

    if let Some(g) = game {
        let _ = remove_game_shortcut(&g.name);
        store
            .remove_game(&g.name, false)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn launch_game(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    let game_cfg = {
        let mut store = state.store.lock().map_err(|e| e.to_string())?;
        store.scan_games();
        store
            .games()
            .iter()
            .find(|g| safe_name(&g.name) == id || g.name == id)
            .cloned()
            .ok_or_else(|| format!("Game '{}' not found", id))?
    };

    let mut resolved = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        store.global_config.resolve(&game_cfg)
    };

    // Check if game is already active
    let current_state = state.state_tracker.get(&resolved.name);
    if current_state != GameState::Idle {
        return Err(format!(
            "Game '{}' is already in {:?} state",
            resolved.name, current_state
        ));
    }

    // Validate executable exists
    if resolved.exe_path.trim().is_empty() {
        return Err(format!("No executable configured for '{}'", resolved.name));
    }
    let exe = Path::new(&resolved.exe_path);
    if !exe.exists() {
        return Err(format!("Executable not found:\n{}", resolved.exe_path));
    }

    // Resolve proton_path from proton_version if empty or invalid
    if resolved.proton_path.trim().is_empty() || !Path::new(&resolved.proton_path).exists() {
        let installed = state.proton_manager.scan_installed();
        let target = if !resolved.proton_version.is_empty() {
            installed
                .into_iter()
                .find(|r| r.name == resolved.proton_version || r.version == resolved.proton_version)
        } else {
            installed.into_iter().next()
        };
        if let Some(runner) = target {
            resolved.proton_path = runner.path;
        } else {
            return Err("No compatible Proton runner found. Please download a runner from Settings -> Runners.".to_string());
        }
    }

    let umu_bin = find_umu_binary();
    let prepared = state
        .launcher
        .prepare_launch(&resolved, &umu_bin, &Paths::prefixes_dir())
        .map_err(|e| e.to_string())?;

    state
        .state_tracker
        .transition(&resolved.name, GameState::Launching);
    let _ = app.emit(
        "game_state_changed",
        serde_json::json!({
            "id": safe_name(&resolved.name),
            "title": resolved.name,
            "status": "launching"
        }),
    );

    let (pid, guard) = state
        .launcher
        .execute_launch(&prepared)
        .map_err(|e| e.to_string())?;

    let session = ActiveSession {
        session_uuid: prepared.session_uuid.clone(),
        pid,
        screensaver_guard: guard,
        started_at: Instant::now(),
    };

    {
        let mut sessions = state.active_sessions.lock().map_err(|e| e.to_string())?;
        sessions.insert(resolved.name.clone(), session);
    }

    let _ = app.emit(
        "game_state_changed",
        serde_json::json!({
            "id": safe_name(&resolved.name),
            "title": resolved.name,
            "status": "running"
        }),
    );

    // Watchdog thread to detect process exit
    let tracker_clone = state.state_tracker.clone();
    let uuid_clone = prepared.session_uuid.clone();
    let name_clone = resolved.name.clone();
    let app_clone = app.clone();
    let sessions_clone = state.active_sessions.clone();
    let store_clone = state.store.clone();
    let started_at = Instant::now();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));

        loop {
            thread::sleep(Duration::from_secs(1));

            let tagged = scan_uuid_pids(&uuid_clone);
            let root_alive = is_pid_alive(pid);

            if tagged.is_empty() && !root_alive {
                break;
            }
        }

        let elapsed = started_at.elapsed().as_secs();
        let was_active = if let Ok(mut sessions) = sessions_clone.lock() {
            sessions.remove(&name_clone).is_some() // Dropping session automatically releases ScreensaverGuard
        } else {
            false
        };

        if was_active {
            if let Ok(mut store) = store_clone.lock() {
                let _ = store.record_playtime(&name_clone, elapsed);
            }
            tracker_clone.force_idle(&name_clone);
            let _ = app_clone.emit(
                "game_state_changed",
                serde_json::json!({
                    "id": safe_name(&name_clone),
                    "title": name_clone,
                    "status": "idle",
                    "playtimeDeltaSeconds": elapsed
                }),
            );
            info!(
                "Game '{}' session exited cleanly after {}s.",
                name_clone, elapsed
            );
        }
    });

    Ok(())
}

#[tauri::command]
pub fn stop_game(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut sessions = state.active_sessions.lock().map_err(|e| e.to_string())?;
    let target_key = sessions
        .keys()
        .find(|k| safe_name(k) == id || *k == &id)
        .cloned();

    if let Some(key) = target_key {
        if let Some(session) = sessions.remove(&key) {
            let elapsed = session.started_at.elapsed().as_secs();
            if let Ok(mut store) = state.store.lock() {
                let _ = store.record_playtime(&key, elapsed);
            }
            state.state_tracker.transition(&key, GameState::Stopping);
            let _ = stop_game_session(session.pid, &session.session_uuid);
            state.state_tracker.force_idle(&key);

            let _ = app.emit(
                "game_state_changed",
                serde_json::json!({
                    "id": safe_name(&key),
                    "title": key,
                    "status": "idle",
                    "playtimeDeltaSeconds": elapsed
                }),
            );
        }
    } else {
        state.state_tracker.force_idle(&id);
        let _ = app.emit(
            "game_state_changed",
            serde_json::json!({
                "id": safe_name(&id),
                "title": id,
                "status": "idle"
            }),
        );
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_play(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    let current_status = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let game = store
            .games()
            .iter()
            .find(|g| safe_name(&g.name) == id || g.name == id);
        if let Some(g) = game {
            state.state_tracker.get(&g.name)
        } else {
            GameState::Idle
        }
    };

    if current_status == GameState::Running || current_status == GameState::Launching {
        stop_game(app, state, id)
    } else {
        launch_game(app, state, id)
    }
}

#[tauri::command]
pub fn get_installed_runners(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let installed = state.proton_manager.scan_installed();
    Ok(installed.into_iter().map(|r| r.name).collect())
}

#[tauri::command]
pub fn get_runners(state: State<'_, AppState>) -> Result<Vec<Runner>, String> {
    let installed = state.proton_manager.scan_installed();
    let mut installed_map = std::collections::HashMap::new();
    for r in &installed {
        installed_map.insert(r.name.clone(), r.clone());
    }

    let mut list = Vec::new();

    for r in installed {
        let sz = calculate_prefix_size(Path::new(&r.path));
        let size_str = if sz > 0 {
            format_bytes(sz)
        } else {
            "Installed".to_string()
        };
        let source_key = if r.source == "cachy" {
            "cachyos".to_string()
        } else {
            r.source.clone()
        };
        list.push(Runner {
            id: r.name.clone(),
            name: r.name.clone(),
            source: source_key,
            version: r.version.clone(),
            size: size_str,
            date: "Local".to_string(),
            installed: true,
            size_bytes: sz,
        });
    }

    let sources = ["ge", "umu", "cachy"];
    let handles: Vec<_> = sources
        .iter()
        .map(|&src| {
            let pm = state.proton_manager.clone();
            thread::spawn(move || pm.fetch_releases(src))
        })
        .collect();

    for h in handles {
        if let Ok(Ok(releases)) = h.join() {
            for rel in releases.into_iter().take(6) {
                let is_inst = installed_map.contains_key(&rel.name);
                if !is_inst {
                    let source_key = if rel.source == "cachy" {
                        "cachyos".to_string()
                    } else {
                        rel.source.clone()
                    };
                    list.push(Runner {
                        id: rel.tag.clone(),
                        name: rel.name.clone(),
                        source: source_key,
                        version: rel.tag.clone(),
                        size: format_bytes(rel.size),
                        date: rel.published.clone(),
                        installed: false,
                        size_bytes: rel.size,
                    });
                }
            }
        }
    }

    Ok(list)
}

#[tauri::command]
pub fn download_runner(
    app: AppHandle,
    state: State<'_, AppState>,
    runner_id: String,
) -> Result<(), String> {
    state.cancel_download_flag.store(false, Ordering::SeqCst);
    let cancel = state.cancel_download_flag.clone();
    let pm = state.proton_manager.clone();
    let runner_id_clone = runner_id.clone();

    thread::spawn(move || {
        let app_progress = app.clone();
        let rid = runner_id_clone.clone();

        let source_id = if runner_id_clone.to_lowercase().contains("cachy") {
            "cachy"
        } else if runner_id_clone.to_lowercase().contains("umu") {
            "umu"
        } else {
            "ge"
        };

        let res = pm.download_and_install(
            &runner_id_clone,
            source_id,
            Some(&cancel),
            Some(&move |percent, _status| {
                let _ = app_progress.emit(
                    "runner_download_progress",
                    serde_json::json!({
                        "runnerId": rid,
                        "progress": percent
                    }),
                );
            }),
        );

        match res {
            Ok(_) => {
                let _ = app.emit(
                    "runner_download_progress",
                    serde_json::json!({
                        "runnerId": runner_id_clone,
                        "progress": 100,
                        "complete": true
                    }),
                );
            }
            Err(e) => {
                error!("Runner download failed: {:?}", e);
                let _ = app.emit(
                    "runner_download_progress",
                    serde_json::json!({
                        "runnerId": runner_id_clone,
                        "progress": 0,
                        "error": e.to_string()
                    }),
                );
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_download(state: State<'_, AppState>) -> Result<(), String> {
    state.cancel_download_flag.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn remove_runner(state: State<'_, AppState>, runner_id: String) -> Result<(), String> {
    state
        .proton_manager
        .delete_version(&runner_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    Ok(AppSettings::from(store.global_config.data()))
}

#[tauri::command]
pub fn update_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let updated_data = settings.to_global_config_data(Some(store.global_config.data()));
    store
        .global_config
        .update(updated_data)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_wine_tool(state: State<'_, AppState>, id: String, tool: String) -> Result<(), String> {
    let (prefix_path, proton_path) = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let game = store
            .games()
            .iter()
            .find(|g| safe_name(&g.name) == id || g.name == id)
            .ok_or_else(|| format!("Game '{}' not found", id))?;

        let resolved = store.global_config.resolve(game);
        let prefix = if !resolved.prefix_path.is_empty() {
            PathBuf::from(&resolved.prefix_path)
        } else {
            Paths::prefixes_dir().join(format!("{}prefix", safe_name(&resolved.name)))
        };

        let mut ppath = resolved.proton_path.clone();
        if ppath.trim().is_empty() || !Path::new(&ppath).exists() {
            let installed = state.proton_manager.scan_installed();
            let target = if !resolved.proton_version.is_empty() {
                installed.into_iter().find(|r| {
                    r.name == resolved.proton_version || r.version == resolved.proton_version
                })
            } else {
                installed.into_iter().next()
            };
            if let Some(r) = target {
                ppath = r.path;
            } else {
                return Err("No compatible Proton runner found. Please install a Proton runner first.".to_string());
            }
        }

        (prefix, ppath)
    };

    let umu_bin = find_umu_binary();
    match tool.as_str() {
        "winecfg" => {
            launch_winecfg(&umu_bin, &proton_path, &prefix_path).map_err(|e| e.to_string())?
        }
        "regedit" => {
            launch_regedit(&umu_bin, &proton_path, &prefix_path).map_err(|e| e.to_string())?
        }
        "taskmgr" => {
            launch_taskmgr(&umu_bin, &proton_path, &prefix_path).map_err(|e| e.to_string())?
        }
        "winetricks" => {
            launch_winetricks(&umu_bin, &proton_path, &prefix_path).map_err(|e| e.to_string())?
        }
        _ => return Err(format!("Unknown wine tool: {}", tool)),
    };
    Ok(())
}

#[tauri::command]
pub fn reset_prefix(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let game = store
        .games()
        .iter()
        .find(|g| safe_name(&g.name) == id || g.name == id)
        .cloned()
        .ok_or_else(|| format!("Game '{}' not found", id))?;

    store
        .reset_game_prefix(&game.name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn backup_game_saves(
    state: State<'_, AppState>,
    id: String,
) -> Result<cruce_core::BackupInfo, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let game = store
        .games()
        .iter()
        .find(|g| safe_name(&g.name) == id || g.name == id)
        .cloned()
        .ok_or_else(|| format!("Game '{}' not found", id))?;

    let prefix = if !game.prefix_path.is_empty() {
        PathBuf::from(&game.prefix_path)
    } else {
        Paths::prefixes_dir().join(format!("{}prefix", safe_name(&game.name)))
    };

    cruce_core::SaveManager::backup_saves(&prefix, &game.name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_game_saves(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let game = store
        .games()
        .iter()
        .find(|g| safe_name(&g.name) == id || g.name == id)
        .cloned()
        .ok_or_else(|| format!("Game '{}' not found", id))?;

    let prefix = if !game.prefix_path.is_empty() {
        PathBuf::from(&game.prefix_path)
    } else {
        Paths::prefixes_dir().join(format!("{}prefix", safe_name(&game.name)))
    };

    let latest = cruce_core::SaveManager::get_latest_backup(&game.name)
        .ok_or_else(|| format!("No backup found for game '{}'", game.name))?;

    cruce_core::SaveManager::restore_saves(&prefix, Path::new(&latest.file_path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_backup_info(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<cruce_core::BackupInfo>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let game = store
        .games()
        .iter()
        .find(|g| safe_name(&g.name) == id || g.name == id)
        .cloned();

    let game_name = match game {
        Some(g) => g.name,
        None => id,
    };

    Ok(cruce_core::SaveManager::get_latest_backup(&game_name))
}

#[tauri::command]
pub fn rescan_library(state: State<'_, AppState>) -> Result<Vec<Game>, String> {
    {
        let mut store = state.store.lock().map_err(|e| e.to_string())?;
        store.scan_games();
    }
    get_games(state)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZipApplyResult {
    pub success: bool,
    pub dlls: Vec<String>,
    pub exe: String,
    pub message: String,
}

#[tauri::command]
pub fn apply_zip_to_game(
    state: State<'_, AppState>,
    id: String,
    zip_path: String,
) -> Result<ZipApplyResult, String> {
    let (mut cfg, target_name) = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let game = store
            .games()
            .iter()
            .find(|g| safe_name(&g.name) == id || g.name == id)
            .cloned()
            .ok_or_else(|| format!("Game '{}' not found", id))?;
        (game.clone(), game.name.clone())
    };

    let dest_dir = if !cfg.install_dir.is_empty() && Path::new(&cfg.install_dir).is_dir() {
        PathBuf::from(&cfg.install_dir)
    } else if !cfg.exe_path.is_empty() {
        Path::new(&cfg.exe_path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(Paths::games_dir)
    } else {
        return Err("No valid install directory found for game".to_string());
    };

    let extracted = cruce_core::zip_archive::extract_zip_mod(Path::new(&zip_path), &dest_dir)
        .map_err(|e| e.to_string())?;

    // Merge DLL overrides
    if !extracted.dlls.is_empty() {
        let mut existing_dlls: std::collections::BTreeSet<String> = cfg
            .custom_overrides
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        for dll in &extracted.dlls {
            existing_dlls.insert(format!("{}=n,b", dll));
        }

        cfg.custom_overrides = existing_dlls.into_iter().collect::<Vec<_>>().join(",");
    }

    // If an executable was extracted, update exe_path and migrate artwork
    let new_exe = if let Some(first_exe) = extracted.exes.first() {
        let old_exe = cfg.exe_path.clone();
        cfg.exe_path = first_exe.clone();
        let artwork_mgr = cruce_core::artwork::ArtworkManager::default();
        artwork_mgr.migrate_artwork(&old_exe, first_exe);
        first_exe.clone()
    } else {
        String::new()
    };

    {
        let mut store = state.store.lock().map_err(|e| e.to_string())?;
        store
            .update_game(&target_name, |c| *c = cfg.clone())
            .map_err(|e| e.to_string())?;
    }

    Ok(ZipApplyResult {
        success: true,
        dlls: extracted.dlls,
        exe: new_exe,
        message: "Mod archive applied successfully".to_string(),
    })
}

#[tauri::command]
pub fn rename_game(
    state: State<'_, AppState>,
    id: String,
    new_title: String,
) -> Result<Game, String> {
    let clean_new = new_title.trim();
    if clean_new.is_empty() {
        return Err("New title cannot be empty".to_string());
    }

    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let old_name = {
        let g = store
            .games()
            .iter()
            .find(|g| safe_name(&g.name) == id || g.name == id)
            .ok_or_else(|| format!("Game '{}' not found", id))?;
        g.name.clone()
    };

    if old_name != clean_new {
        store
            .rename_game(&old_name, clean_new)
            .map_err(|e| e.to_string())?;
    }

    let shortcut = has_game_shortcut(clean_new);
    let cfg = store.get_game(clean_new).cloned().unwrap_or_default();
    Ok(Game::from_config(&cfg, "idle", shortcut, None))
}

#[tauri::command]
pub fn clear_game_logs(id: String) -> Result<(), String> {
    let log_dir = Paths::game_logs_dir(&id);
    if log_dir.exists() {
        if let Ok(entries) = fs::read_dir(&log_dir) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn open_game_log_dir(id: String) -> Result<(), String> {
    let log_dir = Paths::game_logs_dir(&id);
    let _ = fs::create_dir_all(&log_dir);
    open_folder(log_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if p.exists() {
        let _ = std::process::Command::new("xdg-open").arg(p).spawn();
        Ok(())
    } else {
        Err(format!("Path does not exist: {}", path))
    }
}

#[tauri::command]
pub fn toggle_shortcut(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let game = store
        .games()
        .iter()
        .find(|g| safe_name(&g.name) == id || g.name == id)
        .cloned()
        .ok_or_else(|| format!("Game '{}' not found", id))?;

    if has_game_shortcut(&game.name) {
        remove_game_shortcut(&game.name);
        Ok(false)
    } else {
        create_game_shortcut(&game, None).map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub fn get_game_logs(id: String) -> Result<Vec<(String, String)>, String> {
    let log_dir = Paths::game_logs_dir(&id);
    if let Ok(entries) = fs::read_dir(&log_dir) {
        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("log"))
            .collect();
        files.sort();

        if let Some(latest) = files.last() {
            if let Ok(content) = fs::read_to_string(latest) {
                let lines: Vec<(String, String)> = content
                    .lines()
                    .take(200)
                    .map(|l| {
                        let level = if l.to_lowercase().contains("err") {
                            "warn".to_string()
                        } else if l.to_lowercase().contains("info") {
                            "info".to_string()
                        } else {
                            "debug".to_string()
                        };
                        (level, l.to_string())
                    })
                    .collect();
                return Ok(lines);
            }
        }
    }

    Ok(vec![(
        "info".into(),
        "No previous execution log found on disk.".into(),
    )])
}

#[tauri::command]
pub fn browse_file(
    filter_name: Option<String>,
    extensions: Option<Vec<String>>,
) -> Result<Option<String>, String> {
    let mut dialog = rfd::FileDialog::new();
    if let (Some(name), Some(exts)) = (filter_name, extensions) {
        let ext_refs: Vec<&str> = exts.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&name, &ext_refs);
    }
    let res = dialog.pick_file().map(|p| p.to_string_lossy().to_string());
    Ok(res)
}

#[tauri::command]
pub fn browse_directory() -> Result<Option<String>, String> {
    let res = rfd::FileDialog::new()
        .pick_folder()
        .map(|p| p.to_string_lossy().to_string());
    Ok(res)
}
