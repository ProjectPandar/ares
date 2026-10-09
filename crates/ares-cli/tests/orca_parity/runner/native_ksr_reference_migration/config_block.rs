//! GCode.cpp:5591-5643 GCode::append_full_config, through public CLI slicing.

use crate::runner::repo_root;
use std::{fs, path::Path};

const START: &[u8] = b"; CONFIG_BLOCK_START\n";
const END: &[u8] = b"; CONFIG_BLOCK_END\n\n";

#[test]
fn config_export_fixture_matches_resolved_task19b3_views_byte_for_byte() {
    let artifacts = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        tempfile::Builder::new()
            .prefix("ksr-config-")
            .tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        tempfile::tempdir()
    }
    .unwrap()
    .keep();
    let source = repo_root().join("tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
    let output = artifacts.join("ares.gcode");
    assert_cmd::Command::cargo_bin("ares")
        .unwrap()
        .args([
            "slice",
            "-o",
            output.to_str().unwrap(),
            source.to_str().unwrap(),
        ])
        .assert()
        .success();
    let actual = fs::read(output).unwrap();
    let initialized =
        fs::read(repo_root().join("tests/ksr_fdmtest_v4/ksr_fdmtest_v4.initialized-orca.gcode"))
            .unwrap();
    compare(block(&initialized), block(&actual), &artifacts);
}

fn block(bytes: &[u8]) -> &[u8] {
    let start = find_once(bytes, START);
    let end = start + find_once(&bytes[start..], END) + END.len();
    &bytes[start..end]
}

fn find_once(haystack: &[u8], needle: &[u8]) -> usize {
    let matches = haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, window)| (window == needle).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "marker occurrence count");
    matches[0]
}

fn compare(expected: &[u8], actual: &[u8], artifacts: &Path) {
    if actual != expected {
        let index = actual
            .iter()
            .zip(expected)
            .position(|(actual, expected)| actual != expected)
            .unwrap_or(actual.len().min(expected.len()));
        panic!(
            "config block differs at byte {index}: expected {} bytes, got {} bytes\nexpected: {:?}\nactual: {:?}\nartifacts: {artifacts:?}",
            expected.len(),
            actual.len(),
            bounded_text(expected, index),
            bounded_text(actual, index),
        );
    }
}

fn bounded_text(bytes: &[u8], index: usize) -> String {
    let start = index.saturating_sub(80);
    let end = (index + 80).min(bytes.len());
    String::from_utf8_lossy(&bytes[start..end]).into_owned()
}
