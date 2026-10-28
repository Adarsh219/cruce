use crate::error::{CruceError, Result};
use crate::paths::{safe_name, Paths};
use crate::pe_icon::extract_exe_icon;
use crate::types::GameConfig;
use md5::{Digest, Md5};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Path to the .desktop shortcut for a game.
pub fn game_desktop_path(game_name: &str) -> PathBuf {
    Paths::applications_dir().join(format!("crucible-{}.desktop", safe_name(game_name)))
}

/// Path to the cached icon inside the game's artwork folder.
pub fn game_icon_path(game_name: &str, exe_path: &str) -> PathBuf {
    let key = if !exe_path.is_empty() {
        let mut hasher = Md5::new();
        hasher.update(exe_path.trim().to_lowercase().as_bytes());
        let digest = format!("{:x}", hasher.finalize());
        format!("exe_{}", &digest[..16.min(digest.len())])
    } else {
        safe_name(game_name)
    };

    Paths::artwork_dir().join(key).join("icon.png")
}

/// Run `update-desktop-database` silently to inform the desktop shell of changes.
pub fn refresh_desktop_database() {
    let app_dir = Paths::applications_dir();
    let _ = Command::new("update-desktop-database")
        .arg(app_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// Creates a `.desktop` shortcut for a game in `~/.local/share/applications/`.
pub fn create_game_shortcut(game: &GameConfig, launcher_bin_path: Option<&str>) -> Result<PathBuf> {
    if game.name.trim().is_empty() {
        return Err(CruceError::Validation("Game has no name".to_string()));
    }

    let desktop_dir = Paths::applications_dir();
    fs::create_dir_all(&desktop_dir).map_err(|e| CruceError::Io {
        path: desktop_dir.clone(),
        source: e,
    })?;

    let desktop_path = game_desktop_path(&game.name);
    let sname = safe_name(&game.name);

    // Escape executable command
    let launcher_bin = launcher_bin_path.unwrap_or("crucible");
    let exec_cmd = format!("\"{}\" --launch \"{}\"", launcher_bin, game.name.replace('"', "\\\""));

    // Resolve or extract icon
    let mut icon_path_str = "crucible".to_string();
    if !game.exe_path.is_empty() {
        let icon_file = game_icon_path(&game.name, &game.exe_path);
        if icon_file.exists() {
            icon_path_str = icon_file.to_string_lossy().to_string();
        } else if let Ok(true) = extract_exe_icon(Path::new(&game.exe_path), &icon_file) {
            icon_path_str = icon_file.to_string_lossy().to_string();
        }
    }

    let sanitized_name: String = game.name
        .chars()
        .filter(|&c| c != '\n' && c != '\r' && c != ';')
        .collect();

    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={}\n\
         Exec={}\n\
         Icon={}\n\
         Categories=Game;\n\
         StartupWMClass=crucible-{}\n\
         StartupNotify=false\n",
        sanitized_name, exec_cmd, icon_path_str, sname
    );

    fs::write(&desktop_path, content).map_err(|e| CruceError::Io {
        path: desktop_path.clone(),
        source: e,
    })?;

    // Make executable (0o755)
    let _ = fs::set_permissions(&desktop_path, fs::Permissions::from_mode(0o755));

    refresh_desktop_database();
    Ok(desktop_path)
}

/// Removes the `.desktop` shortcut for a game and refreshes the desktop database.
pub fn remove_game_shortcut(game_name: &str) -> bool {
    let desktop_path = game_desktop_path(game_name);
    if desktop_path.exists() {
        let removed = fs::remove_file(&desktop_path).is_ok();
        refresh_desktop_database();
        removed
    } else {
        false
    }
}

/// Check if a `.desktop` shortcut exists for a game.
pub fn has_game_shortcut(game_name: &str) -> bool {
    game_desktop_path(game_name).exists()
}

/// Remove stale `.desktop` files for games no longer in the active library.
pub fn cleanup_old_desktop_files(known_game_names: &[&str]) {
    let desktop_dir = Paths::applications_dir();
    let known_safenames: std::collections::HashSet<String> =
        known_game_names.iter().map(|n| safe_name(n)).collect();

    if let Ok(entries) = fs::read_dir(desktop_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let filename = entry.file_name();
            let name_str = filename.to_string_lossy();
            if name_str.starts_with("crucible-") && name_str.ends_with(".desktop") {
                let sname = &name_str["crucible-".len()..name_str.len() - ".desktop".len()];
                if !known_safenames.contains(sname) {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
        refresh_desktop_database();
    }
}
