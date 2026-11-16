use std::{fs, process::Command};

#[test]
fn direct_bbl_project_accepts_utf8_bom_in_system_preset_resources() {
    let work = tempfile::tempdir().unwrap();
    let profiles = work.path().join("resources/profiles");
    fs::create_dir_all(&profiles).unwrap();
    fs::write(
        profiles.join("Test.json"),
        b"\xef\xbb\xbf{\"name\":\"Test\",\"version\":\"1.0.0\",\"machine_model_list\":[],\"machine_list\":[],\"process_list\":[],\"filament_list\":[{\"name\":\"Test PLA\",\"sub_path\":\"filament/test.json\"}]}",
    ).unwrap();
    fs::create_dir_all(profiles.join("Test/filament")).unwrap();
    fs::write(
        profiles.join("Test/filament/test.json"),
        b"\xef\xbb\xbf{\"type\":\"filament\",\"name\":\"Test PLA\",\"instantiation\":\"true\",\"filament_type\":[\"PLA\"]}",
    ).unwrap();
    let artifacts =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-system-catalog-bom");
    fs::create_dir_all(&artifacts).unwrap();
    let gcode = artifacts.join("ares.gcode");
    let result = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--plate", "1", "--resources-dir"])
        .arg(work.path().join("resources"))
        .args(["-o"])
        .arg(&gcode)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/orca_cli_ender3.3mf"
        ))
        .output()
        .unwrap();
    fs::write(artifacts.join("stdout.log"), &result.stdout).unwrap();
    fs::write(artifacts.join("stderr.log"), &result.stderr).unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        fs::read(&gcode)
            .unwrap()
            .starts_with(b"; HEADER_BLOCK_START\n")
    );
    eprintln!("artifact: {}", artifacts.display());
}
