//! CLI exit-code / channel / result.json parity for the error families
//! (`Utils.hpp:24-59`): the adapter must exit with the POSIX form of the
//! upstream CLI code (negative `return` → 256+code), print the
//! `run found error, return <n>, exit...` line to stdout
//! (`OrcaSlicer.cpp:396-411`), write `<outputdir>/result.json`
//! (`record_exit_reson`, `OrcaSlicer.cpp:416-460`, Linux only), and leave
//! the family's detail on stderr in the native channel shape. Expected
//! artifacts are seeded-native 2.4.2 outputs captured on the same inputs.

use std::process::Command;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/validation/cli-exit")
        .join(name)
}

fn expected(name: &str, kind: &str) -> Vec<u8> {
    std::fs::read(fixture(name).with_file_name(format!("{name}.{kind}"))).unwrap()
}

struct Outcome {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    result_json: Vec<u8>,
}

fn run_cli(input: &str, output_path: &std::path::Path) -> Outcome {
    let out_dir = output_path.parent().unwrap();
    std::fs::create_dir_all(out_dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(output_path)
        .arg(fixture(input).with_file_name(input))
        .output()
        .expect("spawn ares CLI");
    Outcome {
        code: output.status.code().unwrap(),
        stdout: output.stdout,
        stderr: output.stderr,
        result_json: std::fs::read(out_dir.join("result.json")).unwrap_or_default(),
    }
}

fn assert_family(case: &str, input: &str, temp: &std::path::Path) {
    let outcome = run_cli(input, &temp.join("gcode"));
    let code: i32 = String::from_utf8(expected(case, "stdout"))
        .unwrap()
        .parse_line_return_code();
    // `flush_and_exit` returns the negative CLI code; the process exit is
    // its POSIX form 256+n (`OrcaSlicer.cpp:396-411`).
    let expected_exit = 256 + code;
    assert_eq!(
        outcome.code, expected_exit,
        "{case}: exit code {} vs native {}",
        outcome.code, expected_exit
    );
    assert_eq!(
        outcome.stdout,
        expected(case, "stdout"),
        "{case}: stdout channel"
    );
    assert_eq!(
        outcome.stderr,
        expected(case, "stderr"),
        "{case}: stderr channel"
    );
    assert_eq!(
        outcome.result_json,
        expected(case, "result.json"),
        "{case}: result.json bytes"
    );
}

trait ReturnCode {
    fn parse_line_return_code(&self) -> i32;
}

impl ReturnCode for String {
    fn parse_line_return_code(&self) -> i32 {
        // "run found error, return -18, exit...\n"
        let mid = self.split("return ").nth(1).unwrap();
        mid.split(',').next().unwrap().parse().unwrap()
    }
}

#[test]
fn cli_exit_values_family_matches_native() {
    let temp = tempfile::tempdir().unwrap();
    assert_family("values", "values.3mf", temp.path());
}

#[test]
fn cli_exit_o3_range_family_matches_native() {
    let temp = tempfile::tempdir().unwrap();
    assert_family("range", "range.3mf", temp.path());
}

#[test]
fn cli_exit_empty_plate_family_matches_native() {
    let temp = tempfile::tempdir().unwrap();
    assert_family("empty-plate", "empty-plate.3mf", temp.path());
}

#[test]
fn cli_exit_validate_family_matches_native() {
    let temp = tempfile::tempdir().unwrap();
    assert_family("validate", "validate.3mf", temp.path());
}

#[test]
fn cli_exit_postprocess_family_matches_native() {
    let temp = tempfile::tempdir().unwrap();
    assert_family("postprocess", "postprocess.3mf", temp.path());
}

#[test]
fn cli_exit_notfound_family_matches_native() {
    // The native reference was captured against this exact missing path;
    // keep it so the stderr channel stays byte-equal.
    let missing = std::path::Path::new("/tmp/definitely-missing.3mf");
    assert!(!missing.exists(), "test path must stay absent");
    let temp = tempfile::tempdir().unwrap();
    let out_dir = temp.path();
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(out_dir.join("gcode"))
        .arg(missing)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(253));
    assert_eq!(output.stdout, expected("notfound", "stdout"));
    assert_eq!(output.stderr, expected("notfound", "stderr"));
    assert_eq!(
        std::fs::read(out_dir.join("result.json")).unwrap(),
        expected("notfound", "result.json")
    );
}
