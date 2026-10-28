use cruce_core::save_manager::SaveManager;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_save_backup_and_restore_lifecycle() {
    let tmp = tempdir().unwrap();
    let prefix = tmp.path().join("test_prefix");

    // Create realistic Wine prefix user structure
    let saves_dir = prefix.join("drive_c/users/steamuser/Saved Games/MyGame");
    fs::create_dir_all(&saves_dir).unwrap();
    fs::write(saves_dir.join("save1.dat"), b"MOCK_SAVE_DATA_HERO_LEVEL_99").unwrap();

    let appdata = prefix.join("drive_c/users/steamuser/AppData/Local/MyGame");
    fs::create_dir_all(&appdata).unwrap();
    fs::write(appdata.join("config.ini"), b"resolution=1920x1080").unwrap();

    // 1. Back up saves
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let game_name = format!("My Test Game {}", nonce);
    let backup_dir =
        cruce_core::paths::Paths::backups_dir().join(cruce_core::paths::safe_name(&game_name));
    if backup_dir.exists() {
        let _ = fs::remove_dir_all(&backup_dir);
    }

    let backup_info =
        SaveManager::backup_saves(&prefix, &game_name).expect("Backup should succeed");
    assert!(backup_info.size_bytes > 0);
    assert!(std::path::Path::new(&backup_info.file_path).is_file());

    // 2. Query latest backup
    let latest = SaveManager::get_latest_backup(&game_name).expect("Should find latest backup");
    assert_eq!(latest.file_path, backup_info.file_path);

    // 3. Clear prefix user data to simulate loss
    fs::remove_dir_all(prefix.join("drive_c/users")).unwrap();
    assert!(!saves_dir.exists());

    // 4. Restore from backup
    SaveManager::restore_saves(&prefix, std::path::Path::new(&backup_info.file_path))
        .expect("Restore should succeed");

    // 5. Verify restored files
    assert!(saves_dir.join("save1.dat").exists());
    let restored_data = fs::read(saves_dir.join("save1.dat")).unwrap();
    assert_eq!(restored_data, b"MOCK_SAVE_DATA_HERO_LEVEL_99");

    assert!(appdata.join("config.ini").exists());
    let restored_config = fs::read(appdata.join("config.ini")).unwrap();
    assert_eq!(restored_config, b"resolution=1920x1080");

    let _ = fs::remove_dir_all(&backup_dir);
}
