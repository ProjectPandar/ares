//! Public Print boundary controls; Print.cpp:1543-1556,1654-1671.
use ares_core::{ProjectSettings, SliceError};
use serde_json::{Value, json};

fn check(extra: Value, expected: Option<&str>) {
    let mut fields = json!({"nozzle_diameter":["0.4"], "layer_height":"0.2", "line_width":"0", "use_relative_e_distances":"0", "gcode_flavor":"marlin"});
    fields
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let config: ProjectSettings = serde_json::from_value(fields.clone()).unwrap();
    let actual = ares_core::validate_print(&config, false);
    let directory = super::evidence_directory("print-upper-flat-");
    std::fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "settings":fields, "expected":expected, "actual":actual.as_ref().map_err(ToString::to_string)
    })).unwrap()).unwrap();
    match expected {
        Some(expected) => assert!(
            matches!(actual, Err(SliceError::Validate { message: ref text, .. }) if text == expected),
            "{actual:?}; {directory:?}"
        ),
        None => assert!(actual.is_ok(), "{actual:?}; {directory:?}"),
    }
}

#[test]
fn print_upper_auto_equal_and_percentage_boundaries() {
    for width in ["0", "0%", "1.999999", "2", "500%"] {
        check(json!({"line_width":width}), None);
    }
    for width in ["2.000001", "500.0001%"] {
        check(json!({"line_width":width}), Some("Line width too large"));
    }
    check(json!({"line_width":"0.2"}), Some("Line width too small"));
}

#[test]
fn print_upper_each_region_key_uses_same_source_predicate() {
    for key in [
        "inner_wall_line_width",
        "outer_wall_line_width",
        "sparse_infill_line_width",
        "internal_solid_infill_line_width",
        "top_surface_line_width",
        "skin_infill_line_width",
        "skeleton_infill_line_width",
    ] {
        check(json!({key:"2"}), None);
        check(json!({key:"2.000001"}), Some("Line width too large"));
    }
}

#[test]
fn print_upper_support_is_conditional() {
    check(json!({"support_line_width":"3"}), None);
    for activation in ["enable_support", "enforce_support_layers", "raft_layers"] {
        check(
            json!({"support_line_width":"3",activation:"1"}),
            Some("Line width too large"),
        );
    }
}

#[test]
fn print_upper_unused_larger_nozzle_cannot_relax_bound() {
    check(
        json!({"nozzle_diameter":["0.4","1.0"],"filament_map":["1"],"line_width":"2.1"}),
        Some("Line width too large"),
    );
    check(
        json!({"nozzle_diameter":["0.4","1.0"],"filament_map":["2"],"line_width":"2.1"}),
        None,
    );
    check(
        json!({"nozzle_diameter":["1.0","0.4"],"filament_map":["2"],"line_width":"2.1"}),
        Some("Line width too large"),
    );
}

#[test]
fn print_upper_minimum_and_maximum_use_the_same_used_set() {
    let shared = json!({"nozzle_diameter":["0.4","0.8"],"filament_diameter":["1.75","1.75"],"filament_map":["1","2"],"enable_support":"1","support_filament":"2","support_interface_filament":"2"});
    for (width, expected) in [
        ("2.5", None),
        ("500%", None),
        ("500.0001%", Some("Line width too large")),
        ("50%", Some("Line width too small")),
    ] {
        let mut config = shared.clone();
        config
            .as_object_mut()
            .unwrap()
            .insert("line_width".to_owned(), json!(width));
        check(config, expected);
    }
}

#[test]
fn print_upper_checks_keys_in_source_first_error_order() {
    check(
        json!({"line_width":"3","inner_wall_line_width":"0.2"}),
        Some("Line width too large"),
    );
    check(
        json!({"line_width":"0.2","inner_wall_line_width":"3"}),
        Some("Line width too small"),
    );
    check(
        json!({"inner_wall_line_width":"3","outer_wall_line_width":"0.2"}),
        Some("Line width too large"),
    );
    check(
        json!({"inner_wall_line_width":"0.2","outer_wall_line_width":"3"}),
        Some("Line width too small"),
    );
    check(
        json!({"line_width":"3","use_relative_e_distances":"1","layer_change_gcode":"g92 e0"}),
        Some("Line width too large"),
    );
}
