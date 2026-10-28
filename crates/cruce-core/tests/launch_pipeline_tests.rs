use cruce_core::command_builder::build_command;
use cruce_core::desktop_shortcuts::{
    create_game_shortcut, has_game_shortcut, remove_game_shortcut,
};
use cruce_core::dll_overrides::build_dll_overrides;
use cruce_core::env_builder::{build_env, strip_launch_env};
use cruce_core::fingerprint::FingerprintManager;
use cruce_core::gamescope::build_gamescope_command;
use cruce_core::types::{GameConfig, GamescopeSettings};
use cruce_core::wine_tools::calculate_prefix_size;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_dll_overrides_aliases_and_buckets() {
    // 1. Empty input defaults to disabled winemenubuilder
    assert_eq!(build_dll_overrides(""), "winemenubuilder=d");

    // 2. Simple list defaults to n,b mode
    let simple = build_dll_overrides("dxgi, d3d11");
    assert_eq!(simple, "d3d11,dxgi=n,b;winemenubuilder=d");

    // 3. Semicolon-separated pairs with aliases
    let aliases = build_dll_overrides(
        "dxgi = native; d3d11 = builtin; nvapi = disabled; winhttp = b,n; custom = unknown_mode",
    );
    assert_eq!(
        aliases,
        "custom=n,b;winhttp=b,n;d3d11=b;dxgi=n;nvapi,winemenubuilder=d"
    );
}

#[test]
fn test_env_sanitizer_and_builder() {
    let mut test_env = HashMap::new();
    test_env.insert("APPDIR".to_string(), "/tmp/.mount_cruc123".to_string());
    test_env.insert(
        "APPIMAGE".to_string(),
        "/home/user/Crucible.AppImage".to_string(),
    );
    test_env.insert(
        "DESKTOP_STARTUP_ID".to_string(),
        "crucible_1234".to_string(),
    );
    test_env.insert(
        "CRUCIBLE_ORIG_PATH".to_string(),
        "/usr/bin:/bin".to_string(),
    );
    test_env.insert(
        "PATH".to_string(),
        "/tmp/.mount_cruc123/usr/bin:/usr/bin".to_string(),
    );
    test_env.insert("USER_VAR".to_string(), "keep_this".to_string());

    // Run strip_launch_env
    strip_launch_env(&mut test_env);

    // AppImage identity and desktop startup IDs must be removed
    assert!(!test_env.contains_key("APPDIR"));
    assert!(!test_env.contains_key("APPIMAGE"));
    assert!(!test_env.contains_key("DESKTOP_STARTUP_ID"));

    // Original variables must be restored
    assert_eq!(
        test_env.get("PATH").map(|s| s.as_str()),
        Some("/usr/bin:/bin")
    );
    assert!(!test_env.contains_key("CRUCIBLE_ORIG_PATH"));
    assert_eq!(
        test_env.get("USER_VAR").map(|s| s.as_str()),
        Some("keep_this")
    );

    // Test full build_env
    let game = GameConfig {
        name: "Cyberpunk 2077".to_string(),
        exe_path: "/games/cp2077/bin/x64/Cyberpunk2077.exe".to_string(),
        custom_overrides: "dxgi=n,b".to_string(),
        ..Default::default()
    };
    let prefix = Path::new("/tmp/test_prefix");

    let (built_env, session_uuid) =
        build_env(&game, "/steam/compatibilitytools.d/GE-Proton9-25", prefix);

    assert_eq!(
        built_env.get("WINEPREFIX").map(|s| s.as_str()),
        Some("/tmp/test_prefix")
    );
    assert_eq!(
        built_env.get("PROTONPATH").map(|s| s.as_str()),
        Some("/steam/compatibilitytools.d/GE-Proton9-25")
    );
    assert_eq!(built_env.get("WINEARCH").map(|s| s.as_str()), Some("win64"));
    assert_eq!(
        built_env.get("PROTON_VERB").map(|s| s.as_str()),
        Some("waitforexitandrun")
    );
    assert_eq!(
        built_env.get("PULSE_LATENCY_MSEC").map(|s| s.as_str()),
        Some("60")
    );
    assert_eq!(
        built_env.get("WINEDLLOVERRIDES").map(|s| s.as_str()),
        Some("dxgi=n,b;winemenubuilder=d")
    );
    assert_eq!(built_env.get("CRUCIBLE_GAME_ID"), Some(&session_uuid));
    assert_eq!(session_uuid.len(), 36); // Valid UUID v4
}

#[test]
fn test_gamescope_command_builder() {
    let empty_settings = GamescopeSettings::default();
    assert!(build_gamescope_command(&empty_settings).is_empty());

    let configured = GamescopeSettings {
        game_width: "1920".to_string(),
        game_height: "1080".to_string(),
        upscale_width: "2560".to_string(),
        upscale_height: "1440".to_string(),
        upscale_method: "fsr".to_string(),
        window_type: "fullscreen".to_string(),
        fps_limiter: "144".to_string(),
        fps_limiter_no_focus: "60".to_string(),
        enable_force_grab_cursor: true,
        additional_options: "--hdr-enabled".to_string(),
    };

    let cmd = build_gamescope_command(&configured);
    if which::which("gamescope").is_ok() {
        assert!(cmd.contains(&"-w".to_string()));
        assert!(cmd.contains(&"1920".to_string()));
        assert!(cmd.contains(&"-h".to_string()));
        assert!(cmd.contains(&"1080".to_string()));
        assert!(cmd.contains(&"-W".to_string()));
        assert!(cmd.contains(&"2560".to_string()));
        assert!(cmd.contains(&"-H".to_string()));
        assert!(cmd.contains(&"1440".to_string()));
        assert!(cmd.contains(&"-f".to_string()));
        assert!(cmd.contains(&"-r".to_string()));
        assert!(cmd.contains(&"144".to_string()));
        assert!(cmd.contains(&"-o".to_string()));
        assert!(cmd.contains(&"60".to_string()));
        assert!(cmd.contains(&"--force-grab-cursor".to_string()));
        assert!(cmd.contains(&"--hdr-enabled".to_string()));
        assert_eq!(cmd.last().map(|s| s.as_str()), Some("--"));
    }
}

#[test]
fn test_command_builder_nesting() {
    let tmp = tempdir().unwrap();
    let fingerprint = FingerprintManager::new(Some(tmp.path().join("fingerprints")));

    let game = GameConfig {
        name: "Test Game".to_string(),
        exe_path: "/games/test/game.exe".to_string(),
        launch_args: "-novid -freq 144".to_string(),
        wrapper_command: "env SPECIAL_OPT=1".to_string(),
        fingerprint_lock: false,
        enable_gamemode: false,
        enable_mangohud: false,
        enable_gamescope: false,
        ..Default::default()
    };

    let cmd = build_command(
        &game,
        "/usr/bin/umu-run",
        "01234567-89ab-cdef",
        &fingerprint,
    );

    // Verify wrapper prefixes umu-run
    assert!(cmd.len() >= 5);
    let env_pos = cmd
        .iter()
        .position(|s| s == "env")
        .expect("cmd should contain env from wrapper");
    assert_eq!(
        cmd.get(env_pos + 1).map(|s| s.as_str()),
        Some("SPECIAL_OPT=1")
    );
    let umu_pos = cmd
        .iter()
        .position(|s| s == "/usr/bin/umu-run")
        .expect("cmd should contain umu-run");
    assert!(env_pos < umu_pos);

    if which::which("systemd-run").is_ok() {
        assert_eq!(cmd[0], "systemd-run");
        let sysd_pos = cmd.iter().position(|s| s == "systemd-run").unwrap();
        assert!(sysd_pos < env_pos);
    }

    // Followed by umu-run and exe and args
    assert!(cmd.contains(&"/usr/bin/umu-run".to_string()));
    assert!(cmd.contains(&"/games/test/game.exe".to_string()));
    assert!(cmd.contains(&"-novid".to_string()));
    assert!(cmd.contains(&"-freq".to_string()));
    assert!(cmd.contains(&"144".to_string()));
}

#[test]
fn test_desktop_shortcuts_crud() {
    let game = GameConfig {
        name: "The Witcher 3".to_string(),
        exe_path: "/games/witcher3/bin/x64/witcher3.exe".to_string(),
        ..Default::default()
    };

    let shortcut_path = create_game_shortcut(&game, Some("/usr/bin/crucible")).unwrap();
    assert!(shortcut_path.exists());
    assert!(has_game_shortcut("The Witcher 3"));

    // Verify desktop file contents
    let content = fs::read_to_string(&shortcut_path).unwrap();
    assert!(content.contains("[Desktop Entry]"));
    assert!(content.contains("Name=The Witcher 3"));
    assert!(content.contains("Exec=\"/usr/bin/crucible\" --launch \"The Witcher 3\""));
    assert!(content.contains("StartupWMClass=crucible-the_witcher_3"));

    // Verify remove
    assert!(remove_game_shortcut("The Witcher 3"));
    assert!(!shortcut_path.exists());
    assert!(!has_game_shortcut("The Witcher 3"));
}

#[test]
fn test_calculate_prefix_size() {
    let tmp = tempdir().unwrap();
    let prefix = tmp.path().join("my_prefix");
    let drive_c = prefix.join("drive_c");
    fs::create_dir_all(&drive_c).unwrap();

    let mut f1 = File::create(drive_c.join("file1.bin")).unwrap();
    f1.write_all(&vec![0u8; 1024]).unwrap();

    let mut f2 = File::create(drive_c.join("file2.bin")).unwrap();
    f2.write_all(&vec![0u8; 2048]).unwrap();

    let size = calculate_prefix_size(&prefix);
    assert_eq!(size, 3072);
}

#[test]
fn test_hardware_fingerprint_lock_and_full_pipeline_nesting() {
    let tmp = tempdir().unwrap();
    let fp_dir = tmp.path().join("fingerprints");
    let fingerprint = FingerprintManager::new(Some(fp_dir.clone()));

    let game_name = "Elden Ring";
    let snapshot_res = fingerprint.snapshot(game_name);
    assert!(snapshot_res.is_ok());

    let snap_dir = fingerprint.snapshot_dir(game_name);
    assert!(snap_dir.exists());

    let bwrap_args = fingerprint.get_bwrap_args(game_name);
    if which::which("bwrap").is_ok() {
        assert!(!bwrap_args.is_empty());
        assert!(bwrap_args[0].contains("bwrap"));
        assert!(bwrap_args.contains(&"--ro-bind".to_string()));
        assert!(bwrap_args.contains(&"/".to_string()));
        assert!(bwrap_args.contains(&"--dev-bind".to_string()));
        assert!(bwrap_args.contains(&"--proc".to_string()));
        assert!(bwrap_args.contains(&"/proc".to_string()));
        assert_eq!(bwrap_args.last().map(|s| s.as_str()), Some("--"));
    }

    // Now test full hierarchy with bwrap lock, wrapper, mangohud, and gamescope enabled
    let game = GameConfig {
        name: game_name.to_string(),
        exe_path: "/games/eldenring/eldenring.exe".to_string(),
        launch_args: "-fullscreen".to_string(),
        wrapper_command: "env VKD3D_CONFIG=dxr".to_string(),
        fingerprint_lock: true,
        enable_mangohud: true,
        enable_gamescope: true,
        gamescope_settings: GamescopeSettings {
            game_width: "1920".to_string(),
            game_height: "1080".to_string(),
            upscale_width: "2560".to_string(),
            upscale_height: "1440".to_string(),
            upscale_method: "fsr".to_string(),
            window_type: "fullscreen".to_string(),
            fps_limiter: "120".to_string(),
            fps_limiter_no_focus: "".to_string(),
            enable_force_grab_cursor: true,
            additional_options: "".to_string(),
        },
        ..Default::default()
    };

    let cmd = build_command(&game, "/usr/bin/umu-run", "test-uuid-1234", &fingerprint);

    // Verify innermost payload exists
    let umu_pos = cmd
        .iter()
        .position(|s| s == "/usr/bin/umu-run")
        .expect("umu-run must be present");
    let exe_pos = cmd
        .iter()
        .position(|s| s == "/games/eldenring/eldenring.exe")
        .expect("exe must be present");
    let arg_pos = cmd
        .iter()
        .position(|s| s == "-fullscreen")
        .expect("arg must be present");
    assert!(umu_pos < exe_pos);
    assert!(exe_pos < arg_pos);

    // If bwrap is available, it must wrap umu-run
    if which::which("bwrap").is_ok() {
        let bwrap_pos = cmd
            .iter()
            .position(|s| s.contains("bwrap"))
            .expect("bwrap must be present");
        assert!(bwrap_pos < umu_pos);

        // Wrapper must wrap bwrap
        let wrapper_pos = cmd
            .iter()
            .position(|s| s == "VKD3D_CONFIG=dxr")
            .expect("wrapper must be present");
        assert!(wrapper_pos < bwrap_pos);

        // MangoHUD (if present) must wrap wrapper
        if which::which("mangohud").is_ok() {
            let mango_pos = cmd
                .iter()
                .position(|s| s == "mangohud")
                .expect("mangohud must be present");
            assert!(mango_pos < wrapper_pos);

            // Gamescope (if present) must wrap mangohud
            if which::which("gamescope").is_ok() {
                let gs_pos = cmd
                    .iter()
                    .position(|s| s.contains("gamescope"))
                    .expect("gamescope must be present");
                assert!(gs_pos < mango_pos);
            }
        }
    }

    // Clear snapshot
    fingerprint.clear(game_name);
    assert!(!snap_dir.exists());
}
