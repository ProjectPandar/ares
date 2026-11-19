use std::process::Command;

#[test]
fn cli_legacy_stl_missing_options_preserves_debug_channel() {
    let temp = tempfile::tempdir().unwrap();
    let input =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/parity/cube10.stl");
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(temp.path().join("gcode"))
        .arg(input)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"Error: STL input requires --options\n");
    assert!(!temp.path().join("result.json").exists());
}
