//! Public processor times for the real LH Stinger STL session, not planner internals.
//! Failure inventory and upstream boundary: docs/architecture/klipper-time-estimator.md.
use super::{OrcaRunner, cube_at, native, presets};
use ares_core::gcode_processor::process_gcode;
use serde_json::json;

#[test]
fn klipper_stl_session_processor_times_match_native() {
    if !crate::runner::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let Some(oracle) = crate::native_environment::binary("ARES_PROCGCODE_BIN") else {
        return;
    };
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    presets(&runner);
    let reference = native(
        &runner,
        "klipper-time-session",
        &cube_at([118.0, 138.0, 0.0]),
    );
    let input = runner.work.join("processor-input.gcode");
    std::fs::write(&input, &reference).unwrap();
    let text = std::str::from_utf8(&reference).unwrap();
    let settings = crate::runner::processor_migration::settings(text);
    assert_eq!(
        settings.printer.gcode.gcode_flavor,
        ares_core::GCodeFlavor::Klipper
    );
    let actual = process_gcode(text, &settings, false, false);
    let csv = runner.work.join("native-moves.csv");
    let output = std::process::Command::new(oracle)
        .arg("--process-gcode")
        .arg(&input)
        .arg("--dump-moves")
        .arg(&csv)
        .output()
        .unwrap();
    std::fs::write(runner.work.join("native-process.stdout"), &output.stdout).unwrap();
    std::fs::write(runner.work.join("native-process.stderr"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "native processor: {:?}",
        output.status
    );
    let native = std::fs::read_to_string(csv).unwrap();
    let mut rows = native.lines();
    assert_eq!(
        rows.next(),
        Some(
            "gcode_id,type,role,extruder_id,cp_color_id,x,y,z,delta_extruder,feedrate,width,height,mm3_per_mm,travel_dist,fan_speed,temperature,time0,layer_id,internal_only"
        )
    );
    let expected = rows
        .map(|line| line.split(',').collect::<Vec<_>>())
        .filter(|row| row[18] == "0")
        .map(|row| {
            (
                row[0].parse::<u32>().unwrap(),
                row[1].parse::<u8>().unwrap(),
                row[17].parse::<u32>().unwrap(),
                row[16].parse::<f32>().unwrap().to_bits(),
            )
        })
        .collect::<Vec<_>>();
    let actual = actual
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
        .enumerate()
        .filter(|(_, (native, actual))| native != actual)
        .map(|(index, (native, actual))| json!({"index": index, "native": native, "ares": actual}))
        .collect::<Vec<_>>();
    std::fs::write(
        runner.work.join("processor-time-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": crate::runner::cache_identity::raw_digest(&reference),
            "native_records": expected,
            "ares_records": actual,
            "differences": differences,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        expected.len() == actual.len() && differences.is_empty(),
        "complete public non-internal move times/IDs differ: Native/Ares records {}/{}, {} differing paired records, first {:?}; artifacts {}",
        expected.len(),
        actual.len(),
        differences.len(),
        differences.first(),
        runner.work.display()
    );
}
