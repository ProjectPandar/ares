//! `GCodeProcessor::process_G29` (`GCodeProcessor.cpp:6367-6375`) adds its 260 s bed-leveling
//! time for the first measured `G29` of a print only (`docs/architecture/processor-g29-time.md`).
//! A section oracle: the first-layer time estimate must match Native's; the rest of the output
//! is not compared.

use crate::runner::{OrcaRunner, repo_root};

const FIRST_LAYER_TIME: &str = "; estimated first layer printing time (normal mode) = ";

fn first_layer_time(gcode: &[u8]) -> &str {
    let mut lines = std::str::from_utf8(gcode)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(FIRST_LAYER_TIME));
    let line = lines.next().expect("first-layer time estimate");
    assert_eq!(lines.next(), None, "one first-layer time estimate");
    line
}

fn assert_first_layer_time_matches(label: &str, fixture: &str) {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("official v2.5.0 slicing oracle");
    let project = std::fs::read(repo_root().join(fixture)).unwrap();
    let exported = runner
        .fixture_case(&format!("processor-g29-time/{label}"), project)
        .unwrap();
    let native = runner.slice_selected_plate_case(&exported, 1).unwrap();
    let output = exported.output_dir.join("ares.cli.gcode");
    let cli = crate::runner::application::CliCommand::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--plate", "1", "-o"])
        .arg(&output)
        .arg(&exported.project_path)
        .output()
        .unwrap();
    assert!(cli.status.success(), "{label}: CLI {cli:?}");
    let ares = std::fs::read(output).unwrap();
    assert_eq!(
        first_layer_time(&ares),
        first_layer_time(&native.reference),
        "{label}: first-layer time estimate"
    );
}

#[test]
fn mk4_spiral_ranges_count_bed_leveling_once_like_native() {
    assert_first_layer_time_matches(
        "spiral-active-ranges",
        "tests/parity/spiral-layer-ranges/active.3mf",
    );
}

#[test]
fn mk4_aligned_seam_counts_bed_leveling_once_like_native() {
    assert_first_layer_time_matches(
        "rear-seam-aligned-control",
        "tests/parity/rear-seam-visibility/aligned-control.3mf",
    );
}
