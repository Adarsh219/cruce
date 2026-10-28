use cruce_core::artwork::{ArtworkManager, VARIANT_HEADER, VARIANT_HERO, VARIANT_PORTRAIT};
use cruce_core::paths::ensure_within_dir;
use cruce_core::proton_manager::{
    extract_tarball_safe, guess_runner_source, natural_sort_key, ProtonManager,
};
use cruce_core::steam_api::SteamApi;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_normalize_name() {
    assert_eq!(
        SteamApi::normalize_name("The Witcher 3: Wild Hunt"),
        "the witcher 3 wild hunt"
    );
    assert_eq!(
        SteamApi::normalize_name("DOOM: Eternal (Deluxe Edition)!"),
        "doom eternal deluxe edition"
    );
    assert_eq!(
        SteamApi::normalize_name("Cyberpunk_2077---v2.0"),
        "cyberpunk 2077 v2 0"
    );
    assert_eq!(SteamApi::normalize_name("   "), "");
}

#[test]
fn test_ancestor_search_terms_and_exe_candidates() {
    let exe_path = Path::new("/storage/games/The Witcher 3/bin/x64/witcher3.exe");
    let (terms, exe_name) = SteamApi::exe_candidates(exe_path, "The Witcher 3");

    assert_eq!(exe_name.as_deref(), Some("witcher3.exe"));
    assert!(terms.contains(&"witcher3".to_string()));
    assert!(terms.contains(&"The Witcher 3".to_string()));
    // Ancestors "bin" and "x64" are generic directories and should be omitted
    assert!(!terms.contains(&"bin".to_string()));
    assert!(!terms.contains(&"x64".to_string()));
}

#[test]
fn test_find_ancestor_by_name() {
    let exe =
        Path::new("/mnt/games/SteamLibrary/steamapps/common/The Witcher 3/bin/x64/witcher3.exe");
    let found = SteamApi::find_ancestor_by_name(exe, "The Witcher 3");
    assert!(found.is_some());
    assert_eq!(
        found.unwrap().to_str().unwrap(),
        "/mnt/games/SteamLibrary/steamapps/common/The Witcher 3"
    );

    // Case-insensitivity check
    let found_case = SteamApi::find_ancestor_by_name(exe, "the witcher 3");
    assert!(found_case.is_some());

    // Non-existent directory
    let not_found = SteamApi::find_ancestor_by_name(exe, "Cyberpunk 2077");
    assert!(not_found.is_none());
}

#[test]
fn test_artwork_keys_and_paths() {
    let tmp = tempdir().unwrap();
    let manager = ArtworkManager::new(tmp.path().to_path_buf());

    // 1. Path key (SHA1 prefix)
    let key_path = ArtworkManager::artwork_key("/games/witcher3/witcher3.exe", None, "Witcher 3");
    assert!(key_path.starts_with("exe_"));
    assert_eq!(key_path.len(), 4 + 16);

    // 2. AppID key
    let key_app = ArtworkManager::artwork_key("", Some("292030"), "Witcher 3");
    assert_eq!(key_app, "app_292030");

    // 3. Game name fallback
    let key_name = ArtworkManager::artwork_key("", None, "The Witcher 3: Wild Hunt");
    assert_eq!(key_name, "The_Witcher_3_Wild_Hunt");

    // Paths for variants
    let header_p =
        manager.get_artwork_path("/games/witcher3/witcher3.exe", None, "", VARIANT_HEADER);
    assert!(header_p.ends_with("header.jpg"));

    let portrait_p =
        manager.get_artwork_path("/games/witcher3/witcher3.exe", None, "", VARIANT_PORTRAIT);
    assert!(portrait_p.ends_with("portrait.jpg"));

    let hero_p = manager.get_artwork_path("/games/witcher3/witcher3.exe", None, "", VARIANT_HERO);
    assert!(hero_p.ends_with("hero.jpg"));
}

#[test]
fn test_artwork_migration() {
    let tmp = tempdir().unwrap();
    let manager = ArtworkManager::new(tmp.path().to_path_buf());

    let old_path = "/games/old_folder/game.exe";
    let new_path = "/games/new_folder/game.exe";

    let old_dir = manager.get_game_artwork_dir(old_path, None, "");
    fs::create_dir_all(&old_dir).unwrap();
    let art_file = old_dir.join("header.jpg");
    fs::write(&art_file, b"fake_artwork_binary_payload").unwrap();

    assert!(old_dir.exists());

    // Migrate
    let migrated = manager.migrate_artwork(old_path, new_path);
    assert!(migrated);

    let new_dir = manager.get_game_artwork_dir(new_path, None, "");
    assert!(new_dir.exists());
    assert!(!old_dir.exists());
    assert!(new_dir.join("header.jpg").exists());
}

#[test]
fn test_proton_runner_source_guessing_and_sorting() {
    assert_eq!(guess_runner_source("GE-Proton9-25"), "ge");
    assert_eq!(guess_runner_source("ge-proton8-10"), "ge");
    assert_eq!(guess_runner_source("UMU-Proton-9.0-3"), "umu");
    assert_eq!(guess_runner_source("proton-cachyos-9.0"), "cachy");

    // Natural sort: 9-25 must sort higher than 9-9
    let k_9_25 = natural_sort_key("GE-Proton9-25");
    let k_9_9 = natural_sort_key("GE-Proton9-9");
    let k_8_25 = natural_sort_key("GE-Proton8-25");

    assert!(k_9_25 > k_9_9);
    assert!(k_9_9 > k_8_25);
}

#[test]
fn test_scan_installed_runners() {
    let tmp = tempdir().unwrap();
    let compat_dir = tmp.path().join("compatibilitytools.d");
    fs::create_dir_all(&compat_dir).unwrap();

    // Create runner 1: GE-Proton9-9
    let r1 = compat_dir.join("GE-Proton9-9");
    fs::create_dir_all(&r1).unwrap();
    File::create(r1.join("proton")).unwrap();
    fs::write(r1.join("version"), "9.0-9\n").unwrap();

    // Create runner 2: GE-Proton9-25
    let r2 = compat_dir.join("GE-Proton9-25");
    fs::create_dir_all(&r2).unwrap();
    File::create(r2.join("proton")).unwrap();
    fs::write(r2.join("version"), "9.0-25\n").unwrap();

    // Directory without "proton" should be ignored
    let non_runner = compat_dir.join("random_folder");
    fs::create_dir_all(&non_runner).unwrap();

    let manager = ProtonManager::new(compat_dir);
    let installed = manager.scan_installed();

    assert_eq!(installed.len(), 2);
    // Sort order must place 9-25 before 9-9
    assert_eq!(installed[0].name, "GE-Proton9-25");
    assert_eq!(installed[0].version, "9.0-25");
    assert_eq!(installed[0].source, "ge");
    assert_eq!(installed[1].name, "GE-Proton9-9");
}

#[test]
fn test_tarball_extraction_safe() {
    let tmp = tempdir().unwrap();
    let archive_path = tmp.path().join("test_archive.tar.gz");

    // Create a .tar.gz archive in memory with a top-level directory
    {
        let tar_file = File::create(&archive_path).unwrap();
        let enc = GzEncoder::new(tar_file, Compression::default());
        let mut tar = tar::Builder::new(enc);

        let data = b"#!/bin/bash\necho 'Proton test binary'\n";
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();

        tar.append_data(&mut header, "GE-Proton9-25/proton", &data[..])
            .unwrap();

        let vdata = b"9.0-25\n";
        let mut vheader = tar::Header::new_gnu();
        vheader.set_size(vdata.len() as u64);
        vheader.set_mode(0o644);
        vheader.set_cksum();

        tar.append_data(&mut vheader, "GE-Proton9-25/version", &vdata[..])
            .unwrap();

        tar.finish().unwrap();
    }

    let extract_dir = tmp.path().join("extracted");
    let root = extract_tarball_safe(&archive_path, &extract_dir).unwrap();

    assert!(root.exists());
    assert!(root.join("proton").exists());
    assert!(root.join("version").exists());
    let v_read = fs::read_to_string(root.join("version")).unwrap();
    assert_eq!(v_read.trim(), "9.0-25");
}

#[test]
fn test_zip_slip_traversal_detection() {
    let base_dir = Path::new("/home/user/.steam/steam/compatibilitytools.d");
    let safe_child = Path::new("/home/user/.steam/steam/compatibilitytools.d/GE-Proton9-25/proton");
    let malicious = Path::new("/home/user/.steam/steam/compatibilitytools.d/../../etc/shadow");

    assert!(ensure_within_dir(base_dir, safe_child).is_ok());
    assert!(ensure_within_dir(base_dir, malicious).is_err());

    let relative_safe = Path::new("GE-Proton/files/bin/wine");
    let relative_malicious = Path::new("../../../../../etc/passwd");
    assert!(ensure_within_dir(base_dir, relative_safe).is_ok());
    assert!(ensure_within_dir(base_dir, relative_malicious).is_err());
}
