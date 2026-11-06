//! Public controls for Print.cpp:1675-1692 / PrintRegion.cpp:7-23.
use ares_core::{ProjectSettings, SliceError};
use serde_json::{Value, json};

const LOWER: &str = "Line width too small";
const UPPER: &str = "Bridge line width must not exceed nozzle diameter";

fn check(extra: Value, expected: Option<&str>) {
    let mut fields = json!({"nozzle_diameter":["0.4"],"layer_height":"0.2","initial_layer_print_height":"0.2","line_width":"0","bridge_line_width":"100%","use_relative_e_distances":"0","gcode_flavor":"marlin"});
    fields
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let settings: ProjectSettings = serde_json::from_value(fields.clone()).unwrap();
    let actual = ares_core::validate_print(&settings, false);
    let directory = super::evidence_directory("print-bridge-flat-");
    std::fs::write(directory.join("result.json"),serde_json::to_vec_pretty(&json!({"settings":fields,"expected_error":expected,"actual":actual.as_ref().map_err(ToString::to_string)})).unwrap()).unwrap();
    if let Some(expected) = expected {
        assert!(
            matches!(actual,Err(SliceError::Validate { message: ref text, .. }) if text==expected),
            "{actual:?}; {directory:?}"
        );
    } else {
        assert!(actual.is_ok(), "{actual:?}; {directory:?}");
    }
}

#[test]
fn print_bridge_width_auto_and_boundaries() {
    for width in ["0", "0%", "0.200001", "50.0001%", "0.4", "100%"] {
        check(json!({"bridge_line_width":width}), None);
    }
    for width in ["0.1", "25%", "0.2", "50%"] {
        check(json!({"bridge_line_width":width}), Some(LOWER));
    }
    for width in ["0.400001", "100.0001%"] {
        check(json!({"bridge_line_width":width}), Some(UPPER));
    }
}

#[test]
fn print_bridge_width_thin_requires_both_object_flags() {
    for external in ["0", "1"] {
        for internal in ["0", "1"] {
            let both = external == "1" && internal == "1";
            check(
                json!({"bridge_line_width":"0.1","thick_bridges":external,"thick_internal_bridges":internal}),
                if both { None } else { Some(LOWER) },
            );
            check(
                json!({"bridge_line_width":"0.5","thick_bridges":external,"thick_internal_bridges":internal}),
                Some(UPPER),
            );
        }
    }
}

#[test]
fn print_bridge_width_percentage_is_role_local_direct_nozzle_not_filament_map() {
    check(
        json!({"nozzle_diameter":["0.4","0.8"],"filament_map":["2"],"layer_height":"0.3","bridge_line_width":"50%"}),
        Some(LOWER),
    );
}

#[test]
fn print_bridge_width_each_of_four_roles_participates() {
    let shared = json!({"nozzle_diameter":["0.4","0.8"],"filament_diameter":["1.75","1.75"],"filament_map":["2","2"],"layer_height":"0.3","bridge_line_width":"50%","inner_wall_filament_id":"2","sparse_infill_filament_id":"2","internal_solid_filament_id":"2","top_surface_filament_id":"2"});
    check(shared.clone(), None);
    for key in [
        "inner_wall_filament_id",
        "sparse_infill_filament_id",
        "internal_solid_filament_id",
        "top_surface_filament_id",
    ] {
        let mut fields = shared.clone();
        fields[key] = json!("1");
        check(fields, Some(LOWER));
    }
}

#[test]
fn print_bridge_width_outer_and_bottom_roles_are_excluded() {
    check(
        json!({"nozzle_diameter":["0.4","0.8"],"filament_diameter":["1.75","1.75"],"filament_map":["2","2"],"layer_height":"0.3","bridge_line_width":"50%","inner_wall_filament_id":"2","sparse_infill_filament_id":"2","internal_solid_filament_id":"2","top_surface_filament_id":"2","outer_wall_filament_id":"1","bottom_surface_filament_id":"1"}),
        None,
    );
}

#[test]
fn print_bridge_width_zero_and_out_of_range_selectors_use_front() {
    let shared = json!({"nozzle_diameter":["0.4","0.8"],"filament_diameter":["1.75","1.75","1.75"],"filament_map":["2","2","2"],"layer_height":"0.3","bridge_line_width":"50%","inner_wall_filament_id":"2","sparse_infill_filament_id":"2","internal_solid_filament_id":"2","top_surface_filament_id":"2"});
    for selector in ["0", "3"] {
        let mut fields = shared.clone();
        fields["inner_wall_filament_id"] = json!(selector);
        check(fields, Some(LOWER));
    }
}

#[test]
fn print_bridge_width_role_order_does_not_scan_all_uppers_first() {
    let shared = json!({"nozzle_diameter":["0.4","0.2"],"filament_diameter":["1.75","1.75"],"filament_map":["1","1"],"layer_height":"0.3","bridge_line_width":"0.25","inner_wall_filament_id":"1","sparse_infill_filament_id":"2"});
    check(shared.clone(), Some(LOWER));
    let mut fields = shared;
    fields["inner_wall_filament_id"] = json!("2");
    check(fields, Some(UPPER));
}

#[test]
fn print_bridge_width_ordinary_height_and_width_errors_precede_bridge() {
    check(
        json!({"layer_height":"0.5","bridge_line_width":"0.5"}),
        Some("Layer height cannot exceed nozzle diameter."),
    );
    check(
        json!({"line_width":"0.2","bridge_line_width":"0.5"}),
        Some(LOWER),
    );
    check(
        json!({"line_width":"3","bridge_line_width":"0.1"}),
        Some("Line width too large"),
    );
}

#[test]
fn print_bridge_width_native_auto_above_and_both_thick_reach_layer_reset_check() {
    for fields in [
        json!({"bridge_line_width":"0"}),
        json!({"bridge_line_width":"51%"}),
        json!({"bridge_line_width":"0.1","thick_bridges":"1","thick_internal_bridges":"1"}),
    ] {
        super::bridge_width::verify(
            super::RELATIVE_INPUT,
            super::RELATIVE_SHA256,
            fields,
            super::RELATIVE_ERROR,
        );
    }
}

#[test]
fn print_bridge_width_scoped_region_wins_over_global_bridge_width() {
    let input =
        include_bytes!("../../../../../tests/parity/validation/print-bridge-region-upper.3mf");
    for fields in [
        json!({"bridge_line_width":"0.1"}),
        json!({"thick_bridges":"1","thick_internal_bridges":"1"}),
    ] {
        super::bridge_width::verify(
            input,
            "e385145d47ec818770a1b7bb142c6fd6ac5e9f577d89830269f8fcfc79275909",
            fields,
            UPPER,
        );
    }
}

#[test]
fn print_bridge_width_stl_adapter_retains_width_and_object_flags() {
    let input = include_bytes!("../../../../../tests/parity/cube10.stl");
    for (index, fields, expected) in [
        (
            0,
            json!({"layer_height":0.2,"bridge_line_width":0.1}),
            Some(LOWER),
        ),
        (
            1,
            json!({"layer_height":0.2,"bridge_line_width":0.1,"thick_bridges":true,"thick_internal_bridges":true}),
            Some(super::RELATIVE_ERROR),
        ),
        (
            2,
            json!({"layer_height":0.2,"bridge_line_width":0.5}),
            // The unified loaded-config validate (`validate_loaded_config`)
            // runs the upstream `bridge_line_width <= nozzle` bound
            // (PrintConfig.cpp:10504-10517) before the Print.cpp region
            // check, so the STL adapter now returns the key-prefixed map
            // text rather than the bare `UPPER` message.
            Some("bridge_line_width: Bridge line width must not exceed nozzle diameter: 0.500000"),
        ),
    ] {
        let options: ares_core::SliceOptions = serde_json::from_value(fields.clone()).unwrap();
        let actual = ares_core::validate_stl(input, &options);
        let directory = super::evidence_directory("print-bridge-stl-");
        std::fs::write(directory.join("result.json"),serde_json::to_vec_pretty(&json!({"case":index,"options":fields,"expected_error":expected,"actual":actual.as_ref().map_err(ToString::to_string)})).unwrap()).unwrap();
        if index == 2 {
            assert!(
                matches!(actual, Err(SliceError::InvalidValuesInProject(ref text)) if text == "bridge_line_width: Bridge line width must not exceed nozzle diameter: 0.500000"),
                "{actual:?}; {directory:?}"
            );
        } else if let Some(expected) = expected {
            assert!(
                matches!(actual,Err(SliceError::InvalidInput(ref text)) if text==expected),
                "{actual:?}; {directory:?}"
            );
        } else {
            assert!(actual.is_ok(), "{actual:?}; {directory:?}");
        }
    }
}
