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
    assert_eq!(output.result.custom_gcode_times.len(), 3);
    let unrecorded = ares_core::slice_project_plate(
        PROJECT,
        GenerationMetadata::default(),
        Some(1),
        &overrides,
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    assert_eq!(
        output.gcode, unrecorded,
        "custom synchronization must not depend on recording moves"
    );
    check_native_processor(&output, "two-changes");

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
    assert!(plain.result.custom_gcode_times.is_empty());
    check_native_processor(&plain, "no-change");
}

#[tokio::test]
async fn color_change_z_follows_the_next_extrusion_position() {
    // OptionsZCorrector::update copies the preceding move's f32 position
    // at the next extrusion (GCodeProcessor.hpp:693-705). The template hop
    // precedes the slicer's own layer travel: neither the tag Z nor the first
    // travel after it is necessarily the next extrusion's Z.
    let overrides = ConfigOverrides::from_json(
        br##"{"layer_change_gcode":"{if layer_num == 2};COLOR_CHANGE,T0,#FF0000\nM600\nG1 Z0.9 F600\nG1 X21 F600\n{endif}"} "##,
    ).unwrap();
    let output = slice_project_plate_processed(
        PROJECT.to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &overrides,
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    assert_eq!(output.result.custom_gcode_per_print_z.len(), 1);
    use ares_core::gcode_processor::EMoveType;
    let marker = output
        .result
        .moves
        .iter()
        .position(|vertex| vertex.move_type == EMoveType::ColorChange)
        .unwrap();
    let extrusion = output.result.moves[marker + 1..]
        .iter()
        .find(|vertex| vertex.move_type == EMoveType::Extrude)
        .unwrap();
    assert_eq!(
        output.result.custom_gcode_per_print_z[0].print_z,
        f64::from(extrusion.position[2])
    );
    assert_eq!(
        output.result.moves[marker].position[2],
        extrusion.position[2]
    );
    check_native_processor(&output, "corrected-z");
}

#[tokio::test]
async fn pause_and_custom_items_keep_color_volume_intervals_intact() {
    // GCodeProcessor.cpp:3303-3319: both tags record/arm Z correction,
    // but only PausePrint flushes custom time, and neither flushes color volume.
    let overrides = ConfigOverrides::from_json(
        br##"{"layer_change_gcode":"{if layer_num == 2};COLOR_CHANGE,T0,#FF0000\nM600\n{endif}{if layer_num == 4};PAUSE_PRINT\n{endif}{if layer_num == 6};CUSTOM_GCODE\n{endif}"} "##,
    ).unwrap();
    let output = slice_project_plate_processed(
        PROJECT.to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &overrides,
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    use ares_core::gcode_processor::CustomGCodeType;
    assert_eq!(
        output
            .result
            .custom_gcode_per_print_z
            .iter()
            .map(|item| item.kind)
            .collect::<Vec<_>>(),
        [
            CustomGCodeType::ColorChange,
            CustomGCodeType::PausePrint,
            CustomGCodeType::Custom
        ]
    );
    assert_eq!(
        output
            .result
            .print_statistics
            .volumes_per_color_change
            .len(),
        2
    );
    assert_eq!(
        output
            .result
            .custom_gcode_times
            .iter()
            .map(|item| item.kind)
            .collect::<Vec<_>>(),
        [
            CustomGCodeType::ColorChange,
            CustomGCodeType::PausePrint,
            CustomGCodeType::ColorChange
        ]
    );
    check_native_processor(&output, "pause-custom");
}

#[tokio::test]
async fn sync_delays_survive_color_and_pause_time_flushes() {
    // GCodeProcessor.cpp:5430-5436 synchronizes with T seconds. Exercise a
    // delay before ColorChange, between events, before PausePrint and at EOF;
    // all expectations come from processing the exact emitted commands natively.
    let overrides = ConfigOverrides::from_json(
        br##"{"layer_change_gcode":"{if layer_num == 2}SYNC T2.75\n;COLOR_CHANGE,T0,#FF0000\nM600\nSYNC T1.25\n{endif}{if layer_num == 4}SYNC T3.5\n;PAUSE_PRINT\n{endif}","machine_end_gcode":"SYNC T4.25\n"}"##,
    ).unwrap();
    let output = slice_project_plate_processed(
        PROJECT.to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &overrides,
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&output.gcode)
            .unwrap()
            .lines()
            .filter(|line| line.starts_with("SYNC T"))
            .count(),
        4
    );
    assert_eq!(output.result.custom_gcode_times.len(), 3);
    let unrecorded = ares_core::slice_project_plate(
        PROJECT,
        GenerationMetadata::default(),
        Some(1),
        &overrides,
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    assert_eq!(output.gcode, unrecorded);
    check_native_processor(&output, "sync-color-pause");
}

fn check_native_processor(output: &ares_core::ProcessedGCode, name: &str) {
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
    // Match the existing gcode_moves_oracle producer-only normalization:
    // process_file loads embedded printer config only for known producers
    // (GCodeProcessor.cpp:2620-2633). Keep all G-code commands/line ids intact.
    let text = std::str::from_utf8(&output.gcode).unwrap();
    let renamed = text.replacen(
        "\n; generated by Ares 2.4.2\n",
        "\n; generated by OrcaSlicer 2.4.2\n",
        1,
    );
    assert_ne!(renamed, text);
    let oracle_input = directory.join("oracle-input.gcode");
    std::fs::write(&oracle_input, renamed).unwrap();
    let run = std::process::Command::new(oracle)
        .arg("--process-gcode")
        .arg(&oracle_input)
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
    let expected_z: Vec<f64> = native["custom_gcode_per_print_z"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["print_z"].as_f64().unwrap())
        .collect();
    assert_eq!(
        output
            .result
            .custom_gcode_per_print_z
            .iter()
            .map(|item| item.print_z)
            .collect::<Vec<_>>(),
        expected_z
    );
    let expected_times = native["custom_gcode_times"].as_array().unwrap();
    assert_eq!(output.result.custom_gcode_times.len(), expected_times.len());
    for (actual, expected) in output.result.custom_gcode_times.iter().zip(expected_times) {
        assert_eq!(actual.kind as u64, expected["type"].as_u64().unwrap());
        // Native fields are float (GCodeProcessor.hpp:505). Compare those
        // bits, not extra f64 precision introduced by JSON decimal parsing.
        assert_eq!(
            actual.duration.to_bits(),
            (expected["duration"].as_f64().unwrap() as f32).to_bits()
        );
        assert_eq!(
            actual.remaining.to_bits(),
            (expected["remaining"].as_f64().unwrap() as f32).to_bits()
        );
    }
    std::fs::write(
        directory.join("rust-custom-times.json"),
        serde_json::to_vec_pretty(&output.result.custom_gcode_times).unwrap(),
    )
    .unwrap();
    // Both processors accumulate the same per-G1 float volume into a double
    // cache; compare raw volumes, not reconstructed metric legend strings.
    assert_eq!(
        output.result.print_statistics.volumes_per_color_change,
        expected
    );
}
