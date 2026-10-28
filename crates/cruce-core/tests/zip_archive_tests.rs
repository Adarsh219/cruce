use cruce_core::zip_archive::extract_zip_mod;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_extract_zip_mod_contents() {
    let tmp = tempdir().unwrap();
    let zip_path = tmp.path().join("mod.zip");
    let dest_dir = tmp.path().join("game_install");

    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Add dummy dxgi.dll
        zip.start_file("bin/x64/dxgi.dll", options).unwrap();
        zip.write_all(b"MOCK_DXGI_DLL").unwrap();

        // Add dummy d3d11.dll
        zip.start_file("d3d11.dll", options).unwrap();
        zip.write_all(b"MOCK_D3D11_DLL").unwrap();

        // Add dummy patched game.exe
        zip.start_file("bin/x64/game_patched.exe", options).unwrap();
        zip.write_all(b"MOCK_EXE_BINARY").unwrap();

        // Add readme text
        zip.start_file("README.txt", options).unwrap();
        zip.write_all(b"Mod instructions").unwrap();

        zip.finish().unwrap();
    }

    let result = extract_zip_mod(&zip_path, &dest_dir).expect("Extraction should succeed");

    assert!(result.dlls.contains(&"dxgi".to_string()));
    assert!(result.dlls.contains(&"d3d11".to_string()));
    assert_eq!(result.dlls.len(), 2);

    assert_eq!(result.exes.len(), 1);
    assert!(result.exes[0].ends_with("game_patched.exe"));

    // Verify files on disk
    assert!(dest_dir.join("bin/x64/dxgi.dll").is_file());
    assert!(dest_dir.join("d3d11.dll").is_file());
    assert!(dest_dir.join("bin/x64/game_patched.exe").is_file());
    assert!(dest_dir.join("README.txt").is_file());
}

#[test]
fn test_extract_zip_mod_path_traversal_protection() {
    let tmp = tempdir().unwrap();
    let zip_path = tmp.path().join("malicious.zip");
    let dest_dir = tmp.path().join("safe_dir");

    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Attempt path traversal
        let res = zip.start_file("../evil.sh", options);
        if res.is_ok() {
            let _ = zip.write_all(b"rm -rf /");
        }
        let _ = zip.finish();
    }

    let result = extract_zip_mod(&zip_path, &dest_dir);
    // Either zip crate or our ensure_within_dir must block traversal
    if let Ok(res) = result {
        assert!(!tmp.path().join("evil.sh").exists());
        let _ = res;
    }
}
