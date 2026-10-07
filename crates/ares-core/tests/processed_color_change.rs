//! Public processed-slice seam: GCodeProcessor.cpp:3237-3301 must preserve
//! active-filament color tokens as ordered CustomGCode::Item records.
//! Failure inventory before implementation: the current recorder discards
//! color text; events could use zero-based tool ids, duplicate on M600, or
//! survive a later no-change slice. Fixture tags are supplied only through
//! the existing user template override, never through emitter changes.

use ares_core::{
    ConfigOverrides, GenerationMetadata, SlicerContext, slice_project_plate_processed,
};

const PROJECT: &[u8] = include_bytes!("../../../tests/parity/orca_cli_ender3.3mf");

#[tokio::test]
async fn processed_layer_templates_preserve_ordered_color_change_items() {
    let overrides = ConfigOverrides::from_json(
        br##"{"layer_change_gcode":"{if layer_num == 2};COLOR_CHANGE,T0,#FF0000\nM600\n{endif}{if layer_num == 4};COLOR_CHANGE,T0,#0000FF\nM600\n{endif}"} "##,
    )
    .unwrap();
    let output = slice_project_plate_processed(
        PROJECT.to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &overrides,
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    let gcode = std::str::from_utf8(&output.gcode).unwrap();
    assert_eq!(
        gcode
            .lines()
            .filter(|line| line.starts_with(";COLOR_CHANGE,T0,"))
            .count(),
        2
    );
    // CustomGCode::Item uses one-based extruder ids, unlike MoveVertex.
    let items = &output.result.custom_gcode_per_print_z;
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].extruder, 1);
    assert_eq!(items[1].extruder, 1);
    assert_eq!(items[0].color, "#FF0000");
    assert_eq!(items[1].color, "#0000FF");
    assert_eq!(
        output
            .result
            .print_statistics
            .volumes_per_color_change
            .len(),
        3
    );
    assert!(
        output
            .result
            .print_statistics
            .volumes_per_color_change
            .iter()
            .all(|volume| *volume > 0.0)
    );
    check_native_volumes(&output, "two-changes");

    let plain = slice_project_plate_processed(
        PROJECT.to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    assert!(plain.result.custom_gcode_per_print_z.is_empty());
    // UsedFilaments::process_caches flushes the trailing model cache at EOF,
    // even with no color-change event (GCodeProcessor.cpp:1641-1648).
    assert_eq!(
        plain.result.print_statistics.volumes_per_color_change.len(),
        1
    );
    check_native_volumes(&plain, "no-change");
}

fn check_native_volumes(output: &ares_core::ProcessedGCode, name: &str) {
    let Some(oracle) = std::env::var_os("ARES_PROCGCODE_BIN") else {
        eprintln!("native comparison requires ARES_PROCGCODE_BIN (procgcode.nix)");
        return;
    };
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/color-change-slice-oracle")
        .join(name);
    std::fs::create_dir_all(&directory).unwrap();
    let gcode = directory.join("input.gcode");
    let native_json = directory.join("native.json");
    std::fs::write(&gcode, &output.gcode).unwrap();
    let run = std::process::Command::new(oracle)
        .arg("--process-gcode")
        .arg(&gcode)
        .arg("--dump-color-change")
        .arg(&native_json)
        .arg("--dump-moves")
        .arg(directory.join("native.csv"))
        .output()
        .unwrap();
    std::fs::write(
        directory.join("native.log"),
        [&run.stdout[..], &run.stderr[..]].concat(),
    )
    .unwrap();
    assert!(run.status.success(), "native processor: {:?}", run.status);
    let native: serde_json::Value =
        serde_json::from_slice(&std::fs::read(native_json).unwrap()).unwrap();
    let expected: Vec<f64> =
        serde_json::from_value(native["volumes_per_color_change"].clone()).unwrap();
    std::fs::write(
        directory.join("rust-statistics.json"),
        serde_json::to_vec_pretty(&output.result.print_statistics).unwrap(),
    )
    .unwrap();
    // Both processors accumulate the same per-G1 float volume into a double
    // cache; compare raw volumes, not reconstructed metric legend strings.
    assert_eq!(
        output.result.print_statistics.volumes_per_color_change,
        expected
    );
}
