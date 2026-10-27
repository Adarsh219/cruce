use cruce_core::process::{
    detached_fork, is_game_running, is_pid_alive, scan_uuid_pids, stop_game_session,
};
use cruce_core::screensaver::ScreensaverGuard;
use cruce_core::state_tracker::GameStateTracker;
use cruce_core::types::GameState;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn test_game_state_tracker_transitions() {
    let tracker = GameStateTracker::new();
    let game = "Cyberpunk 2077";

    // Initial state is Idle
    assert_eq!(tracker.get(game), GameState::Idle);

    // Valid transition: Idle -> Launching
    assert!(tracker.transition(game, GameState::Launching));
    assert_eq!(tracker.get(game), GameState::Launching);

    // Invalid transition: Launching -> Stopping (must go to Running or Idle first)
    assert!(!tracker.transition(game, GameState::Stopping));
    assert_eq!(tracker.get(game), GameState::Launching);

    // Valid transition: Launching -> Running
    assert!(tracker.transition(game, GameState::Running));
    assert_eq!(tracker.get(game), GameState::Running);

    // Valid transition: Running -> Stopping
    assert!(tracker.transition(game, GameState::Stopping));
    assert_eq!(tracker.get(game), GameState::Stopping);

    // Valid transition: Stopping -> Idle
    assert!(tracker.transition(game, GameState::Idle));
    assert_eq!(tracker.get(game), GameState::Idle);

    // Crash recovery: force_idle
    tracker.transition(game, GameState::Launching);
    tracker.force_idle(game);
    assert_eq!(tracker.get(game), GameState::Idle);
}

#[test]
fn test_game_state_tracker_concurrent_threads() {
    let tracker = Arc::new(GameStateTracker::new());
    let mut handles = Vec::new();

    for i in 0..10 {
        let t = Arc::clone(&tracker);
        handles.push(thread::spawn(move || {
            let game_name = format!("Game_{}", i);
            assert!(t.transition(&game_name, GameState::Launching));
            assert!(t.transition(&game_name, GameState::Running));
            assert!(t.transition(&game_name, GameState::Stopping));
            assert!(t.transition(&game_name, GameState::Idle));
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(tracker.active_games().len(), 0);
}

#[test]
fn test_pid_liveness() {
    let my_pid = std::process::id();
    assert!(is_pid_alive(my_pid));

    // Non-existent PID
    assert!(!is_pid_alive(4_194_300));
}

#[test]
fn test_scan_uuid_pids_finds_tagged_process() {
    let tmp = tempdir().unwrap();
    let log_file = tmp.path().join("child_uuid.log");
    let cwd = tmp.path();

    let unique_uuid = Uuid::new_v4().to_string();
    let cmd = vec![
        "/usr/bin/sh".to_string(),
        "-c".to_string(),
        "sleep 1".to_string(),
    ];
    let mut env = HashMap::new();
    env.insert("CRUCIBLE_GAME_ID".to_string(), unique_uuid.clone());

    let child_pid = detached_fork(&cmd, &env, cwd, &log_file).unwrap();
    assert!(is_pid_alive(child_pid));
    assert!(is_game_running(child_pid, &unique_uuid));

    let found_pids = scan_uuid_pids(&unique_uuid);
    assert!(
        found_pids.contains(&child_pid),
        "Expected /proc scanner to find child PID {} with UUID {}",
        child_pid,
        unique_uuid
    );

    // Non-existent UUID returns empty set
    let non_existent = Uuid::new_v4().to_string();
    let empty_pids = scan_uuid_pids(&non_existent);
    assert!(empty_pids.is_empty());

    // Clean up child
    let _ = stop_game_session(child_pid, &unique_uuid);
}

#[test]
fn test_screensaver_guard_raii() {
    let mut guard = ScreensaverGuard::new(Some(99999));
    assert_eq!(guard.cookie(), Some(99999));

    guard.release();
    assert_eq!(guard.cookie(), None);
}

#[test]
fn test_detached_fork_execution_and_logging() {
    let tmp = tempdir().unwrap();
    let log_file = tmp.path().join("child_output.log");
    let cwd = tmp.path();

    let cmd = vec![
        "/usr/bin/sh".to_string(),
        "-c".to_string(),
        "echo CRUCE_DETACHED_OK; sleep 0.3".to_string(),
    ];
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "1".to_string());

    let grandchild_pid = detached_fork(&cmd, &env, cwd, &log_file).unwrap();
    assert!(grandchild_pid > 0);

    // Process should be alive right after launch
    assert!(is_pid_alive(grandchild_pid));

    // Wait for the short process to finish
    thread::sleep(Duration::from_millis(600));

    // Grandchild has exited
    assert!(!is_pid_alive(grandchild_pid));

    // Verify stdout redirection to log file
    let log_contents = fs::read_to_string(&log_file).unwrap_or_default();
    assert!(
        log_contents.contains("CRUCE_DETACHED_OK"),
        "Expected log to contain child output, got: {}",
        log_contents
    );
}

#[test]
fn test_stop_game_session() {
    let tmp = tempdir().unwrap();
    let log_file = tmp.path().join("sleeper.log");
    let cwd = tmp.path();

    let cmd = vec![
        "/usr/bin/sh".to_string(),
        "-c".to_string(),
        "sleep 30".to_string(),
    ];
    let env = HashMap::new();

    let pid = detached_fork(&cmd, &env, cwd, &log_file).unwrap();
    assert!(is_pid_alive(pid));

    // Stop process
    stop_game_session(pid, "").unwrap();

    // Give reaper a moment to SIGTERM
    thread::sleep(Duration::from_millis(300));

    // Verify process was killed
    assert!(!is_pid_alive(pid));
}
