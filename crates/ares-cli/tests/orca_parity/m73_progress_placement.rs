//! M73 progress placement (`GCodeProcessor::TimeMachine::calculate_time`
//! `g1_times_cache` push at `GCode/GCodeProcessor.cpp:748` — gated by the
//! Normal-mode move-vertex filter `:656-662` (`Extrude`/`Travel`/`Wipe`
//! only; zero-displacement `:5271` and E-only non-wipe blocks never reach
//! the push) — consumed by the post-process `process_line_move` probe
//! `:1463-1475` and `ExportLines::update` `:985-1018`). The witness is the
//! unedited `orca_cli_ender3.3mf` slice: every `M73` must land on the same
//! G-code line as Native (historically Ares deferred `M73 P10 R7` two lines
//! past the `G1 X114.69 Y114.69 F9000` travel, and later ports over-included
//! F-only lines — both fixed by the `:656-662` retention port). The
//! comparison asserts the exact (preceding line, M73 line) sequence of both
//! sides — placement only, byte-exact, no normalization.
use std::fs;

use crate::runner::{OrcaRunner, stages};

const PROJECT: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ender3.3mf");

/// Every `M73 ` line paired with the line immediately before it.
fn progress_sequence(gcode: &str) -> Vec<(&str, &str)> {
    let lines: Vec<&str> = gcode.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with("M73 "))
        .map(|(index, line)| {
            (
                index
                    .checked_sub(1)
                    .and_then(|prev| lines.get(prev))
                    .copied()
                    .unwrap_or(""),
                *line,
            )
        })
        .collect()
}

#[test]
fn m73_progress_lines_land_on_the_same_gcode_lines_as_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("m73-placement-")
        .tempdir_in(&root)
        .unwrap()
        .keep();
    let input = directory.join("input.3mf");
    let output = directory.join("orca");
    fs::write(&input, PROJECT).unwrap();
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        output.to_str().unwrap().to_owned(),
        input.to_str().unwrap().to_owned(),
    ];
    stages::run(
        runner.bin(),
        args.iter().cloned(),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let native = fs::read_to_string(output.join("plate_1.gcode")).unwrap();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let ares = runtime
        .block_on(ares_core::slice_project(
            PROJECT,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let ares = String::from_utf8(ares).unwrap();

    let native_sequence = progress_sequence(&native);
    let ares_sequence = progress_sequence(&ares);
    fs::write(
        directory.join("sequences.txt"),
        format!(
            "native ({} M73 lines):\n{}\n\nares ({} M73 lines):\n{}\n",
            native_sequence.len(),
            native_sequence
                .iter()
                .map(|(prev, m73)| format!("{prev}\n{m73}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ares_sequence.len(),
            ares_sequence
                .iter()
                .map(|(prev, m73)| format!("{prev}\n{m73}"))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
    )
    .unwrap();

    assert!(
        native_sequence.len() >= 20,
        "native witness lost its progress lines: {}",
        native_sequence.len()
    );
    for (index, (native_pair, ares_pair)) in native_sequence.iter().zip(&ares_sequence).enumerate()
    {
        assert_eq!(
            native_pair, ares_pair,
            "M73 pair {index} places on a different line\nnative: {native_pair:?}\nares:   {ares_pair:?}"
        );
    }
    assert_eq!(
        native_sequence.len(),
        ares_sequence.len(),
        "M73 line count differs\nnative: {}\nares:   {}",
        native_sequence.len(),
        ares_sequence.len()
    );
}
