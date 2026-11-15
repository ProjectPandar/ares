//! `GCodeProcessor::store_move_vertex` ORCA preview attributes
//! (`GCodeProcessor.cpp:5630-5706`): every recorded vertex carries the
//! Normal-mode move acceleration, the axis-selected jerk and
//! `m_pressure_advance` (M900/M572/`SET_PRESSURE_ADVANCE`, `:5031-5060`).
//!
//! Failure inventory before implementation: fields left at zero; print
//! acceleration used for travel/retract; per-block planner acceleration
//! instead of the setter-clamped machine value; JD jerk computed with the
//! print instead of the move acceleration, in f64, or without the axis clamp;
//! E-only/Z-only/XY axis selection wrong; Seam vertices using the seam instead
//! of the move delta; `SET_PRESSURE_ADVANCE` honoured outside Klipper or the
//! regex no-match/`stof` failure paths swapped; M900 without `K` resetting;
//! negative values not clamped; actual-speed vertices not inheriting values.
//!
//! The same vertices also carry the Normal-mode `actual_feedrate`
//! (`GCodeProcessor.cpp:480-572,5962-5997`): the entry speed on a non-motion
//! predecessor, the cruise speed on inserted vertices, cruise or exit speed on
//! the block's move, and that value again on a directly following Seam.
//! Failure inventory: field left at zero; exit used for cruise-only or
//! no-deceleration blocks; predecessor entry written for motion predecessors
//! or same-type ones; seam not synchronized; f64 instead of f32 profile.
//!
//! All expected bits come from runtime initialized-Native dynamics dumps.

use crate::runner::OrcaRunner;
use ares_core::{
    ConfigOverrides, GenerationMetadata, ProcessedGCode, SlicerContext,
    slice_project_plate_processed,
};

use crate::native_environment;

#[path = "../../../../ares-core/tests/support/native_policy_checks.rs"]
mod native_policy_checks;

#[test]
fn required_native_dynamics_cannot_fall_back_to_frozen_encodings() {
    native_policy_checks::require_missing_dependencies_fail(
        "runner::adjacent_motion::marlin2_classic_jerk",
        &["ARES_PROCGCODE_DYNAMICS_BIN"],
    );
}

#[test]
fn optional_native_dynamics_does_not_read_frozen_encodings() {
    let directory = tempfile::Builder::new()
        .prefix("motion-policy-")
        .tempdir()
        .unwrap()
        .keep();
    let run = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "runner::adjacent_motion::marlin2_classic_jerk",
            "--nocapture",
        ])
        .env("ARES_REQUIRE_ORCA", "0")
        .env_remove("ARES_PROCGCODE_DYNAMICS_BIN")
        .output()
        .unwrap();
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    std::fs::write(directory.join("optional.log"), &diagnostic).unwrap();
    assert!(
        diagnostic.contains("running 1 test") && diagnostic.contains("ARES_PROCGCODE_DYNAMICS_BIN"),
        "{diagnostic}"
    );
    assert!(
        run.status.success()
            && diagnostic.contains("SKIPPED:")
            && diagnostic.contains("not executed"),
        "optional Native must skip visibly without reading frozen RLE: {diagnostic}; artifacts {directory:?}"
    );
}

const PROJECT: &[u8] = include_bytes!("../../../../../tests/parity/orca_cli_ender3.3mf");

async fn processed(overrides: &str) -> ProcessedGCode {
    slice_project_plate_processed(
        PROJECT.to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &ConfigOverrides::from_json(overrides.as_bytes()).unwrap(),
        SlicerContext::new(0),
    )
    .await
    .unwrap()
}

/// `count,bits…` runs of identical f32 bit tuples.
fn encode<const N: usize>(values: impl Iterator<Item = [u32; N]>) -> String {
    let mut runs: Vec<(usize, [u32; N])> = Vec::new();
    for value in values {
        match runs.last_mut() {
            Some((count, last)) if *last == value => *count += 1,
            _ => runs.push((1, value)),
        }
    }
    runs.iter()
        .map(|(count, bits)| {
            let bits: String = bits.iter().map(|b| format!(",{b:08x}")).collect();
            format!("{count}{bits}\n")
        })
        .collect()
}

fn check(runner: &OrcaRunner, name: &str, output: &ProcessedGCode) {
    let directory = &runner.work;
    let actual = encode(output.result.moves.iter().map(|vertex| {
        [
            vertex.pressure_advance.to_bits(),
            vertex.acceleration.to_bits(),
            vertex.jerk.to_bits(),
        ]
    }));
    std::fs::write(directory.join("ares.rle"), &actual).unwrap();
    let actual_speed = encode(
        output
            .result
            .moves
            .iter()
            .map(|vertex| [vertex.actual_feedrate.to_bits()]),
    );
    std::fs::write(directory.join("ares.actual-speed.rle"), &actual_speed).unwrap();
    let time = encode(
        output
            .result
            .moves
            .iter()
            .map(|vertex| [vertex.time.to_bits()]),
    );
    std::fs::write(directory.join("ares.time.rle"), &time).unwrap();
    let capture = check_native(runner, output);
    let expected =
        |suffix: &str| std::fs::read_to_string(capture.join(format!("native{suffix}"))).unwrap();
    assert_eq!(
        actual,
        expected(".rle"),
        "{name}: per-vertex PA/acceleration/jerk bits"
    );
    // The planner feeds actual_feedrate, so its Normal move times
    // (`TimeMachine::calculate_time`) are asserted first.
    assert_eq!(
        time,
        expected(".time.rle"),
        "{name}: per-vertex Normal-mode move time bits"
    );
    assert_eq!(
        actual_speed,
        expected(".actual-speed.rle"),
        "{name}: per-vertex actual_feedrate bits"
    );
}

fn check_native(runner: &OrcaRunner, output: &ProcessedGCode) -> std::path::PathBuf {
    let directory = &runner.work;
    let text = std::str::from_utf8(&output.gcode).unwrap();
    // Producer-only rename so process_file loads the embedded config
    // (`GCodeProcessor.cpp:2620-2633`); line ids are unchanged.
    let renamed = text.replacen(
        "\n; generated by Ares 2.5.0 on 1980-01-01 at 00:00:00\n",
        "\n; generated by OrcaSlicer 2.5.0\n",
        1,
    );
    assert_ne!(renamed, text);
    let input = directory.join("oracle-input.gcode");
    std::fs::write(&input, renamed).unwrap();
    let capture = runner
        .processor_dynamics_reference(&input)
        .unwrap()
        .unwrap();
    let csv = capture.join("native-dynamics.csv");
    let rows: Vec<Vec<String>> = std::fs::read_to_string(&csv)
        .unwrap()
        .lines()
        .skip(1)
        .map(|line| line.split(',').map(str::to_owned).collect())
        .collect();
    let identities: Vec<_> = output
        .result
        .moves
        .iter()
        .map(|v| {
            [
                v.gcode_id.to_string(),
                (v.move_type as i32).to_string(),
                u8::from(v.internal_only).to_string(),
            ]
        })
        .collect();
    let native_identities: Vec<_> = rows
        .iter()
        .map(|row| [row[0].clone(), row[1].clone(), row[2].clone()])
        .collect();
    assert_eq!(identities, native_identities, "vertex identities");
    let bits = |row: &[String], column: usize| row[column].parse::<f32>().unwrap().to_bits();
    let native = encode(
        rows.iter()
            .map(|row| std::array::from_fn::<_, 3, _>(|column| bits(row, 3 + column))),
    );
    std::fs::write(directory.join("native.rle"), native).unwrap();
    let native = encode(rows.iter().map(|row| [bits(row, 6)]));
    std::fs::write(directory.join("native.actual-speed.rle"), native).unwrap();
    let moves = std::fs::read_to_string(capture.join("native.csv")).unwrap();
    let moves: Vec<Vec<String>> = moves
        .lines()
        .skip(1)
        .map(|line| line.split(',').map(str::to_owned).collect())
        .collect();
    assert_eq!(moves.len(), rows.len(), "native moves dump length");
    let native = encode(moves.iter().map(|row| [bits(row, 16)]));
    std::fs::write(directory.join("native.time.rle"), native).unwrap();
    directory.clone()
}

macro_rules! case {
    ($name:ident, $overrides:expr) => {
        #[tokio::test]
        async fn $name() {
            if native_environment::binary("ARES_PROCGCODE_DYNAMICS_BIN").is_none() {
                return;
            }
            let Some(runner) = OrcaRunner::from_env().unwrap() else {
                return;
            };
            check(&runner, stringify!($name), &processed($overrides).await);
        }
    };
}

// Marlin 2 with junction deviation: JD jerk from each move's own acceleration
// (travel/retract/print), axis-max clamp (X limited to 800), M204 above and
// below the maxima, M900 with/without K and negative K, a Klipper-only
// command that Marlin 2 must ignore.
case!(
    marlin2_junction_deviation,
    r##"{"gcode_flavor":"marlin2",
    "machine_max_junction_deviation":["0.013","0.013"],
    "machine_max_acceleration_x":["800","800"],
    "machine_max_acceleration_extruding":["1500","1500"],
    "machine_max_acceleration_retracting":["1000","1000"],
    "machine_max_acceleration_travel":["2500","2500"],
    "layer_change_gcode":"{if layer_num == 1}M900 K0.04\nM204 P900 R700 T3000\n{endif}{if layer_num == 2}M900\nM204 P5000 R5000 T400\nSET_PRESSURE_ADVANCE ADVANCE=0.9\n{endif}{if layer_num == 3}M900 K-1\nM204 S1200 T600\n{endif}{if layer_num == 4}M900 K0.0312\nM204 S0\n{endif}"}"##
);

// Marlin legacy: classic M205 jerks per axis, no separate travel
// acceleration, M572 pressure advance in every flavor.
case!(
    marlin_classic_jerk,
    r##"{"gcode_flavor":"marlin",
    "layer_change_gcode":"{if layer_num == 1}M205 X7.5 Y11 Z0.3 E4.5\nM572 S0.06\n{endif}{if layer_num == 2}M205 X12 Y9\nM204 P800 T2000\nM572 D0 S0.0725\n{endif}{if layer_num == 3}M572\n{endif}"}"##
);

// Marlin 2 without junction deviation keeps the classic per-axis jerks.
case!(
    marlin2_classic_jerk,
    r##"{"gcode_flavor":"marlin2",
    "machine_max_junction_deviation":["0","0"],
    "layer_change_gcode":"{if layer_num == 2}M205 X6 Y6.5 Z0.25 E3\nM900 K0.08\n{endif}"}"##
);

// Klipper: SET_VELOCITY_LIMIT accel/corner velocity, SET_PRESSURE_ADVANCE
// with EXTRUDER/SMOOTH_TIME words, lowercase (dispatched, regex no-match keeps
// value), missing ADVANCE (keeps), `ADVANCE=.` (stof failure -> 0).
case!(
    klipper_pressure_advance,
    r##"{"gcode_flavor":"klipper",
    "layer_change_gcode":"{if layer_num == 1}SET_PRESSURE_ADVANCE ADVANCE=0.045\nSET_VELOCITY_LIMIT ACCEL=3000 SQUARE_CORNER_VELOCITY=6\n{endif}{if layer_num == 2}SET_PRESSURE_ADVANCE EXTRUDER=extruder ADVANCE=0.03 SMOOTH_TIME=0.04\n{endif}{if layer_num == 3}set_pressure_advance ADVANCE=0.5\nSET_PRESSURE_ADVANCE SMOOTH_TIME=0.1\n{endif}{if layer_num == 4}SET_PRESSURE_ADVANCE ADVANCE=.\n{endif}{if layer_num == 5}SET_PRESSURE_ADVANCE ADVANCE = 1.25.5\nSET_VELOCITY_LIMIT ACCEL=700\n{endif}"}"##
);

// Option moves (Pause_Print, Custom_GCode) right before travel/extrusion:
// `actual_feedrate` takes the next block's entry speed on the non-motion
// predecessor (`GCodeProcessor.cpp:490-497`).
case!(
    marlin2_option_moves,
    r##"{"gcode_flavor":"marlin2",
    "layer_change_gcode":"{if layer_num == 2};PAUSE_PRINT\nM400\n{endif}{if layer_num == 3};CUSTOM_GCODE\nG4 P0\n{endif}"}"##
);
