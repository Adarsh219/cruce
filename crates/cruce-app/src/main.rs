#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cruce_app::commands;
use cruce_app::state::AppState;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use tauri::Manager;
use tracing::info;

fn get_socket_path() -> std::path::PathBuf {
    let uid = unsafe { libc::getuid() };
    std::env::temp_dir().join(format!("cruce-gui-{}.sock", uid))
}

fn try_notify_existing_instance(launch_cmd: Option<&str>) -> bool {
    let path = get_socket_path();
    if let Ok(mut stream) = UnixStream::connect(&path) {
        let msg = match launch_cmd {
            Some(target) => format!("launch:{}\n", target),
            None => "show\n".to_string(),
        };
        let _ = stream.write_all(msg.as_bytes());
        true
    } else {
        false
    }
}

fn setup_single_instance_listener(app_handle: tauri::AppHandle) {
    let path = get_socket_path();
    let _ = std::fs::remove_file(&path);

    if let Ok(listener) = UnixListener::bind(&path) {
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut buf = [0u8; 256];
                let mut s = stream;
                if let Ok(n) = s.read(&mut buf) {
                    let msg = String::from_utf8_lossy(&buf[..n]);
                    let trimmed = msg.trim();
                    if let Some(target) = trimmed.strip_prefix("launch:") {
                        let target = target.to_string();
                        let app_clone = app_handle.clone();
                        let state = app_handle.state::<AppState>();
                        let _ = commands::launch_game(app_clone, state, target);
                    }
                    if let Some(w) = app_handle.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                    }
                }
            }
        });
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    let launch_target = if args.len() >= 3 && args[1] == "--launch" {
        Some(args[2].clone())
    } else if let Some(pos) = args.iter().position(|a| a == "--launch") {
        args.get(pos + 1).cloned()
    } else {
        None
    };

    // Check if another instance is already running
    if try_notify_existing_instance(launch_target.as_deref()) {
        info!("Existing Cruce instance notified. Focusing window and exiting.");
        std::process::exit(0);
    }

    let state = AppState::new();
    let launch_target_clone = launch_target.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .setup(move |app| {
            setup_single_instance_listener(app.handle().clone());
            if let Some(target) = launch_target_clone {
                let handle = app.handle().clone();
                let state_handle = app.state::<AppState>();
                let _ = commands::launch_game(handle, state_handle, target);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_games,
            commands::get_game,
            commands::save_game,
            commands::add_game,
            commands::remove_game,
            commands::rename_game,
            commands::launch_game,
            commands::stop_game,
            commands::toggle_play,
            commands::get_runners,
            commands::get_installed_runners,
            commands::download_runner,
            commands::cancel_download,
            commands::remove_runner,
            commands::get_settings,
            commands::update_settings,
            commands::open_wine_tool,
            commands::reset_prefix,
            commands::backup_game_saves,
            commands::restore_game_saves,
            commands::get_backup_info,
            commands::rescan_library,
            commands::open_folder,
            commands::toggle_shortcut,
            commands::get_game_logs,
            commands::clear_game_logs,
            commands::open_game_log_dir,
            commands::apply_zip_to_game,
            commands::browse_file,
            commands::browse_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running cruce application");
}
