use crate::types::GamescopeSettings;
use std::process::Command;
use tracing::warn;

/// Checks if the installed gamescope is "new style" (>= 3.12, supports -F / --filter flags).
pub fn detect_gamescope_version(gamescope_bin: &str) -> bool {
    let output = Command::new(gamescope_bin).arg("--help").output();
    if let Ok(out) = output {
        let help_text = String::from_utf8_lossy(&out.stdout).to_string()
            + &String::from_utf8_lossy(&out.stderr);
        return help_text.contains("-F, --filter") || help_text.contains("--filter");
    }
    false
}

/// Convert structured Gamescope settings into a CLI command prefix ending with `--`.
/// Returns an empty list if gamescope is not configured or not installed.
pub fn build_gamescope_command(settings: &GamescopeSettings) -> Vec<String> {
    let has_upscale = !settings.game_width.is_empty()
        || !settings.game_height.is_empty()
        || !settings.upscale_width.is_empty()
        || !settings.upscale_height.is_empty()
        || !settings.upscale_method.is_empty()
        || !settings.window_type.is_empty();

    let has_limiter = !settings.fps_limiter.is_empty() || !settings.fps_limiter_no_focus.is_empty();
    let has_misc =
        settings.enable_force_grab_cursor || !settings.additional_options.trim().is_empty();

    if !has_upscale && !has_limiter && !has_misc {
        return Vec::new();
    }

    let gamescope_bin = match which::which("gamescope") {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => {
            warn!("Gamescope enabled but 'gamescope' not found on PATH.");
            return Vec::new();
        }
    };

    let new_version = detect_gamescope_version(&gamescope_bin);
    let mut cmd = vec![gamescope_bin];

    if has_upscale {
        if !settings.game_width.is_empty() {
            cmd.push("-w".to_string());
            cmd.push(settings.game_width.clone());
        }
        if !settings.game_height.is_empty() {
            cmd.push("-h".to_string());
            cmd.push(settings.game_height.clone());
        }
        if !settings.upscale_width.is_empty() {
            cmd.push("-W".to_string());
            cmd.push(settings.upscale_width.clone());
        }
        if !settings.upscale_height.is_empty() {
            cmd.push("-H".to_string());
            cmd.push(settings.upscale_height.clone());
        }

        let method = settings.upscale_method.to_ascii_lowercase();
        match method.as_str() {
            "fsr" => {
                if new_version {
                    cmd.push("-F".to_string());
                    cmd.push("fsr".to_string());
                } else {
                    cmd.push("-U".to_string());
                }
            }
            "nis" => {
                if new_version {
                    cmd.push("-F".to_string());
                    cmd.push("nis".to_string());
                } else {
                    cmd.push("-Y".to_string());
                }
            }
            "integer" => {
                if new_version {
                    cmd.push("-S".to_string());
                    cmd.push("integer".to_string());
                } else {
                    cmd.push("-i".to_string());
                }
            }
            "stretch" if new_version => {
                cmd.push("-S".to_string());
                cmd.push("stretch".to_string());
            }
            _ => {}
        }

        let wt = settings.window_type.to_ascii_lowercase();
        if wt == "fullscreen" {
            cmd.push("-f".to_string());
        } else if wt == "borderless" {
            cmd.push("-b".to_string());
        }
    }

    if has_limiter {
        if !settings.fps_limiter.is_empty() {
            cmd.push("-r".to_string());
            cmd.push(settings.fps_limiter.clone());
        }
        if !settings.fps_limiter_no_focus.is_empty() {
            cmd.push("-o".to_string());
            cmd.push(settings.fps_limiter_no_focus.clone());
        }
    }

    if settings.enable_force_grab_cursor {
        cmd.push("--force-grab-cursor".to_string());
    }

    let extra = settings.additional_options.trim();
    if !extra.is_empty() {
        if let Some(parts) = shlex::split(extra) {
            cmd.extend(parts);
        }
    }

    cmd.push("--".to_string());
    cmd
}
