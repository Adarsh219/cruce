use crate::command_builder::build_command;
use crate::env_builder::build_env;
use crate::error::{CruceError, Result};
use crate::fingerprint::FingerprintManager;
use crate::game_store::GameStore;
use crate::paths::{safe_name, Paths};
use crate::process::detached_fork;
use crate::screensaver::{ScreensaverGuard, ScreensaverManager};
use crate::state_tracker::GameStateTracker;
use crate::types::{GameConfig, GameState};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

const MIN_NOFILE_LIMIT: u64 = 524_288;

/// Prepared launch context holding all parameters required for execution.
#[derive(Debug, Clone)]
pub struct PreparedLaunch {
    pub game_name: String,
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: PathBuf,
    pub log_file: PathBuf,
    pub session_uuid: String,
}

/// Core Game Launcher coordinating the validate -> prepare -> execute pipeline.
pub struct GameLauncher {
    pub state: Arc<GameStateTracker>,
    pub fingerprint: FingerprintManager,
    pub screensaver: ScreensaverManager,
}

impl GameLauncher {
    pub fn new(state: Arc<GameStateTracker>) -> Self {
        Self {
            state,
            fingerprint: FingerprintManager::new(None),
            screensaver: ScreensaverManager::new(),
        }
    }

    /// Verifies prerequisites (executable path, Proton installation, umu-run binary,
    /// and file descriptor limits) and transitions state from Idle to Launching.
    pub fn validate_launch(
        &self,
        game_name: &str,
        store: &GameStore,
        umu_path: &str,
    ) -> Result<GameConfig> {
        let current_state = self.state.get(game_name);
        if current_state != GameState::Idle {
            return Err(CruceError::Validation(format!(
                "Game '{}' is already in {:?} state.",
                game_name, current_state
            )));
        }

        let resolved = store
            .get_resolved_game(game_name)
            .ok_or_else(|| CruceError::GameNotFound(game_name.to_string()))?;

        // 1. Verify executable exists
        let exe = Path::new(&resolved.exe_path);
        if !exe.exists() {
            return Err(CruceError::Validation(format!(
                "Executable not found:\n{}",
                resolved.exe_path
            )));
        }

        // 2. Verify Proton path exists
        if resolved.proton_path.is_empty() || !Path::new(&resolved.proton_path).is_dir() {
            return Err(CruceError::Validation(
                "No valid Proton version configured. Edit the game and select a Proton version."
                    .to_string(),
            ));
        }

        // 3. Verify umu-run binary exists
        if umu_path.is_empty() || !Path::new(umu_path).exists() {
            return Err(CruceError::Validation(
                "umu-run binary not found. Check that it is installed or bundled.".to_string(),
            ));
        }

        // 4. Verify open file descriptor limit for esync/fsync
        let no_esync = resolved.env_vars.get("PROTON_NO_ESYNC").map(|s| s.as_str()) == Some("1");
        let no_fsync = resolved.env_vars.get("PROTON_NO_FSYNC").map(|s| s.as_str()) == Some("1");
        if !no_esync && !no_fsync {
            check_nofile_limit()?;
        }

        if !self.state.transition(game_name, GameState::Launching) {
            return Err(CruceError::Validation(format!(
                "Game '{}' state changed concurrently",
                game_name
            )));
        }

        Ok(resolved)
    }

    /// Assembles environment variables, wrappers, command-line arguments,
    /// prefix directories, and timestamped log paths for the target game.
    pub fn prepare_launch(
        &self,
        game: &GameConfig,
        umu_path: &str,
        prefixes_dir: &Path,
    ) -> Result<PreparedLaunch> {
        let sname = safe_name(&game.name);

        // Resolve prefix directory
        let prefix_path = if !game.prefix_path.is_empty() {
            PathBuf::from(&game.prefix_path)
        } else {
            prefixes_dir.join(format!("{}prefix", sname))
        };
        let _ = fs::create_dir_all(&prefix_path);

        // Clean broken symlinks in Wine system32/syswow64
        clean_broken_prefix_symlinks(&prefix_path);

        // Generate timestamped log file
        let log_dir = Paths::game_logs_dir(&game.name);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        let log_file = log_dir.join(format!("{}.log", stamp));

        // Assemble environment and command line
        let (env, session_uuid) = build_env(game, &game.proton_path, &prefix_path);
        let command = build_command(game, umu_path, &session_uuid, &self.fingerprint);

        // Resolve working directory (cwd)
        let exe = Path::new(&game.exe_path);
        let cwd = if !game.install_dir.is_empty() && Path::new(&game.install_dir).is_dir() {
            PathBuf::from(&game.install_dir)
        } else {
            exe.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
        };

        Ok(PreparedLaunch {
            game_name: game.name.clone(),
            command,
            env,
            cwd,
            log_file,
            session_uuid,
        })
    }

    /// Spawns the prepared command via detached double-fork, acquires a D-Bus
    /// screensaver inhibitor, and transitions the game state to Running.
    pub fn execute_launch(&self, prepared: &PreparedLaunch) -> Result<(u32, ScreensaverGuard)> {
        match detached_fork(
            &prepared.command,
            &prepared.env,
            &prepared.cwd,
            &prepared.log_file,
        ) {
            Ok(pid) => {
                let guard = self
                    .screensaver
                    .inhibit("Cruce", &format!("Playing {}", prepared.game_name));
                self.state
                    .transition(&prepared.game_name, GameState::Running);
                Ok((pid, guard))
            }
            Err(e) => {
                self.state.force_idle(&prepared.game_name);
                Err(e)
            }
        }
    }
}

/// Checks RLIMIT_NOFILE to ensure esync/fsync will not crash the game.
fn check_nofile_limit() -> Result<()> {
    unsafe {
        let mut rlim = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlim) == 0
            && rlim.rlim_max != libc::RLIM_INFINITY
            && rlim.rlim_max < MIN_NOFILE_LIMIT
        {
            return Err(CruceError::Validation(format!(
                "ESYNC/FSYNC require an open file descriptor limit of at least 524288.\n\
                     Current hard limit is {}.\n\n\
                     Add to /etc/systemd/user.conf or /etc/security/limits.conf:\n  \
                       DefaultLimitNOFILE=1048576\n\n\
                     Or disable ESYNC/FSYNC in the game's advanced settings.",
                rlim.rlim_max
            )));
        }
    }
    Ok(())
}

/// Removes broken symlinks in Wine's system32/syswow64 directories.
fn clean_broken_prefix_symlinks(prefix_path: &Path) {
    let dirs = [
        prefix_path.join("drive_c").join("windows").join("system32"),
        prefix_path.join("drive_c").join("windows").join("syswow64"),
    ];

    for d in &dirs {
        if let Ok(entries) = fs::read_dir(d) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(meta) = fs::symlink_metadata(entry.path()) {
                    if meta.file_type().is_symlink() && !entry.path().exists() {
                        let _ = fs::remove_file(entry.path());
                        debug!("Removed broken Wine prefix symlink: {:?}", entry.path());
                    }
                }
            }
        }
    }
}
