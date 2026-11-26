use ares_core::{GenerationMetadata, SlicerContext};
use serde_json::json;
use std::io::{Cursor, Read, Write};

mod enum_declarations_support;

#[tokio::test]
async fn project_nozzle_manual_is_retained_but_does_not_replace_the_plate_default() {
    let input = enum_declarations_support::input("filament_map_mode", "Nozzle Manual");
    let input = with_plate_map_mode(input, "");
    let project = ares_core::load_project(&input).unwrap();
    assert_eq!(
        enum_declarations_support::loaded_value(&project, "filament_map_mode"),
        "Nozzle Manual"
    );
    assert!(
        !ares_core::project_config_substitutions(&input)
            .unwrap()
            .iter()
            .any(|entry| entry.key == "filament_map_mode")
    );
    assert_map_gcode("project-nozzle-manual", input, "Auto For Flush", 0).await;
}

#[tokio::test]
async fn plate_nozzle_manual_overrides_the_project_mode() {
    let input = enum_declarations_support::input("filament_map_mode", "Auto For Flush");
    let input = with_plate_map_mode(
        input,
        "<metadata key=\"filament_map_mode\" value=\"Nozzle Manual\"/>",
    );
    assert_map_gcode("plate-nozzle-manual", input, "Nozzle Manual", 1).await;
}

fn with_plate_map_mode(input: Vec<u8>, replacement: &str) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        if entry.name() == "Metadata/model_settings.config" {
            let mut xml = String::new();
            entry.read_to_string(&mut xml).unwrap();
            let old = "<metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>";
            assert_eq!(xml.matches(old).count(), 1);
            let xml = xml.replace(old, replacement);
            output
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            output.write_all(xml.as_bytes()).unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    output.finish().unwrap().into_inner()
}

async fn assert_map_gcode(name: &str, input: Vec<u8>, mode: &str, nozzle: i32) {
    let work = enum_declarations_support::artifacts(name);
    std::fs::write(work.join("input.3mf"), &input).unwrap();
    let bytes =
        ares_core::slice_project(&input, GenerationMetadata::default(), SlicerContext::new(1))
            .await
            .unwrap();
    std::fs::write(work.join("output.gcode"), &bytes).unwrap();
    let gcode = std::str::from_utf8(&bytes).unwrap();
    assert!(
        gcode
            .lines()
            .any(|line| line == format!("; filament_map_mode = {mode}"))
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == format!("; filament_nozzle_map = {nozzle}"))
    );
}

#[tokio::test]
async fn ultimaker_host() {
    let work = enum_declarations_support::artifacts("host_type-ultimaker");
    let input = enum_declarations_support::input("host_type", "ultimaker");
    std::fs::write(work.join("input.3mf"), &input).unwrap();
    let project = ares_core::load_project(&input).unwrap();
    assert_eq!(
        enum_declarations_support::loaded_value(&project, "host_type"),
        "ultimaker"
    );
    let substitutions = ares_core::project_config_substitutions(&input).unwrap();
    assert!(!substitutions.iter().any(|entry| entry.key == "host_type"));
    let result =
        ares_core::slice_project(&input, GenerationMetadata::default(), SlicerContext::new(1))
            .await;
    std::fs::write(
        work.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "loaded": "ultimaker", "substitutions": [],
            "error": result.as_ref().err().map(|error| format!("{error:?}")),
            "output_bytes": result.as_ref().ok().map(Vec::len),
        }))
        .unwrap(),
    )
    .unwrap();
    let bytes = result.unwrap();
    std::fs::write(work.join("output.gcode"), &bytes).unwrap();
    assert!(
        std::str::from_utf8(&bytes)
            .unwrap()
            .lines()
            .any(|line| line == "; host_type = ultimaker")
    );
}

#[tokio::test]
async fn leading_edge_brim_flat_bed() {
    let work = enum_declarations_support::artifacts("brim_type-leading_edge_only");
    let input = enum_declarations_support::input("brim_type", "leading_edge_only");
    std::fs::write(work.join("input.3mf"), &input).unwrap();
    let project = ares_core::load_project(&input).unwrap();
    assert_eq!(
        enum_declarations_support::loaded_value(&project, "brim_type"),
        "leading_edge_only"
    );
    let substitutions = ares_core::project_config_substitutions(&input).unwrap();
    assert!(!substitutions.iter().any(|entry| entry.key == "brim_type"));
    let result =
        ares_core::slice_project(&input, GenerationMetadata::default(), SlicerContext::new(1))
            .await;
    std::fs::write(
        work.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "loaded": "leading_edge_only", "substitutions": [],
            "error": result.as_ref().err().map(|error| format!("{error:?}")),
            "output_bytes": result.as_ref().ok().map(Vec::len),
        }))
        .unwrap(),
    )
    .unwrap();
    let bytes = result.unwrap();
    std::fs::write(work.join("output.gcode"), &bytes).unwrap();
    assert!(
        std::str::from_utf8(&bytes)
            .unwrap()
            .lines()
            .any(|line| line == "; brim_type = leading_edge_only")
    );
}
