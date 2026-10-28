use cruce_core::game_store::GameStore;
use cruce_core::launcher::GameLauncher;
use cruce_core::paths::Paths;
use cruce_core::proton_manager::ProtonManager;
use cruce_core::screensaver::ScreensaverGuard;
use cruce_core::state_tracker::GameStateTracker;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[allow(dead_code)]
pub struct ActiveSession {
    pub session_uuid: String,
    pub pid: u32,
    pub screensaver_guard: ScreensaverGuard,
    pub started_at: Instant,
}

pub struct AppState {
    pub store: Arc<Mutex<GameStore>>,
    pub state_tracker: Arc<GameStateTracker>,
    pub launcher: Arc<GameLauncher>,
    pub proton_manager: Arc<ProtonManager>,
    pub cancel_download_flag: Arc<AtomicBool>,
    pub active_sessions: Arc<Mutex<HashMap<String, ActiveSession>>>,
}

impl AppState {
    pub fn new() -> Self {
        let state_tracker = Arc::new(GameStateTracker::new());
        let launcher = Arc::new(GameLauncher::new(state_tracker.clone()));
        let store = Arc::new(Mutex::new(GameStore::new(None, None, None)));
        let proton_manager = Arc::new(ProtonManager::new(Paths::steam_compat_dir()));
        let cancel_download_flag = Arc::new(AtomicBool::new(false));
        let active_sessions = Arc::new(Mutex::new(HashMap::new()));

        Self {
            store,
            state_tracker,
            launcher,
            proton_manager,
            cancel_download_flag,
            active_sessions,
        }
    }

    pub fn new_with_dirs(
        games_dir: std::path::PathBuf,
        prefixes_dir: std::path::PathBuf,
        global_config_path: std::path::PathBuf,
        compat_dir: std::path::PathBuf,
    ) -> Self {
        let state_tracker = Arc::new(GameStateTracker::new());
        let launcher = Arc::new(GameLauncher::new(state_tracker.clone()));
        let store = Arc::new(Mutex::new(GameStore::new(
            Some(games_dir),
            Some(prefixes_dir),
            Some(global_config_path),
        )));
        let proton_manager = Arc::new(ProtonManager::new(compat_dir));
        let cancel_download_flag = Arc::new(AtomicBool::new(false));
        let active_sessions = Arc::new(Mutex::new(HashMap::new()));

        Self {
            store,
            state_tracker,
            launcher,
            proton_manager,
            cancel_download_flag,
            active_sessions,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
