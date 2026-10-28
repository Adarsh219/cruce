use cruce_app::models::{AppSettings, Game};
use cruce_app::state::AppState;
use cruce_core::types::{GameConfig, GameState};
use tempfile::tempdir;

#[test]
fn test_app_state_crud_and_game_lifecycle() {
    let tmp = tempdir().unwrap();
    let games_dir = tmp.path().join("games");
    let prefixes_dir = tmp.path().join("prefixes");
    let global_config_path = tmp.path().join("global_config.json");
    let compat_dir = tmp.path().join("compatibilitytools.d");

    let app_state = AppState::new_with_dirs(
        games_dir.clone(),
        prefixes_dir.clone(),
        global_config_path.clone(),
        compat_dir.clone(),
    );

    // 1. Add game via store
    let game_cfg = GameConfig {
        name: "Elden Ring".to_string(),
        exe_path: "/games/eldenring/eldenring.exe".to_string(),
        install_dir: "/games/eldenring".to_string(),
        proton_version: "GE-Proton9-23".to_string(),
        launch_args: "-novid".to_string(),
        enable_gamemode: true,
        ..Default::default()
    };

    {
        let mut store = app_state.store.lock().unwrap();
        store.add_game(game_cfg).unwrap();
        assert_eq!(store.games().len(), 1);
    }

    // 2. Query games and map to frontend Game model
    let game = {
        let mut store = app_state.store.lock().unwrap();
        store.scan_games();
        let cfg = store.get_game("Elden Ring").unwrap();
        Game::from_config(cfg, "idle", false, None)
    };

    assert_eq!(game.id, "elden_ring");
    assert_eq!(game.title, "Elden Ring");
    assert_eq!(game.runner, "GE-Proton9-23");
    assert!(game.graphics.gamemode);
    assert_eq!(game.status, "idle");

    // 3. Update game via frontend Game model
    let mut modified_game = game.clone();
    modified_game.launch.args = "-novid -high".to_string();
    modified_game.wine.wayland = true;
    modified_game.wine.hdr = true;
    modified_game.wine.bwrap_lock = true;
    modified_game.graphics.gamescope = true;
    modified_game.graphics.gs.method = "fsr".to_string();
    modified_game.graphics.gs.fps = "144".to_string();

    {
        let mut store = app_state.store.lock().unwrap();
        let existing = store.get_game(&modified_game.title).cloned();
        let updated_cfg = modified_game.to_game_config(existing.as_ref());
        store
            .update_game(&modified_game.title, |cfg| *cfg = updated_cfg)
            .unwrap();
    }

    // 4. Verify disk persistence and updated fields
    {
        let mut store = app_state.store.lock().unwrap();
        store.scan_games();
        let reloaded = store.get_game("Elden Ring").unwrap();
        assert_eq!(reloaded.launch_args, "-novid -high");
        assert!(reloaded.enable_gamescope);
        assert!(reloaded.fingerprint_lock);
        assert_eq!(
            reloaded
                .env_vars
                .get("PROTON_ENABLE_WAYLAND")
                .map(|s| s.as_str()),
            Some("1")
        );
        assert_eq!(
            reloaded
                .env_vars
                .get("PROTON_ENABLE_HDR")
                .map(|s| s.as_str()),
            Some("1")
        );
        assert_eq!(reloaded.gamescope_settings.fps_limiter, "144");
    }

    // 5. Test state tracker transitions
    assert_eq!(app_state.state_tracker.get("Elden Ring"), GameState::Idle);
    app_state
        .state_tracker
        .transition("Elden Ring", GameState::Launching);
    assert_eq!(
        app_state.state_tracker.get("Elden Ring"),
        GameState::Launching
    );
    app_state
        .state_tracker
        .transition("Elden Ring", GameState::Running);
    assert_eq!(
        app_state.state_tracker.get("Elden Ring"),
        GameState::Running
    );
    app_state.state_tracker.force_idle("Elden Ring");
    assert_eq!(app_state.state_tracker.get("Elden Ring"), GameState::Idle);

    // 6. Delete game
    {
        let mut store = app_state.store.lock().unwrap();
        store.remove_game("Elden Ring", false).unwrap();
        assert_eq!(store.games().len(), 0);
    }
}

#[test]
fn test_app_settings_persistence() {
    let tmp = tempdir().unwrap();
    let games_dir = tmp.path().join("games");
    let prefixes_dir = tmp.path().join("prefixes");
    let global_config_path = tmp.path().join("global_config.json");
    let compat_dir = tmp.path().join("compatibilitytools.d");

    let app_state =
        AppState::new_with_dirs(games_dir, prefixes_dir, global_config_path, compat_dir);

    let mut settings = {
        let store = app_state.store.lock().unwrap();
        AppSettings::from(store.global_config.data())
    };

    settings.default_runner = "GE-Proton9-23".to_string();
    settings.default_args = "--vulkan".to_string();
    settings.default_gamemode = true;
    settings.default_mangohud = true;
    settings.default_gamescope = true;

    {
        let mut store = app_state.store.lock().unwrap();
        let updated_data = settings.to_global_config_data(Some(store.global_config.data()));
        store.global_config.update(updated_data).unwrap();
    }

    // Reload from fresh config
    {
        let store = app_state.store.lock().unwrap();
        let loaded_settings = AppSettings::from(store.global_config.data());
        assert_eq!(loaded_settings.default_runner, "GE-Proton9-23");
        assert_eq!(loaded_settings.default_args, "--vulkan");
        assert!(loaded_settings.default_gamemode);
        assert!(loaded_settings.default_mangohud);
        assert!(loaded_settings.default_gamescope);
    }
}

#[test]
fn test_frontend_json_contract() {
    let raw_frontend_json = r#"{
        "id": "hades_ii",
        "title": "Hades II",
        "exe": "/games/hades2/Hades2.exe",
        "installDir": "/games/hades2",
        "prefix": "/prefixes/hades2",
        "runner": "GE-Proton9-23",
        "hue": 240,
        "playtimeMin": 45,
        "lastPlayedDaysAgo": 1,
        "status": "idle",
        "shortcut": false,
        "prefixSize": "2.4 GB",
        "launch": {
            "args": "-vulkan",
            "dllOverrides": "dxgi=n",
            "wrapper": "gamemoderun",
            "env": [
                { "id": "env-1", "key": "CUSTOM_SETTING", "value": "1" }
            ]
        },
        "graphics": {
            "gamemode": true,
            "mangohud": true,
            "gamescope": true,
            "dlss": false,
            "fsr3": false,
            "fsr4": false,
            "fsr4rdna3": false,
            "xess": false,
            "gs": {
                "renderW": "1080",
                "renderH": "1920",
                "outW": "1440",
                "outH": "2560",
                "method": "fsr",
                "windowType": "borderless",
                "grabCursor": false,
                "fps": "144",
                "fpsUnfocused": "60",
                "extra": "--adaptive-sync"
            }
        },
        "wine": {
            "nvapi": false,
            "wayland": true,
            "hdr": true,
            "wow64": false,
            "wined3d": false,
            "log": false,
            "noSteamClient": false,
            "skipRuntimeUpdate": false,
            "bwrapLock": true
        }
    }"#;

    // 1. Deserialization from frontend JSON contract
    let game: Game = serde_json::from_str(raw_frontend_json).expect("Failed to deserialize frontend Game JSON");
    assert_eq!(game.id, "hades_ii");
    assert_eq!(game.title, "Hades II");
    assert_eq!(game.playtime_min, 45);
    assert!(game.wine.wayland);
    assert!(game.wine.hdr);
    assert!(game.wine.bwrap_lock);
    assert_eq!(game.graphics.gs.method, "fsr");
    assert_eq!(game.graphics.gs.fps, "144");

    // 2. Conversion to cruce-core GameConfig
    let cfg = game.to_game_config(None);
    assert_eq!(cfg.name, "Hades II");
    assert_eq!(cfg.exe_path, "/games/hades2/Hades2.exe");
    assert_eq!(cfg.playtime_seconds, 45 * 60);
    assert_eq!(cfg.wrapper_command, "gamemoderun");
    assert!(cfg.enable_gamemode);
    assert!(cfg.enable_mangohud);
    assert!(cfg.enable_gamescope);
    assert!(cfg.fingerprint_lock);
    assert_eq!(
        cfg.env_vars
            .get("PROTON_ENABLE_WAYLAND")
            .map(|s| s.as_str()),
        Some("1")
    );
    assert_eq!(
        cfg.env_vars.get("PROTON_ENABLE_HDR").map(|s| s.as_str()),
        Some("1")
    );
    assert_eq!(
        cfg.env_vars.get("CUSTOM_SETTING").map(|s| s.as_str()),
        Some("1")
    );
    assert_eq!(cfg.gamescope_settings.upscale_method, "fsr");
    assert_eq!(cfg.gamescope_settings.fps_limiter, "144");

    // 3. Reserialization back to frontend Game model
    let reserialized = Game::from_config(&cfg, "idle", false, Some("2.4 GB".to_string()));
    assert_eq!(reserialized.title, game.title);
    assert_eq!(reserialized.runner, game.runner);
    assert_eq!(reserialized.playtime_min, game.playtime_min);
    assert_eq!(reserialized.prefix_size, game.prefix_size);
    assert!(reserialized.wine.wayland);
    assert!(reserialized.wine.hdr);
    assert!(reserialized.wine.bwrap_lock);
}

#[test]
fn test_end_to_end_game_launch_supervision_lifecycle() {
    let tmp = tempdir().unwrap();
    let games_dir = tmp.path().join("games");
    let prefixes_dir = tmp.path().join("prefixes");
    let global_config_path = tmp.path().join("global_config.json");
    let compat_dir = tmp.path().join("compatibilitytools.d");

    let app_state = AppState::new_with_dirs(
        games_dir.clone(),
        prefixes_dir.clone(),
        global_config_path.clone(),
        compat_dir.clone(),
    );

    let game_cfg = GameConfig {
        name: "Hollow Knight".to_string(),
        exe_path: "/games/hollow_knight/hollow_knight.exe".to_string(),
        install_dir: "/games/hollow_knight".to_string(),
        proton_version: "GE-Proton9-23".to_string(),
        launch_args: "-novid".to_string(),
        enable_gamemode: true,
        enable_mangohud: true,
        ..Default::default()
    };

    // 1. Prepare launch pipeline
    let prepared = app_state
        .launcher
        .prepare_launch(&game_cfg, "/usr/bin/umu-run", &prefixes_dir)
        .expect("prepare_launch must succeed");

    assert_eq!(prepared.game_name, "Hollow Knight");
    assert!(!prepared.session_uuid.is_empty());
    assert!(prepared.command.contains(&"/usr/bin/umu-run".to_string()));
    assert!(prepared
        .command
        .contains(&"/games/hollow_knight/hollow_knight.exe".to_string()));
    assert!(prepared.command.contains(&"-novid".to_string()));

    // Verify environment variables injected
    assert_eq!(
        prepared.env.get("CRUCIBLE_GAME_ID"),
        Some(&prepared.session_uuid)
    );
    assert_eq!(
        prepared.env.get("PROTON_VERB").map(|s| s.as_str()),
        Some("waitforexitandrun")
    );

    // 2. State transition validation
    assert!(app_state
        .state_tracker
        .transition("Hollow Knight", GameState::Launching));
    assert_eq!(
        app_state.state_tracker.get("Hollow Knight"),
        GameState::Launching
    );

    assert!(app_state
        .state_tracker
        .transition("Hollow Knight", GameState::Running));
    assert_eq!(
        app_state.state_tracker.get("Hollow Knight"),
        GameState::Running
    );

    // 3. Stop session and return to Idle
    assert!(app_state
        .state_tracker
        .transition("Hollow Knight", GameState::Stopping));
    assert_eq!(
        app_state.state_tracker.get("Hollow Knight"),
        GameState::Stopping
    );

    app_state.state_tracker.force_idle("Hollow Knight");
    assert_eq!(
        app_state.state_tracker.get("Hollow Knight"),
        GameState::Idle
    );
}

#[test]
fn test_app_save_backup_and_restore_commands() {
    let tmp = tempdir().unwrap();
    let app_state = AppState::new();

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let game_name = format!("App Test Game {}", nonce);
    let backup_dir =
        cruce_core::paths::Paths::backups_dir().join(cruce_core::paths::safe_name(&game_name));
    if backup_dir.exists() {
        let _ = std::fs::remove_dir_all(&backup_dir);
    }

    let prefix = tmp.path().join("prefix_hollow_knight");
    let saves = prefix.join("drive_c/users/steamuser/AppData/LocalLow/Team Cherry/Hollow Knight");
    std::fs::create_dir_all(&saves).unwrap();
    std::fs::write(saves.join("user1.dat"), b"SAVE_DATA_KNIGHT_100_PERCENT").unwrap();

    let game_cfg = GameConfig {
        name: game_name.clone(),
        exe_path: "/games/hk/hollow_knight.exe".to_string(),
        prefix_path: prefix.to_string_lossy().to_string(),
        ..Default::default()
    };

    {
        let mut store = app_state.store.lock().unwrap();
        store.add_game(game_cfg).unwrap();
    }

    // 1. Verify initially no backup info
    let initial_info = cruce_core::SaveManager::get_latest_backup(&game_name);
    assert!(initial_info.is_none());

    // 2. Perform backup
    let backup_info = cruce_core::SaveManager::backup_saves(&prefix, &game_name).unwrap();
    assert!(backup_info.size_bytes > 0);
    assert!(std::path::Path::new(&backup_info.file_path).is_file());

    // 3. Query latest
    let query_info = cruce_core::SaveManager::get_latest_backup(&game_name).unwrap();
    assert_eq!(query_info.file_path, backup_info.file_path);

    // 4. Wipe save file
    std::fs::remove_file(saves.join("user1.dat")).unwrap();
    assert!(!saves.join("user1.dat").exists());

    // 5. Restore
    cruce_core::SaveManager::restore_saves(&prefix, std::path::Path::new(&backup_info.file_path))
        .unwrap();
    assert!(saves.join("user1.dat").exists());
    assert_eq!(
        std::fs::read(saves.join("user1.dat")).unwrap(),
        b"SAVE_DATA_KNIGHT_100_PERCENT"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&backup_dir);
}

#[test]
fn test_apply_zip_and_rename_commands() {
    let tmp = tempdir().unwrap();
    let app_state = AppState::new();

    let install_dir = tmp.path().join("install");
    std::fs::create_dir_all(&install_dir).unwrap();
    let game_exe = install_dir.join("game.exe");
    std::fs::write(&game_exe, b"ORIGINAL_EXE").unwrap();

    let initial_name = format!(
        "Zip Test Game {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let game_cfg = GameConfig {
        name: initial_name.clone(),
        exe_path: game_exe.to_string_lossy().to_string(),
        install_dir: install_dir.to_string_lossy().to_string(),
        custom_overrides: "dxgi=n,b".to_string(),
        ..Default::default()
    };

    {
        let mut store = app_state.store.lock().unwrap();
        store.add_game(game_cfg).unwrap();
    }

    // 1. Rename game via store
    let new_name = format!(
        "Renamed Game {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    {
        let mut store = app_state.store.lock().unwrap();
        store.rename_game(&initial_name, &new_name).unwrap();
        assert!(store.get_game(&new_name).is_some());
        assert!(store.get_game(&initial_name).is_none());
    }

    // 2. Clear logs test
    let log_dir = cruce_core::paths::Paths::game_logs_dir(&new_name);
    std::fs::create_dir_all(&log_dir).unwrap();
    std::fs::write(log_dir.join("session.log"), b"OLD_LOG").unwrap();
    assert!(log_dir.join("session.log").exists());
    let _ = cruce_app::commands::clear_game_logs(new_name);
    assert!(!log_dir.join("session.log").exists());
}
