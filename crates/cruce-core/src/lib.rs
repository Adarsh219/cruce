pub mod artwork;
pub mod command_builder;
pub mod config;
pub mod desktop_shortcuts;
pub mod dll_overrides;
pub mod env_builder;
pub mod error;
pub mod fingerprint;
pub mod game_store;
pub mod gamescope;
pub mod launcher;
pub mod paths;
pub mod pe_icon;
pub mod process;
pub mod proton_manager;
pub mod save_manager;
pub mod screensaver;
pub mod state_tracker;
pub mod steam_api;
pub mod storage;
pub mod types;
pub mod wine_tools;
pub mod zip_archive;

pub use artwork::{
    ArtworkFetchResult, ArtworkManager, VARIANT_HEADER, VARIANT_HERO, VARIANT_PORTRAIT,
};
pub use command_builder::build_command;
pub use config::GlobalConfig;
pub use desktop_shortcuts::{
    cleanup_old_desktop_files, create_game_shortcut, game_desktop_path, game_icon_path,
    has_game_shortcut, refresh_desktop_database, remove_game_shortcut,
};
pub use dll_overrides::build_dll_overrides;
pub use env_builder::{build_env, resolve_appid, strip_launch_env};
pub use error::{CruceError, Result};
pub use fingerprint::FingerprintManager;
pub use game_store::GameStore;
pub use gamescope::build_gamescope_command;
pub use launcher::{GameLauncher, PreparedLaunch};
pub use paths::{
    artwork_safe_name, display_name_from_exe, ensure_within_dir, find_app_id_in_game_dir,
    find_game_root, safe_name, steam_id_for_name, Paths,
};
pub use pe_icon::extract_exe_icon;
pub use process::{
    detached_fork, get_descendant_pids, is_game_running, is_pid_alive, scan_uuid_pids,
    stop_game_session,
};
pub use proton_manager::{
    extract_tarball_safe, guess_runner_source, natural_sort_key, InstalledRunner, ProtonManager,
    ProtonRelease,
};
pub use save_manager::{BackupInfo, SaveManager};
pub use screensaver::{ScreensaverGuard, ScreensaverManager};
pub use state_tracker::GameStateTracker;
pub use steam_api::{SteamApi, SteamCmdRecord};
pub use types::{GameConfig, GameState, GamescopeSettings, GlobalConfigData};
pub use wine_tools::{
    calculate_prefix_size, launch_regedit, launch_taskmgr, launch_winecfg, launch_winetricks,
};
pub use zip_archive::{extract_zip_mod, ExtractedZipContents};
