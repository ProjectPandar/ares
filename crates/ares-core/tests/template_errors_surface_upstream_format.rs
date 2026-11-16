//! O7: custom G-code template parse errors surface with the upstream
//! message format (GCode.cpp:3636-3649 aggregation,
//! PlaceholderParser.cpp:1726-1770 line/caret, :988-1000 vs :918-932 kind).

#[path = "../../../tests/support/slice_project_plate.rs"]
mod direct_project;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use direct_project::slice_project_plate;

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/../../tests/parity/validation/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    ))
    .unwrap()
}

/// Matrix O7 (4 cases share one root cause): Ginger G1 machine_start_gcode
/// references `{extruder_rotation_volume[0]}` which 2.4.2 never defines.
/// Upstream: new-syntax symbol resolution fails
/// (PlaceholderParser.cpp:988-1000 resolve_variable) -> "Not a variable
/// name"; the caret points at the identifier start (column 102).
#[tokio::test]
async fn ginger_g1_start_gcode_not_a_variable_name() {
    let error = slice_project_plate(
        fixture("template-errors-ginger-g1-template.3mf"),
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        // clang-format off
        r#"Failed to generate G-code for invalid custom G-code.

machine_start_gcode Parsing error at line 1: Not a variable name
START_PRINT BED_TEMPERATURE=[bed_temperature_initial_layer] KAMP_LEVELING=1 EXTRUDER_ROTATION_VOLUME={extruder_rotation_volume[0]} MIXING_STEPPER_ROTATION_VOLUME={mixing_stepper_rotation_volume[0]} PURGE_LAYER_HEIGHT=2 PURGE_PARKING_SPEED=10000 PURGE_LENGHT=500 PURGE_SPEED=500 PURGE_MATERIAL_QUANTITY=10000 EXTRUDER_TEMPERATURE=[nozzle_temperature] EXTRUDER_TEMPERATURE_INITIAL_LAYER=[nozzle_temperature_initial_layer] PRESSURE_ADVANCE=0.2 PRESSURE_ADVANCE_SMOOTH_TIME=0.5 ZONE_1_TEMPERATURE={multi_zone_1_initial_layer[0]} ZONE_2_TEMPERATURE={multi_zone_2_initial_layer[0]} ZONE_3_TEMPERATURE={multi_zone_3_initial_layer[0]}
                                                                                                      ^

Please check the custom G-code or use the default custom G-code."# // clang-format on
    );
}

/// Matrix O7 fifth case: FLSun QQ-S Pro machine_start_gcode line 51
/// references `[output_filename_format]` -- legacy expansion
/// (PlaceholderParser.cpp:918-932 legacy_variable_expansion) ->
/// "Variable does not exist"; caret at column 12.
#[tokio::test]
async fn flsun_qqs_output_filename_variable_does_not_exist() {
    let error = slice_project_plate(
        fixture("template-errors-flsun-qqs-template.3mf"),
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        // clang-format off
        r#"Failed to generate G-code for invalid custom G-code.

machine_start_gcode Parsing error at line 51: Variable does not exist
M117 Print [output_filename_format]; Display: Printing started...
            ^

Please check the custom G-code or use the default custom G-code."# // clang-format on
    );
}
