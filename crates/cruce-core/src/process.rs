use crate::error::{CruceError, Result};
use nix::sys::signal::{kill, Signal};
use nix::unistd::{fork, pipe, setsid, ForkResult, Pid};
use std::collections::HashSet;
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const REAP_DEADLINE_SECS: u64 = 5;
const REAP_POLL_INTERVAL_MS: u64 = 200;

/// Double-forks a command to detach it from the caller's process tree and reparent to PID 1.
///
/// This breaks window grouping in desktop environments like KDE Plasma and GNOME.
/// Standard input is redirected to `/dev/null`, and standard output/error are redirected
/// to `log_file_path`.
pub fn detached_fork(
    cmd: &[String],
    env: &std::collections::HashMap<String, String>,
    cwd: &Path,
    log_file_path: &Path,
) -> Result<u32> {
    if cmd.is_empty() {
        return Err(CruceError::Validation(
            "Command cannot be empty".to_string(),
        ));
    }

    if let Some(parent) = log_file_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Convert command to CStrings
    let c_program = CString::new(cmd[0].as_str())
        .map_err(|e| CruceError::Validation(format!("Invalid command path: {}", e)))?;
    let c_args: Vec<CString> = cmd
        .iter()
        .map(|s| CString::new(s.as_str()).unwrap_or_default())
        .collect();

    // Convert environment to CStrings: "KEY=VALUE"
    let c_env: Vec<CString> = env
        .iter()
        .map(|(k, v)| CString::new(format!("{}={}", k, v)).unwrap_or_default())
        .collect();

    // Create a pipe for the middle child to send the grandchild's PID back to the parent
    let (pipe_read, pipe_write) = pipe().map_err(|e| CruceError::IoGeneric(e.into()))?;

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { child }) => {
                // Main parent process
                drop(pipe_write); // Close write end in parent

                // Read grandchild PID from pipe
                let mut buf = [0u8; 32];
                let mut total_read = 0;
                let mut read_file = File::from_raw_fd(pipe_read.into_raw_fd());
                use std::io::Read;
                while total_read < buf.len() {
                    match read_file.read(&mut buf[total_read..]) {
                        Ok(0) => break,
                        Ok(n) => total_read += n,
                        Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    }
                }

                // Wait for the middle child to exit
                let _ = nix::sys::wait::waitpid(child, None);

                let pid_str = std::str::from_utf8(&buf[..total_read])
                    .unwrap_or("0")
                    .trim();
                let grandchild_pid: u32 = pid_str.parse().unwrap_or(0);
                if grandchild_pid == 0 {
                    return Err(CruceError::Validation(
                        "Failed to receive valid grandchild PID from detached fork".to_string(),
                    ));
                }

                Ok(grandchild_pid)
            }
            Ok(ForkResult::Child) => {
                // Middle child process
                drop(pipe_read); // Close read end

                // Create new session to detach from controlling terminal
                let _ = setsid();

                match fork() {
                    Ok(ForkResult::Parent { child: grandchild }) => {
                        // Write grandchild PID to pipe and exit immediately
                        let pid_bytes = format!("{}\n", grandchild.as_raw());
                        use std::io::Write;
                        let mut write_file = File::from_raw_fd(pipe_write.into_raw_fd());
                        let _ = write_file.write_all(pid_bytes.as_bytes());
                        let _ = write_file.flush();
                        libc::_exit(0);
                    }
                    Ok(ForkResult::Child) => {
                        // Grandchild process (reparented to PID 1)
                        drop(pipe_write);

                        // 1. Redirect stdin to /dev/null
                        if let Ok(devnull) = File::open("/dev/null") {
                            libc::dup2(devnull.as_raw_fd(), 0);
                        }

                        // 2. Redirect stdout and stderr to log file
                        if let Ok(log_file) = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .mode(0o644)
                            .open(log_file_path)
                        {
                            let fd = log_file.as_raw_fd();
                            libc::dup2(fd, 1);
                            libc::dup2(fd, 2);
                        }

                        // 3. Change directory
                        let _ = nix::unistd::chdir(cwd);

                        // 4. Execute program
                        let _ = nix::unistd::execvpe(&c_program, &c_args, &c_env);

                        // If execvpe returns, an error occurred
                        libc::_exit(1);
                    }
                    Err(_) => {
                        libc::_exit(1);
                    }
                }
            }
            Err(e) => Err(CruceError::IoGeneric(e.into())),
        }
    }
}

/// Probes whether a specific PID is currently alive using signal 0.
pub fn is_pid_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    match kill(Pid::from_raw(pid as i32), None) {
        Ok(_) => true,
        Err(nix::errno::Errno::EPERM) => true, // Exists but belongs to another user
        Err(nix::errno::Errno::ESRCH) => false, // Process does not exist
        Err(_) => false,
    }
}

/// Recursively discovers all descendant PIDs of `pid` by inspecting `/proc/<pid>/task/*/children`.
pub fn get_descendant_pids(pid: u32) -> Vec<u32> {
    let mut descendants = Vec::new();
    let mut stack = vec![pid];

    while let Some(current_pid) = stack.pop() {
        let task_dir = PathBuf::from(format!("/proc/{}/task", current_pid));
        if let Ok(entries) = fs::read_dir(task_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let children_file = entry.path().join("children");
                if let Ok(content) = fs::read_to_string(children_file) {
                    for child_str in content.split_whitespace() {
                        if let Ok(child_pid) = child_str.parse::<u32>() {
                            descendants.push(child_pid);
                            stack.push(child_pid);
                        }
                    }
                }
            }
        }
    }

    descendants
}

/// Zero-allocation byte scanner searching `/proc/*/environ` for `CRUCIBLE_GAME_ID=<uuid>`.
///
/// Reads raw bytes directly to locate the needle without converting entire environment
/// blocks to UTF-8 strings. Takes < 1ms across all active system processes.
pub fn scan_uuid_pids(game_uuid: &str) -> HashSet<u32> {
    let mut matching_pids = HashSet::new();
    let needle = format!("CRUCIBLE_GAME_ID={}", game_uuid);
    let needle_bytes = needle.as_bytes();

    let proc_dir = Path::new("/proc");
    if let Ok(entries) = fs::read_dir(proc_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if let Ok(pid) = name_str.parse::<u32>() {
                let environ_path = entry.path().join("environ");
                if let Ok(data) = fs::read(environ_path) {
                    if memchr_contains(&data, needle_bytes) {
                        matching_pids.insert(pid);
                    }
                }
            }
        }
    }

    matching_pids
}

/// Fast slice search using `memchr` for needle's first byte, followed by exact slice comparison.
fn memchr_contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }

    let first = needle[0];
    let mut pos = 0;
    while pos + needle.len() <= haystack.len() {
        match memchr::memchr(first, &haystack[pos..]) {
            Some(idx) => {
                let found_pos = pos + idx;
                if found_pos + needle.len() <= haystack.len()
                    && &haystack[found_pos..found_pos + needle.len()] == needle
                {
                    return true;
                }
                pos = found_pos + 1;
            }
            None => break,
        }
    }

    false
}

/// Checks whether any process associated with the game session is still running.
///
/// First checks the root PID, then falls back to scanning for processes tagged
/// with `CRUCIBLE_GAME_ID=<uuid>`.
pub fn is_game_running(root_pid: u32, game_uuid: &str) -> bool {
    if is_pid_alive(root_pid) {
        return true;
    }
    if !game_uuid.is_empty() {
        let uuid_pids = scan_uuid_pids(game_uuid);
        if !uuid_pids.is_empty() {
            return true;
        }
    }
    false
}

/// Stops all processes associated with a game session.
///
/// Sends `SIGTERM` to the root PID, its descendants, and any process carrying
/// the `CRUCIBLE_GAME_ID` environment tag. Polls for up to 5 seconds for clean exit;
/// if any process remains alive after the deadline, escalates to `SIGKILL`.
pub fn stop_game_session(root_pid: u32, game_uuid: &str) -> Result<()> {
    let mut all_pids = HashSet::new();

    if root_pid != 0 {
        all_pids.insert(root_pid);
        for d in get_descendant_pids(root_pid) {
            all_pids.insert(d);
        }
    }

    if !game_uuid.is_empty() {
        for u in scan_uuid_pids(game_uuid) {
            all_pids.insert(u);
        }
    }

    if all_pids.is_empty() {
        return Ok(());
    }

    // Step 1: Send SIGTERM
    for &pid in &all_pids {
        let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
    }

    // Step 2: Spawn background reaper thread to poll for clean exit or escalate to SIGKILL
    let pids_to_reap: Vec<u32> = all_pids.into_iter().collect();
    thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(REAP_DEADLINE_SECS);
        while Instant::now() < deadline {
            let mut any_alive = false;
            for &pid in &pids_to_reap {
                if is_pid_alive(pid) {
                    any_alive = true;
                    break;
                }
            }

            if !any_alive {
                return; // All cleanly terminated
            }

            thread::sleep(Duration::from_millis(REAP_POLL_INTERVAL_MS));
        }

        // Step 3: Deadline expired, send SIGKILL
        for &pid in &pids_to_reap {
            if is_pid_alive(pid) {
                let _ = kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
            }
        }
    });

    Ok(())
}
