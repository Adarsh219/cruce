use crate::fingerprint::FingerprintManager;
use crate::gamescope::build_gamescope_command;
use crate::paths::safe_name;
use crate::types::GameConfig;

/// Assembles the final command line for launching a game process.
///
/// Follows Crucible's exact nesting hierarchy (innermost to outermost):
/// `[gamescope] ➔ [systemd-run] ➔ [mangohud] ➔ [gamemoderun] ➔ [wrapper] ➔ [bwrap] ➔ [umu-run] ➔ [game.exe] ➔ [game args]`
pub fn build_command(
    game: &GameConfig,
    umu_bin: &str,
    game_uuid: &str,
    fingerprint: &FingerprintManager,
) -> Vec<String> {
    let mut game_cmd = vec![umu_bin.to_string(), game.exe_path.clone()];

    // Extra user launch arguments
    if !game.launch_args.trim().is_empty() {
        if let Some(args) = shlex::split(&game.launch_args) {
            game_cmd.extend(args);
        }
    }

    // 1. Bubblewrap hardware fingerprint isolation (Denuvo anti-tamper freeze)
    if game.fingerprint_lock {
        let bwrap_args = fingerprint.get_bwrap_args(&game.name);
        if !bwrap_args.is_empty() {
            let mut combined = bwrap_args;
            combined.extend(game_cmd);
            game_cmd = combined;
        }
    }

    // 2. Custom wrapper command
    let wrapper_str = game.wrapper_command.trim();
    if !wrapper_str.is_empty() {
        if let Some(wrapper_parts) = shlex::split(wrapper_str) {
            let mut combined = wrapper_parts;
            combined.extend(game_cmd);
            game_cmd = combined;
        }
    }

    // 3. GameMode OS optimizations
    if game.enable_gamemode && which::which("gamemoderun").is_ok() {
        let mut combined = vec!["gamemoderun".to_string()];
        combined.extend(game_cmd);
        game_cmd = combined;
    }

    // 4. MangoHUD in-game performance overlay
    if game.enable_mangohud && which::which("mangohud").is_ok() {
        let mut combined = vec!["mangohud".to_string()];
        combined.extend(game_cmd);
        game_cmd = combined;
    }

    // 5. Systemd user scope resource isolation
    if which::which("systemd-run").is_ok() {
        let uuid_prefix = if game_uuid.len() >= 8 {
            &game_uuid[..8]
        } else {
            game_uuid
        };
        let unit_name = format!("--unit=game-{}-{}", safe_name(&game.name), uuid_prefix);
        let mut combined = vec![
            "systemd-run".to_string(),
            "--user".to_string(),
            "--scope".to_string(),
            unit_name,
            "--".to_string(),
        ];
        combined.extend(game_cmd);
        game_cmd = combined;
    }

    // 6. Gamescope micro-compositor (Outermost)
    if game.enable_gamescope {
        let gs_cmd = build_gamescope_command(&game.gamescope_settings);
        if !gs_cmd.is_empty() {
            let mut combined = gs_cmd;
            combined.extend(game_cmd);
            game_cmd = combined;
        } else if let Ok(gs_bin) = which::which("gamescope") {
            let mut combined = vec![gs_bin.to_string_lossy().to_string(), "--".to_string()];
            combined.extend(game_cmd);
            game_cmd = combined;
        }
    }

    game_cmd
}
