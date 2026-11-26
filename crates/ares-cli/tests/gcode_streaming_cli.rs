use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn fixture_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

fn has_tmp_files(dir: &Path) -> bool {
    fs::read_dir(dir)
        .expect("read output dir")
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.ends_with(".tmp"))
        })
}

#[tokio::test]
async fn cli_streaming_paths_match_core_output_without_leftover_tmp_files() {
    let fixture = fixture_path("tests/parity/validation/bbl-toolchange-boundary.3mf");
    let input_bytes = fs::read(&fixture).expect("read fixture");
    let expected = ares_core::slice_project_plate_validated(
        &input_bytes,
        ares_core::GenerationMetadata::default(),
        Some(1),
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .expect("core validated slice succeeds");

    let temp = tempfile::tempdir().expect("tempdir");
    let single_out = temp.path().join("single.gcode");
    Command::cargo_bin("ares")
        .expect("ares binary")
        .args([
            "slice",
            fixture.to_str().unwrap(),
            "-o",
            single_out.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(&single_out).expect("read single.gcode"), expected);
    assert!(!has_tmp_files(temp.path()));

    let direct_dir = temp.path().join("direct");
    fs::create_dir_all(&direct_dir).expect("mkdir direct");
    Command::cargo_bin("ares")
        .expect("ares binary")
        .args([
            "--slice",
            "1",
            "--outputdir",
            direct_dir.to_str().unwrap(),
            fixture.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(
        fs::read(direct_dir.join("plate_1.gcode")).expect("read direct plate_1.gcode"),
        expected
    );
    assert!(!has_tmp_files(&direct_dir));

    let plates_dir = temp.path().join("plates");
    Command::cargo_bin("ares")
        .expect("ares binary")
        .args([
            "slice",
            "--slice",
            "1",
            "--outputdir",
            plates_dir.to_str().unwrap(),
            fixture.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(
        fs::read(plates_dir.join("plate_1.gcode")).expect("read plates plate_1.gcode"),
        expected
    );
    assert!(!has_tmp_files(&plates_dir));
}

#[test]
fn cli_streaming_leaves_no_partial_or_clobbered_file_on_validation_error() {
    let fixture = fixture_path("tests/parity/validation/bbl-toolchange-boundary.3mf");
    let temp = tempfile::tempdir().expect("tempdir");

    let missing_out = temp.path().join("should-not-exist.gcode");
    Command::cargo_bin("ares")
        .expect("ares binary")
        .args([
            "slice",
            "--overrides",
            r#"{"layer_height":"99.0"}"#,
            "-o",
            missing_out.to_str().unwrap(),
            fixture.to_str().unwrap(),
        ])
        .assert()
        .failure();
    assert!(
        !missing_out.exists(),
        "validation error must not create output"
    );
    assert!(!has_tmp_files(temp.path()));

    let sentinel_out = temp.path().join("existing.gcode");
    let sentinel = b"; untouched sentinel\n";
    fs::write(&sentinel_out, sentinel).expect("write sentinel");
    Command::cargo_bin("ares")
        .expect("ares binary")
        .args([
            "slice",
            "--plate",
            "99",
            "-o",
            sentinel_out.to_str().unwrap(),
            fixture.to_str().unwrap(),
        ])
        .assert()
        .failure();
    assert_eq!(
        fs::read(&sentinel_out).expect("read sentinel"),
        sentinel,
        "slice error must not overwrite existing output file"
    );
    assert!(!has_tmp_files(temp.path()));

    let direct_dir = temp.path().join("direct-err");
    fs::create_dir_all(&direct_dir).expect("mkdir direct-err");
    Command::cargo_bin("ares")
        .expect("ares binary")
        .args([
            "--slice",
            "99",
            "--outputdir",
            direct_dir.to_str().unwrap(),
            fixture.to_str().unwrap(),
        ])
        .assert()
        .failure();
    assert!(!direct_dir.join("plate_99.gcode").exists());
    assert!(!has_tmp_files(&direct_dir));
}

#[cfg(unix)]
#[test]
fn cli_streaming_writes_directly_to_non_regular_target() {
    let fixture = fixture_path("tests/parity/validation/bbl-toolchange-boundary.3mf");
    Command::cargo_bin("ares")
        .expect("ares binary")
        .args(["slice", fixture.to_str().unwrap(), "-o", "/dev/null"])
        .assert()
        .success();
}
