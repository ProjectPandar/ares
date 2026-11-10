//! Public processed-slice seam: GCodeProcessor.cpp:3237-3301 must preserve
//! active-filament color tokens as ordered CustomGCode::Item records.
//! Failure inventory before implementation: the current recorder discards
//! color text; events could use zero-based tool ids, duplicate on M600, or
//! survive a later no-change slice. Fixture tags are supplied only through
//! the existing user template override, never through emitter changes.

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};

use crate::runner::application::slice_project_plate_processed;

#[path = "adjacent_color/native.rs"]
mod native;
use native::check_native_processor;

#[test]
fn color_oracle_availability_follows_required_optional_policy() {
    let directory = tempfile::Builder::new()
        .prefix("color-policy-")
        .tempdir()
        .unwrap()
        .keep();
    for key in ["ARES_PROCGCODE_BIN"] {
        for required in [true, false] {
            // Declared files only: the missing peer stops before either can launch.
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "runner::adjacent_color::no_change_slice_is_a_native_processor_control",
                    "--nocapture",
                ])
                .env("ARES_REQUIRE_ORCA", if required { "1" } else { "0" })
                .env("ARES_PROCGCODE_BIN", std::env::current_exe().unwrap())
                .env_remove(key)
                .output()
                .unwrap();
            let diagnostic = format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            std::fs::write(
                directory.join(format!("{key}-required-{required}.log")),
                &diagnostic,
            )
            .unwrap();
            assert!(
                diagnostic.contains("running 1 test") && diagnostic.contains(key),
                "{diagnostic}; artifacts {directory:?}"
            );
            assert_eq!(
                output.status.success(),
                !required,
                "{diagnostic}; artifacts {directory:?}"
            );
            assert_eq!(diagnostic.contains("SKIPPED:"), !required, "{diagnostic}");
            if !required {
                assert!(diagnostic.contains("not executed"), "{diagnostic}");
            }
        }
    }
}

const PROJECT: &[u8] = include_bytes!("../../../../../tests/parity/orca_cli_ender3.3mf");

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
    let unrecorded = crate::runner::application::slice_project_plate(
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
}

#[tokio::test]
async fn tag_defaults_invalid_colors_and_inactive_tools_match_native() {
    for (name, tag, two_tools) in [
        ("default-color", ";COLOR_CHANGE,T0", false),
        ("default-tool-color", ";COLOR_CHANGE", false),
        ("invalid-color", ";COLOR_CHANGE,T0,#ZZ0000", false),
        ("out-of-range-tool", ";COLOR_CHANGE,T1,#FF0000", false),
        ("inactive-tool", ";COLOR_CHANGE,T1,#FF0000", true),
        ("inline-tag", "G4 P0 ;COLOR_CHANGE,T0,#FF0000", false),
    ] {
        let mut values = serde_json::json!({
            "layer_change_gcode": format!("{{if layer_num == 2}}{tag}\nM600\n{{endif}}"),
        });
        if two_tools {
            values["filament_diameter"] = serde_json::json!(["1.75", "1.75"]);
            values["filament_colour"] = serde_json::json!(["#00AE42", "#0000FF"]);
            values["nozzle_diameter"] = serde_json::json!(["0.4", "0.4"]);
            // Per-head 2×2 matrix (two heads): keep the slicing fixture valid.
            values["flush_volumes_matrix"] =
                serde_json::json!(["0", "140", "140", "0", "0", "140", "140", "0"]);
        }
        let overrides = ConfigOverrides::from_json(&serde_json::to_vec(&values).unwrap()).unwrap();
        let output = slice_project_plate_processed(
            PROJECT.to_vec(),
            GenerationMetadata::default(),
            Some(1),
            &overrides,
            SlicerContext::new(0),
        )
        .await
        .unwrap();
        check_native_processor(&output, name);
    }
}

#[tokio::test]
async fn no_change_slice_is_a_native_processor_control() {
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
    // OptionsZCorrector::update sets print_z from the preceding move's f32
    // position at the next extrusion (GCodeProcessor.hpp:693-705). The
    // approved stable correction remaps the pending marker across actual-speed
    // insertions, so the marker and custom item follow that same extrusion Z,
    // not the tag Z or the first intervening travel.
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
        output.result.moves[marker].position[2].to_bits(),
        extrusion.position[2].to_bits()
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
async fn committed_bbl_cardinality_fixtures_match_native_inside_start_templates() {
    // These are the committed standalone native inputs, now embedded through
    // the public slice seam. The following real model adds a final positive
    // interval, so compare the emitted file itself, not standalone cardinalities.
    for (name, events) in [
        ("normal", 2),
        ("no-change", 0),
        ("bare-m600", 0),
        ("consecutive", 2),
        ("zero-before", 1),
        ("zero-between", 2),
        ("zero-after", 2),
        ("zero-only", 1),
        ("support-only", 1),
    ] {
        let template = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../tests/parity/color-change/{name}.gcode")),
        )
        .unwrap();
        let overrides = ConfigOverrides::from_json(
            &serde_json::to_vec(&serde_json::json!({
                "printer_model": "Bambu Lab A1", "machine_start_gcode": template,
            }))
            .unwrap(),
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
        assert_eq!(
            output.result.custom_gcode_per_print_z.len(),
            events,
            "{name}"
        );
        check_native_processor(&output, &format!("bbl-{name}"));
    }
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
    let unrecorded = crate::runner::application::slice_project_plate(
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
