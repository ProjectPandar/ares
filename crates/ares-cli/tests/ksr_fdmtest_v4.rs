#[path = "ksr_fdmtest_v4/golden.rs"]
mod golden;
// Diagnostic-only parsers and behavioral tests, never acceptance fallback.
#[path = "ksr_fdmtest_v4/semantic.rs"]
mod semantic;
#[path = "ksr_fdmtest_v4/tests.rs"]
mod tests;

use std::{fs, path::PathBuf};

use sha2::{Digest, Sha256};

const PROJECT_SHA256: &str = "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9";
const REFERENCE_SHA256: &str = "3f2d028f82b6a50f5244f0bfdb193370975b21c55661575edbf98d78d8aa25e4";
const NORMALIZED_REFERENCE_SHA256: &str =
    "164e7d1383d6086c368fd1110ad14fb3bb6cd105696ada9b3f12523b1ad02d73";

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/ksr_fdmtest_v4")
        .join(name)
}

fn project() -> Vec<u8> {
    fs::read(fixture_path("ksr_fdmtest_v4.project.3mf")).unwrap()
}

fn reference() -> Vec<u8> {
    fs::read(fixture_path("ksr_fdmtest_v4.gcode")).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn count_bytes(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|window| *window == needle)
        .count()
}

fn compare_actual(actual: &[u8]) -> Result<(), String> {
    let directory = tempfile::Builder::new()
        .prefix("ares-ksr-ordered-")
        .tempdir()
        .unwrap()
        .keep();
    for (source, destination) in [
        ("ksr_fdmtest_v4.project.3mf", "input.3mf"),
        ("ksr_fdmtest_v4.initialized-orca.gcode", "orca.gcode"),
        ("initialized-reference.json", "reference.json"),
    ] {
        fs::copy(fixture_path(source), directory.join(destination)).unwrap();
    }
    fs::write(directory.join("ares.gcode"), actual).unwrap();
    eprintln!("complete KSR artifacts: {directory:?}");
    golden::compare_ordered_bytes_generator_only(
        &fs::read(directory.join("orca.gcode")).unwrap(),
        actual,
    )
    .map_err(|error| format!("{error}; complete artifacts {directory:?}"))
}
