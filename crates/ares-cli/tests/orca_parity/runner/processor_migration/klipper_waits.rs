//! Klipper `M109`/`M190` synchronize the time estimator
//! (`docs/architecture/klipper-temperature-wait-sync-spec.md`).
use super::{settings, slice};
use crate::runner::OrcaRunner;
use ares_core::gcode_processor::process_gcode;
use serde_json::json;

const PROJECT: &[u8] =
    include_bytes!("../../../../../../tests/parity/merged-jerk-tolerance/equal-control.3mf");

#[test]
fn klipper_temperature_waits_synchronize_processor_times() {
    let Some(oracle) = crate::native_environment::binary("ARES_PROCGCODE_BIN") else {
        return;
    };
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let reference = slice(&runner, PROJECT);
    let text = std::str::from_utf8(&reference).unwrap();
    let settings = settings(text);
    assert_eq!(
        settings.printer.gcode.gcode_flavor,
        ares_core::GCodeFlavor::Klipper
    );
    assert!(
        waits_between_moves(text),
        "Native must wait for bed and nozzle temperatures between two moves"
    );

    let input = runner.work.join("processor-input.gcode");
    std::fs::write(&input, &reference).unwrap();
    let csv = runner.work.join("native-moves.csv");
    let output = std::process::Command::new(oracle)
        .arg("--process-gcode")
        .arg(&input)
        .arg("--dump-moves")
        .arg(&csv)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "native processor: {:?}",
        output.status
    );
    let native = std::fs::read_to_string(csv).unwrap();
    let mut rows = native.lines();
    let header = rows.next().unwrap().split(',').collect::<Vec<_>>();
    let column = |name| header.iter().position(|column| *column == name).unwrap();
    let (id, kind, layer, time, internal) = (
        column("gcode_id"),
        column("type"),
        column("layer_id"),
        column("time0"),
        column("internal_only"),
    );
    let expected = rows
        .map(|line| line.split(',').collect::<Vec<_>>())
        .filter(|row| row[internal] == "0")
        .map(|row| {
            (
                row[id].parse::<u32>().unwrap(),
                row[kind].parse::<u8>().unwrap(),
                row[layer].parse::<u32>().unwrap(),
                row[time].parse::<f32>().unwrap().to_bits(),
            )
        })
        .collect::<Vec<_>>();
    let actual = process_gcode(text, &settings, false, false)
        .result
        .moves
        .iter()
        .filter(|item| !item.internal_only)
        .map(|item| {
            (
                item.gcode_id,
                item.move_type as u8,
                item.layer_id,
                item.time.to_bits(),
            )
        })
        .collect::<Vec<_>>();
    let differences = expected
        .iter()
        .zip(&actual)
        .filter(|(native, actual)| native != actual)
        .map(|(native, actual)| json!({"native": native, "ares": actual}))
        .collect::<Vec<_>>();
    std::fs::write(
        runner.work.join("processor-time-comparison.json"),
        serde_json::to_vec_pretty(&json!({"differences": differences})).unwrap(),
    )
    .unwrap();
    assert!(
        expected.len() == actual.len() && differences.is_empty(),
        "non-internal move times differ: Native/Ares records {}/{}, {} differing, first {:?}; {}",
        expected.len(),
        actual.len(),
        differences.len(),
        differences.first(),
        runner.work.display()
    );
}

/// A move, then `M190` and `M109`, then a move, with nothing else that synchronizes between.
fn waits_between_moves(text: &str) -> bool {
    let mut moved = false;
    let mut waits = (false, false);
    for line in text.lines() {
        let command = line.split(';').next().unwrap().trim();
        let token = command.split_whitespace().next().unwrap_or_default();
        let has = |letter: char| {
            command
                .split_whitespace()
                .any(|word| word.starts_with(letter))
        };
        match token {
            "G0" | "G1" => {
                if moved && waits.0 && waits.1 {
                    return true;
                }
                moved = true;
                waits = (false, false);
            }
            "M190" => waits.0 = true,
            "M109" => waits.1 = true,
            "M400" => moved = false,
            "G4" if has('S') || has('P') => moved = false,
            "G92" if !has('E') => moved = false,
            _ => {}
        }
    }
    false
}
