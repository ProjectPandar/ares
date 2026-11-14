//! Failure modes prewritten before the Print.cpp:1751-1781 guard port.
use ares_core::{ConfigOverrides, GenerationMetadata, ProjectSettings, SliceError, SlicerContext};
use serde_json::{Value, json};

const DEFAULT_ERROR: &str = "curr_bed_type Default Plate has no first-layer temperature";
const BEDS: [(&str, &str); 6] = [
    ("Cool Plate", "cool_plate_temp"),
    ("Textured Cool Plate", "textured_cool_plate_temp"),
    ("Engineering Plate", "eng_plate_temp"),
    ("High Temp Plate", "hot_plate_temp"),
    ("Textured PEI Plate", "textured_plate_temp"),
    ("Supertack Plate", "supertack_plate_temp"),
];

fn check(extra: Value, bbl: bool, expected: Option<&str>) {
    let mut fields = json!({"nozzle_diameter":["0.4"],"filament_map":["1"],"layer_height":"0.2","initial_layer_print_height":"0.2","line_width":"0","bridge_line_width":"0","gcode_flavor":"marlin","use_relative_e_distances":"0","support_multi_bed_types":"1","curr_bed_type":"Cool Plate"});
    fields
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let settings: ProjectSettings = serde_json::from_value(fields.clone()).unwrap();
    let actual = ares_core::validate_print(&settings, bbl);
    let directory = super::evidence_directory("print-bed-flat-");
    std::fs::write(directory.join("result.json"),serde_json::to_vec_pretty(&json!({"settings":fields,"bbl":bbl,"expected_error":expected,"actual":actual.as_ref().map_err(ToString::to_string)})).unwrap()).unwrap();
    if let Some(expected) = expected {
        assert!(
            matches!(actual,Err(SliceError::Validate { message: ref text, .. }) if text==expected),
            "{actual:?}; {directory:?}"
        );
    } else {
        assert!(actual.is_ok(), "{actual:?}; {directory:?}");
    }
}

fn regions_second_with_default_volume() -> Value {
    json!({"nozzle_diameter":["0.4","0.4"],"filament_diameter":["1.75","1.75"],"filament_map":["1","1"],"outer_wall_filament_id":"2","inner_wall_filament_id":"2","sparse_infill_filament_id":"2","internal_solid_filament_id":"2","top_surface_filament_id":"2","bottom_surface_filament_id":"2"})
}

#[test]
fn print_bed_compat_default_plate_skipped_without_bbl_or_multi_bed() {
    check(
        json!({"curr_bed_type":"Default Plate","support_multi_bed_types":"0"}),
        false,
        None,
    );
    let options: ares_core::SliceOptions = serde_json::from_value(json!({
        "gcode_flavor":"klipper","support_multi_bed_types":false,"curr_bed_type":"Default Plate"
    }))
    .unwrap();
    assert!(
        ares_core::validate_stl(
            include_bytes!("../../../../../tests/parity/cube10.stl"),
            &options
        )
        .is_ok()
    );
}

#[test]
fn print_bed_compat_flat_bbl_or_multi_bed_activation() {
    for bbl in [false, true] {
        for multi in ["0", "1"] {
            let error = "Plate 1: Cool Plate does not support filament 1\n";
            check(
                json!({"support_multi_bed_types":multi,"cool_plate_temp":["0"]}),
                bbl,
                (bbl || multi == "1").then_some(error),
            );
        }
    }
}

#[test]
fn print_bed_compat_flat_six_vectors_selected_only_and_exact_zero() {
    for (bed, key) in BEDS {
        check(
            json!({"curr_bed_type":bed,key:["0"]}),
            false,
            Some(&format!("Plate 1: {bed} does not support filament 1\n")),
        );
        let mut fields = json!({"curr_bed_type":bed});
        for (_, other) in BEDS {
            fields[other] = json!([if other == key { "1" } else { "0" }]);
        }
        fields[format!("{key}_initial_layer")] = json!(["0"]);
        check(fields, false, None);
    }
}

#[test]
fn print_bed_compat_flat_unused_zero_is_not_scanned_and_no_physical_map() {
    let mut fields = regions_second_with_default_volume();
    fields["outer_wall_filament_id"] = json!("1");
    fields["inner_wall_filament_id"] = json!("1");
    fields["sparse_infill_filament_id"] = json!("1");
    fields["internal_solid_filament_id"] = json!("1");
    fields["top_surface_filament_id"] = json!("1");
    fields["bottom_surface_filament_id"] = json!("1");
    fields["filament_map"] = json!(["2", "1"]);
    fields["cool_plate_temp"] = json!(["45", "0"]);
    check(fields, false, None);
}

#[test]
fn print_bed_compat_flat_second_used_index_get_at_falls_back_to_front() {
    for (temperature, expected) in [
        ("45", None),
        (
            "0",
            Some("Plate 1: Cool Plate does not support filament 1\n"),
        ),
    ] {
        let mut fields = regions_second_with_default_volume();
        fields["cool_plate_temp"] = json!([temperature]);
        check(fields, false, expected);
    }
}

#[test]
fn print_bed_compat_flat_shared_usage_sorted_before_first_zero() {
    check(
        json!({"nozzle_diameter":["0.4","0.4"],"filament_diameter":["1.75","1.75"],"filament_map":["1","1"],"outer_wall_filament_id":"2","inner_wall_filament_id":"1","wall_loops":"2","cool_plate_temp":["0","0"]}),
        false,
        Some("Plate 1: Cool Plate does not support filament 1\n"),
    );
}

#[test]
fn print_bed_compat_flat_support_current_and_explicit_usage() {
    let base = json!({"nozzle_diameter":["0.4","0.4"],"filament_diameter":["1.75","1.75"],"filament_map":["1","1"],"cool_plate_temp":["45","0"],"support_filament":"2","support_interface_filament":"2"});
    check(base.clone(), false, None);
    let mut enabled = base;
    enabled["enable_support"] = json!("1");
    check(
        enabled,
        false,
        Some("Plate 1: Cool Plate does not support filament 2\n"),
    );
    let mut current = regions_second_with_default_volume();
    current.as_object_mut().unwrap().extend(json!({"enable_support":"1","support_filament":"0","support_interface_filament":"0","cool_plate_temp":["45","0"]}).as_object().unwrap().clone());
    check(
        current,
        false,
        Some("Plate 1: Cool Plate does not support filament 2\n"),
    );
}

#[test]
fn print_bed_compat_flat_wipe_tower_explicit_filament_enters_used_union() {
    check(
        json!({"nozzle_diameter":["0.4","0.4","0.4","0.4"],"filament_diameter":["1.75","1.75","1.75"],"filament_map":["1","1","1"],"wall_loops":"2","inner_wall_filament_id":"2","enable_prime_tower":"1","wipe_tower_filament":"3","cool_plate_temp":["45","45","0"]}),
        false,
        Some("Plate 1: Cool Plate does not support filament 3\n"),
    );
}

#[test]
fn print_bed_compat_flat_height_bridge_and_layer_reset_check_still_precede_bed() {
    check(
        json!({"cool_plate_temp":["0"],"layer_height":"0.5"}),
        false,
        Some("Layer height cannot exceed nozzle diameter."),
    );
    check(
        json!({"cool_plate_temp":["0"],"bridge_line_width":"0.1"}),
        false,
        Some("Line width too small"),
    );
    check(
        json!({"cool_plate_temp":["0"],"before_layer_change_gcode":"G92 E0\n"}),
        false,
        Some(super::ABSOLUTE_ERROR),
    );
}

#[test]
fn print_bed_compat_default_sentinel_exact_public_error_not_native_red() {
    let fields = json!({"curr_bed_type":"Default Plate","cool_plate_temp":["0"]});
    let overrides = ConfigOverrides::from_json(fields.to_string().as_bytes()).unwrap();
    let input = super::bed_compatibility::INPUT;
    let validation = ares_core::validate_project_plate(input, None, &overrides);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let sliced = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        None,
        &overrides,
        SlicerContext::new(0),
    ));
    let directory = super::evidence_directory("print-bed-default-");
    std::fs::write(directory.join("result.json"),serde_json::to_vec_pretty(&json!({"accepted_deviation":"native139UB; currentAresError/noGcode/noPanic; NOTparityRED","overrides":fields,"expected_error":DEFAULT_ERROR,"validation":validation.as_ref().map_err(ToString::to_string),"validated_slice":sliced.as_ref().map(Vec::len).map_err(ToString::to_string)})).unwrap()).unwrap();
    assert!(
        matches!(validation,Err(SliceError::Validate { message: ref text, .. }) if text==DEFAULT_ERROR),
        "{validation:?}; {directory:?}"
    );
    assert!(
        matches!(sliced,Err(SliceError::Validate { message: ref text, .. }) if text==DEFAULT_ERROR),
        "{sliced:?}; {directory:?}"
    );
    check(fields, false, Some(DEFAULT_ERROR));
}

#[test]
fn print_bed_compat_stl_adapter_keeps_bed_vector_and_activation() {
    let input = include_bytes!("../../../../../tests/parity/cube10.stl");
    for (fields, expected) in [
        (
            json!({"gcode_flavor":"klipper","support_multi_bed_types":true,"curr_bed_type":"Cool Plate","cool_plate_temp":[0]}),
            Some("Plate 1: Cool Plate does not support filament 1\n"),
        ),
        (
            json!({"gcode_flavor":"klipper","support_multi_bed_types":false,"curr_bed_type":"Cool Plate","cool_plate_temp":[0]}),
            None,
        ),
        (
            json!({"gcode_flavor":"klipper","support_multi_bed_types":true,"curr_bed_type":"Default Plate","cool_plate_temp":[0]}),
            Some(DEFAULT_ERROR),
        ),
    ] {
        let options: ares_core::SliceOptions = serde_json::from_value(fields.clone()).unwrap();
        let actual = ares_core::validate_stl(input, &options);
        let directory = super::evidence_directory("print-bed-stl-");
        std::fs::write(directory.join("result.json"),serde_json::to_vec_pretty(&json!({"options":fields,"expected_error":expected,"actual":actual.as_ref().map_err(ToString::to_string)})).unwrap()).unwrap();
        if let Some(expected) = expected {
            assert!(
                matches!(actual,Err(SliceError::InvalidInput(ref text)) if text==expected),
                "{actual:?}; {directory:?}"
            );
        } else {
            assert!(actual.is_ok(), "{actual:?}; {directory:?}");
        }
    }
}

#[test]
fn print_bed_compat_pure_processing_remains_unvalidated() {
    // Print.cpp:2779-2809 rejects this ordinary-zero configuration during validation;
    // bed_compatibility::print_bed_compat_original_cool_zero_rejects_before_processing covers it.
    let fields = json!({"cool_plate_temp":["0"]});
    let overrides = ConfigOverrides::from_json(fields.to_string().as_bytes()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let gcode = runtime
        .block_on(crate::runner::application::slice_project_plate(
            super::bed_compatibility::INPUT,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &overrides,
            SlicerContext::new(0),
        ))
        .unwrap();
    // GCode.cpp:4016-4028,5525-5530 selects the initial-layer temperature for startup.
    // The fixture retains cool_plate_temp_initial_layer=["45"] despite ordinary zero.
    let startup = std::str::from_utf8(&gcode)
        .unwrap()
        .split_once(";LAYER_CHANGE\n")
        .unwrap()
        .0;
    let startup_bed_commands: Vec<_> = startup
        .lines()
        .map(|line| line.split(';').next().unwrap().trim())
        .filter(|line| matches!(line.split_ascii_whitespace().next(), Some("M140" | "M190")))
        .collect();
    let directory = super::evidence_directory("print-bed-pure-");
    std::fs::write(directory.join("ares.gcode"), &gcode).unwrap();
    std::fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "overrides":fields,"startup_bed_commands":startup_bed_commands,"bytes":gcode.len(),
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        startup_bed_commands,
        ["M140 S45", "M190 S45"],
        "{directory:?}"
    );
}
