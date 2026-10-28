use crate::error::{CruceError, Result};
use crate::process::detached_fork;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Executes a Wine helper utility inside the specified Wine prefix using UMU.
fn launch_wine_tool(
    tool_command: &[&str],
    umu_bin: &str,
    proton_path: &str,
    prefix_path: &Path,
) -> Result<u32> {
    if !prefix_path.exists() {
        fs::create_dir_all(prefix_path).map_err(|e| CruceError::Io {
            path: prefix_path.to_path_buf(),
            source: e,
        })?;
    }

    let mut env: HashMap<String, String> = std::env::vars().collect();
    crate::env_builder::strip_launch_env(&mut env);

    env.insert(
        "WINEPREFIX".to_string(),
        prefix_path.to_string_lossy().to_string(),
    );
    env.insert("PROTONPATH".to_string(), proton_path.to_string());
    env.insert("GAMEID".to_string(), "0".to_string());
    env.insert("SteamAppId".to_string(), "0".to_string());
    env.insert("SteamGameId".to_string(), "0".to_string());

    let mut cmd = vec![umu_bin.to_string()];
    for part in tool_command {
        cmd.push(part.to_string());
    }

    let log_path = prefix_path.join("cruce_tool.log");
    detached_fork(&cmd, &env, prefix_path, &log_path)
}

/// Launch Winetricks GUI in the given prefix.
pub fn launch_winetricks(umu_bin: &str, proton_path: &str, prefix_path: &Path) -> Result<u32> {
    launch_wine_tool(&["winetricks", "--gui"], umu_bin, proton_path, prefix_path)
}

/// Launch Winecfg configuration dialog in the given prefix.
pub fn launch_winecfg(umu_bin: &str, proton_path: &str, prefix_path: &Path) -> Result<u32> {
    launch_wine_tool(&["winecfg"], umu_bin, proton_path, prefix_path)
}

/// Launch Windows Registry Editor (regedit) in the given prefix.
pub fn launch_regedit(umu_bin: &str, proton_path: &str, prefix_path: &Path) -> Result<u32> {
    launch_wine_tool(&["regedit"], umu_bin, proton_path, prefix_path)
}

/// Launch Windows Task Manager (taskmgr) in the given prefix.
pub fn launch_taskmgr(umu_bin: &str, proton_path: &str, prefix_path: &Path) -> Result<u32> {
    launch_wine_tool(&["taskmgr"], umu_bin, proton_path, prefix_path)
}

/// Calculate the total disk space in bytes consumed by a Wine prefix.
pub fn calculate_prefix_size(prefix_path: &Path) -> u64 {
    if !prefix_path.exists() {
        return 0;
    }

    WalkDir::new(prefix_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}
