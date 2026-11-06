//! Public flat Print validation controls for Print.cpp:1543-1556,1654-1671.
use ares_core::{ProjectSettings, SliceError};
use serde_json::{Value, json};

const ERROR: &str = "Line width too small";

fn settings(extra: Value) -> ProjectSettings {
    let mut fields = json!({"nozzle_diameter":["0.4"], "layer_height":"0.2", "line_width":"0", "use_relative_e_distances":"0", "gcode_flavor":"marlin"});
    fields
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_value(fields).unwrap()
}

fn check(extra: Value, rejected: bool) {
    let config = settings(extra.clone());
    let actual = ares_core::validate_print(&config, false);
    let evidence = super::evidence_directory("o4-flat-");
    std::fs::write(evidence.join("result.json"), serde_json::to_vec_pretty(&json!({"settings":extra, "expected_rejection":rejected, "ares":actual.as_ref().map_err(ToString::to_string)})).unwrap()).unwrap();
    if rejected {
        assert!(
            matches!(actual, Err(SliceError::Validate { message: ref text, .. }) if text == ERROR),
            "expected exact O4 rejection: {actual:?}; {evidence:?}"
        );
    } else {
        assert!(
            actual.is_ok(),
            "expected legal width: {actual:?}; {evidence:?}"
        );
    }
}

#[test]
fn flat_auto_and_nozzle_relative_percent_boundaries() {
    for width in ["0", "0%", "0.200001", "50.0001%", "100%"] {
        check(json!({"line_width":width}), false);
    }
    for width in ["0.199999", "0.2", "49%", "50%"] {
        check(json!({"line_width":width}), true);
    }
}

#[test]
fn flat_each_region_width_rejects_equality() {
    for key in [
        "inner_wall_line_width",
        "outer_wall_line_width",
        "sparse_infill_line_width",
        "internal_solid_infill_line_width",
        "top_surface_line_width",
        "skin_infill_line_width",
        "skeleton_infill_line_width",
    ] {
        check(json!({key:"50%"}), true);
        check(json!({key:"0"}), false);
    }
}

#[test]
fn flat_support_width_is_conditional_on_support_or_raft() {
    check(json!({"support_line_width":"0.2"}), false);
    check(
        json!({"support_line_width":"0.2", "enable_support":"1"}),
        true,
    );
    check(
        json!({"support_line_width":"0.2", "enforce_support_layers":"1"}),
        true,
    );
    check(json!({"support_line_width":"0.2", "raft_layers":"1"}), true);
}

#[test]
fn flat_first_error_precedes_layer_reset_check() {
    check(
        json!({"line_width":"0.2", "use_relative_e_distances":"1", "layer_change_gcode":"g92 e0"}),
        true,
    );
}

#[test]
fn flat_nozzle_baseline_uses_mapped_used_extruder_only() {
    check(
        json!({"nozzle_diameter":["0.2", "0.6"], "filament_map":["2"], "line_width":"50%"}),
        true,
    );
    check(
        json!({"nozzle_diameter":["0.6", "0.2"], "filament_map":["2"], "line_width":"100%"}),
        true,
    );
}
