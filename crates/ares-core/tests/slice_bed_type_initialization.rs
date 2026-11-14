use ares_core::{ConfigOverrides, GenerationMetadata, SliceOptions, SlicerContext};
use serde_json::json;

#[tokio::test]
async fn byte_slice_rejects_a_default_current_bed() {
    check_byte_current("Default Plate").await;
}

#[tokio::test]
async fn byte_slice_rejects_an_empty_current_bed() {
    check_byte_current("").await;
}

async fn check_byte_current(current: &str) {
    let options: SliceOptions = serde_json::from_value(json!({
        "curr_bed_type": current,
        "default_bed_type": "Textured PEI Plate",
        "cool_plate_temp_initial_layer": [35],
        "cool_plate_temp": [31],
        "hot_plate_temp_initial_layer": [45],
        "hot_plate_temp": [41],
        "textured_plate_temp_initial_layer": [55],
        "textured_plate_temp": [51],
        "machine_start_gcode": "M140 S[bed_temperature_initial_layer_single]\nM190 S[bed_temperature_initial_layer_single]"
    }))
    .unwrap();
    let result =
        ares_core::slice(include_bytes!("../../../tests/parity/cube10.stl"), options).await;
    assert!(
        matches!(result, Err(ares_core::SliceError::InvalidInput(ref message)) if message == "curr_bed_type has an invalid value"),
        "{current:?}: {result:?}"
    );
}

// OrcaSlicer.cpp:7993–8001 inherits plate defaults, not global sentinels.
#[tokio::test]
async fn project_rejects_global_default_and_preserves_concrete_bed() {
    for (index, (current, hint, initial, ordinary)) in [
        ("Default Plate", "Textured PEI Plate", 61, 59),
        ("Default Plate", "", 55, 50),
        ("Default Plate", "invalid", 55, 50),
        ("High Temp Plate", "Textured PEI Plate", 55, 50),
    ]
    .into_iter()
    .enumerate()
    {
        let overrides = ConfigOverrides::from_json(
            &serde_json::to_vec(&json!({
                "curr_bed_type": current,
                "default_bed_type": hint,
                "printer_model": "BL-P001",
                "support_multi_bed_types": "1"
            }))
            .unwrap(),
        )
        .unwrap();
        let result = ares_core::slice_project_plate_validated(
            include_bytes!("../../../tests/parity/bed-type-selection/omitted.3mf"),
            GenerationMetadata::default(),
            Some(1),
            &overrides,
            SlicerContext::new(0),
        )
        .await;
        if current == "Default Plate" {
            assert!(
                matches!(result, Err(ares_core::SliceError::Validate { ref message, .. }) if message == "curr_bed_type Default Plate has no first-layer temperature"),
                "{hint:?}: {result:?}"
            );
        } else {
            check_commands(
                &result.unwrap(),
                initial,
                ordinary,
                &format!("project-{index}.gcode"),
            );
        }
    }
}

fn check_commands(gcode: &[u8], initial: i32, ordinary: i32, name: &str) {
    let root = std::env::var_os("ARES_BED_TYPE_ARTIFACT_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("ares-bed-initialization-{}", std::process::id()))
        });
    std::fs::create_dir_all(&root).unwrap();
    let output = root.join(name);
    std::fs::write(&output, gcode).unwrap();
    println!("G-code: {}", output.display());
    let commands: Vec<_> = std::str::from_utf8(gcode)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with("M140 ") || line.starts_with("M190 "))
        .map(|line| line.split(';').next().unwrap().trim())
        .collect();
    for command in [
        format!("M140 S{initial}"),
        format!("M190 S{initial}"),
        format!("M140 S{ordinary}"),
    ] {
        assert!(
            commands.contains(&command.as_str()),
            "{command}: {commands:?}"
        );
    }
}
