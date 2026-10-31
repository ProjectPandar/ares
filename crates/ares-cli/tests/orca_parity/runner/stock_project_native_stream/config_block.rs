//! GCode.cpp:5591-5643 GCode::append_full_config, through public CLI slicing.

use super::producer;
use crate::runner::{ExportedCase, cache_identity, repo_root};
use std::{fs, path::Path};

const START: &[u8] = b"; CONFIG_BLOCK_START\n";
const END: &[u8] = b"; CONFIG_BLOCK_END\n\n";

#[test]
fn config_export_fixture_matches_resolved_views_byte_for_byte() {
    let Some(runner) = producer::runner(
        "runner::stock_project_native_stream::config_block::config_export_fixture_matches_resolved_views_byte_for_byte",
    ) else {
        return;
    };
    let source = repo_root().join("tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");
    let project = fs::read(&source).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&project),
        "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9",
        "{:?}",
        runner.work
    );
    let project_path = runner.work.join(source.file_name().unwrap());
    fs::write(&project_path, &project).unwrap();
    let output_dir = runner.work.join("native");
    fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: "ksr/config-block".to_owned(),
            project,
            project_path: project_path.clone(),
            output_dir,
        })
        .unwrap();
    fs::write(runner.work.join("orca.gcode"), &case.reference).unwrap();
    let output = runner.work.join("ares.gcode");
    assert_cmd::Command::cargo_bin("ares")
        .unwrap()
        .args([
            "slice",
            "-o",
            output.to_str().unwrap(),
            project_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let actual = fs::read(output).unwrap();
    compare(block(&case.reference), block(&actual), &runner.work);
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
