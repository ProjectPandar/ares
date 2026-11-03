//! Targeted E2E for the non-tower `change_filament_gcode` expansion
//! (`GCode.cpp:7814-7853` + `7884-7948`): extract every expansion block
//! (`======== X2D filament_change gcode` header through
//! `;not set fan changing filament`) from native and ares output and
//! assert block count and byte-for-byte equality — every expanded
//! `L[flush_length]`, `F`, `H`, `T`, `P`, `SYNC T…` parameter included.
//!
//! The fixture is `input-multi-used.3mf` (X2D, two filaments, object-level
//! `sparse_infill_filament_id=2`) narrowed to this slice's surface:
//! `enable_prime_tower=0` (tower-activation `used_filaments` counting is a
//! separate upstream divergence — see /tmp/ares-matrix-clusters.md),
//! `time_lapse_gcode=""` (per-layer timelapse block emission is another
//! unported slice), `z_hop_types`/`filament_z_hop_types` = Normal Lift
//! (the spiral-lift predicate after toolchange diverges independently),
//! `z_hop`/`filament_z_hop` = 0 (the pre-toolchange lift ordering is a
//! separate emit slice), and `disable_m73=1` (the print-time estimate
//! model diverges ~13%, which shifts every `M73 P/R` threshold —
//! `GCodeProcessor.cpp:1024/1035/2125`). The X2D `change_filament_gcode`
//! still exercises `L[flush_length]` and friends — the placeholder
//! family this chain ports.

use std::fs;

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/multi-filament-bridge-anchor-narrowed.3mf");
const INPUT_SHA256: &str = "22d3813ddbbfbc40bd3955fb35d5f6d19b7d8ee23472e7aa2e354e50a23cfe82";

#[test]
fn change_filament_flush_blocks() {
    use sha2::{Digest, Sha256};
    assert_eq!(
        Sha256::digest(INPUT)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        INPUT_SHA256
    );
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("change-filament-blocks-")
        .tempdir()
        .unwrap()
        .keep();
    let input = directory.join("input.3mf");
    fs::write(&input, INPUT).unwrap();
    let output = directory.join("orca");
    fs::create_dir(&output).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            input.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let reference = stages::read(&output.join("plate_1.gcode"), stages::Stage::Slice).unwrap();

    let actual_bytes = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            INPUT,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();

    let blocks = |bytes: &[u8]| -> Vec<Vec<String>> {
        let mut result: Vec<Vec<String>> = Vec::new();
        let mut current: Vec<String> = Vec::new();
        let mut inside = false;
        let mut flush = |current: &mut Vec<String>| {
            if !current.is_empty() {
                result.push(std::mem::take(current));
            }
        };
        for line in std::str::from_utf8(bytes).unwrap().lines() {
            if line == "======== X2D filament_change gcode ==========" && inside {
                flush(&mut current);
            }
            inside |= line == "======== X2D filament_change gcode ==========";
            if !inside {
                continue;
            }
            current.push(line.to_owned());
            if line == ";not set fan changing filament" {
                flush(&mut current);
                inside = false;
            }
        }
        if inside {
            flush(&mut current);
        }
        result
    };
    let reference_blocks = blocks(&reference);
    let actual_blocks = blocks(&actual_bytes);
    assert_eq!(reference_blocks.len(), actual_blocks.len());
    for (index, (expected, actual)) in reference_blocks
        .iter()
        .zip(actual_blocks.iter())
        .enumerate()
    {
        assert_eq!(expected, actual, "change_filament block {index} differs");
    }
}
