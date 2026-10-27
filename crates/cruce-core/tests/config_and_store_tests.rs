use cruce_core::paths::{
    display_name_from_exe, find_app_id_in_game_dir, find_game_root, safe_name, steam_id_for_name,
};
use cruce_core::types::{GameConfig, GamescopeSettings, GlobalConfigData};
use cruce_core::{GameStore, GlobalConfig};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_safe_name_conversions() {
    assert_eq!(
        safe_name("The Witcher 3: Wild Hunt"),
        "the_witcher_3_wild_hunt"
    );
    assert_eq!(safe_name("Cyberpunk 2077"), "cyberpunk_2077");
    assert_eq!(safe_name("DOOM---Eternal!"), "doom_eternal");
    assert_eq!(
        safe_name("  Half-Life 2: Episode 1  "),
        "half_life_2_episode_1"
    );
    assert_eq!(safe_name("___test___"), "test");
}

#[test]
fn test_display_name_from_exe() {
    assert_eq!(display_name_from_exe(Path::new("witcher3.exe")), "Witcher3");
    assert_eq!(
        display_name_from_exe(Path::new("/games/cyberpunk_2077.exe")),
        "Cyberpunk 2077"
    );
    assert_eq!(
        display_name_from_exe(Path::new("C:/Games/baldur-gate-3.exe")),
        "Baldur Gate 3"
    );
}

#[test]
fn test_steam_id_for_name_matches_python() {
    // Explicit Steam App ID passes through
    assert_eq!(steam_id_for_name("Cyberpunk 2077", "1091500"), "1091500");

    // "umu-default" derives deterministic MD5 app ID
    // In Python: int(hashlib.md5("Cyberpunk 2077".encode()).hexdigest()[:8], 16) % 900000 + 100000 == 784972
    let derived = steam_id_for_name("Cyberpunk 2077", "umu-default");
    assert_eq!(derived, "784972");
}

#[test]
fn test_find_game_root_deep_hierarchy() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("MyAwesomeGame");
    let nested = root.join("Engine").join("Binaries").join("Win64");
    fs::create_dir_all(&nested).unwrap();

    let exe_path = nested.join("Game.exe");
    File::create(&exe_path).unwrap();

    let detected = find_game_root(&exe_path);
    assert_eq!(detected, Some(root));
}

#[test]
fn test_find_app_id_in_game_dir() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("SteamGame");
    let sub = root.join("deep").join("folder");
    fs::create_dir_all(&sub).unwrap();

    let app_id_file = sub.join("steam_appid.txt");
    let mut f = File::create(app_id_file).unwrap();
    writeln!(f, "  1091500  ").unwrap();

    let found = find_app_id_in_game_dir(&root);
    assert_eq!(found, Some("1091500".to_string()));
}

#[test]
fn test_global_config_two_tier_inheritance() {
    let tmp = tempdir().unwrap();
    let config_path = tmp.path().join("global_config.json");

    let mut global_config = GlobalConfig::new(Some(config_path));
    let mut global_env = HashMap::new();
    global_env.insert("GLOBAL_ONLY".to_string(), "1".to_string());
    global_env.insert("OVERRIDDEN".to_string(), "global_val".to_string());
    global_env.insert("DISABLED_BY_GAME".to_string(), "true".to_string());

    global_config
        .update(GlobalConfigData {
            proton_version: "GE-Proton9-25".to_string(),
            launch_args: "-novid".to_string(),
            custom_overrides: "dxgi=n,b".to_string(),
            env_vars: global_env,
            wrapper_command: "mangohud".to_string(),
            fingerprint_lock: false,
            enable_gamemode: true,
            enable_mangohud: true,
            enable_gamescope: false,
            gamescope_settings: GamescopeSettings {
                game_width: "1920".to_string(),
                game_height: "1080".to_string(),
                upscale_method: "fsr".to_string(),
                ..Default::default()
            },
        })
        .unwrap();

    // Game with overrides and disabled options
    let mut game_env = HashMap::new();
    game_env.insert("OVERRIDDEN".to_string(), "game_val".to_string());
    game_env.insert("GAME_ONLY".to_string(), "42".to_string());

    let game = GameConfig {
        name: "Test Game".to_string(),
        exe_path: "/games/test/game.exe".to_string(),
        proton_version: "".to_string(),         // Falls back to global
        launch_args: "-fullscreen".to_string(), // Overrides global
        custom_overrides: "".to_string(),       // Falls back to global
        env_vars: game_env,
        disabled_env_vars: vec!["DISABLED_BY_GAME".to_string()],
        enable_gamemode: false,
        disabled_global_flags: vec!["enable_gamemode".to_string()], // Explicitly disabled
        enable_gamescope: true,                                     // Overrides global false
        gamescope_settings: GamescopeSettings {
            game_width: "2560".to_string(), // Overrides global 1920
            game_height: "".to_string(),    // Falls back to global 1080
            ..Default::default()
        },
        ..Default::default()
    };

    let resolved = global_config.resolve(&game);

    // 1. Scalar inheritance
    assert_eq!(resolved.proton_version, "GE-Proton9-25");
    assert_eq!(resolved.launch_args, "-fullscreen");
    assert_eq!(resolved.custom_overrides, "dxgi=n,b");
    assert_eq!(resolved.wrapper_command, "mangohud");

    // 2. Boolean flags with explicit disable
    assert!(!resolved.enable_gamemode); // disabled_global_flags won
    assert!(resolved.enable_mangohud); // inherited from global
    assert!(resolved.enable_gamescope); // overridden by game

    // 3. Env vars merge
    assert_eq!(
        resolved.env_vars.get("GLOBAL_ONLY").map(|s| s.as_str()),
        Some("1")
    );
    assert_eq!(
        resolved.env_vars.get("OVERRIDDEN").map(|s| s.as_str()),
        Some("game_val")
    );
    assert_eq!(
        resolved.env_vars.get("GAME_ONLY").map(|s| s.as_str()),
        Some("42")
    );
    assert!(!resolved.env_vars.contains_key("DISABLED_BY_GAME"));

    // 4. Gamescope merge
    assert_eq!(resolved.gamescope_settings.game_width, "2560");
    assert_eq!(resolved.gamescope_settings.game_height, "1080");
    assert_eq!(resolved.gamescope_settings.upscale_method, "fsr");
}

#[test]
fn test_game_store_crud_lifecycle() {
    let tmp = tempdir().unwrap();
    let games_dir = tmp.path().join("games");
    let prefixes_dir = tmp.path().join("Prefix");
    let global_config = tmp.path().join("global_config.json");

    let mut store = GameStore::new(
        Some(games_dir.clone()),
        Some(prefixes_dir.clone()),
        Some(global_config),
    );

    assert_eq!(store.games().len(), 0);

    // 1. Add game
    let game1 = GameConfig {
        name: "Cyberpunk 2077".to_string(),
        exe_path: "/games/cp2077/bin/x64/Cyberpunk2077.exe".to_string(),
        proton_version: "GE-Proton9-25".to_string(),
        ..Default::default()
    };
    store.add_game(game1).unwrap();

    assert_eq!(store.games().len(), 1);
    assert_eq!(
        store.get_game("Cyberpunk 2077").unwrap().name,
        "Cyberpunk 2077"
    );

    // File on disk check
    let json_file = games_dir.join("cyberpunk_2077.json");
    assert!(json_file.exists());

    // 2. Safe name collision test (same safe_name but different display name)
    let collision_game = GameConfig {
        name: "Cyberpunk: 2077".to_string(),
        exe_path: "/another/path.exe".to_string(),
        ..Default::default()
    };
    let collision_err = store.add_game(collision_game);
    assert!(collision_err.is_err());

    // 3. Playtime tracking
    store.record_playtime("Cyberpunk 2077", 120).unwrap();
    let updated = store.get_game("Cyberpunk 2077").unwrap();
    assert_eq!(updated.playtime_seconds, 120);
    assert!(!updated.last_played.is_empty());

    // 4. Rename game
    store
        .rename_game("Cyberpunk 2077", "Cyberpunk 2077 Phantom Liberty")
        .unwrap();
    assert!(store.get_game("Cyberpunk 2077").is_none());
    assert!(store.get_game("Cyberpunk 2077 Phantom Liberty").is_some());
    assert!(!json_file.exists()); // Old file unlinked
    assert!(games_dir
        .join("cyberpunk_2077_phantom_liberty.json")
        .exists()); // New file exists

    // 5. Remove game with prefix
    let prefix_dir = prefixes_dir.join("cyberpunk_2077_phantom_libertyprefix");
    fs::create_dir_all(&prefix_dir).unwrap();
    assert!(prefix_dir.exists());

    store
        .remove_game("Cyberpunk 2077 Phantom Liberty", true)
        .unwrap();
    assert_eq!(store.games().len(), 0);
    assert!(!prefix_dir.exists()); // Prefix removed
}
