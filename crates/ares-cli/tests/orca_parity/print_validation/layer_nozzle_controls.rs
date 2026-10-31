//! Public Print boundary, including direct-index raft get_at/front semantics.
use ares_core::{ProjectSettings, SliceError};
use serde_json::{Value, json};

fn check(extra: Value, rejected: bool) {
    check_exact(
        extra,
        rejected.then_some("Layer height cannot exceed nozzle diameter."),
    );
}

fn check_exact(extra: Value, expected: Option<&str>) {
    let mut fields = json!({"nozzle_diameter":["0.4"],"layer_height":"0.2","initial_layer_print_height":"0.2","line_width":"0","gcode_flavor":"marlin","use_relative_e_distances":"0"});
    fields
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let settings: ProjectSettings = serde_json::from_value(fields.clone()).unwrap();
    let actual = ares_core::validate_print(&settings, false);
    let directory = super::evidence_directory("print-layer-flat-");
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
fn print_layer_nozzle_equal_is_legal_strictly_above_is_not() {
    for key in ["layer_height", "initial_layer_print_height"] {
        for height in ["0.399999", "0.4"] {
            if key == "layer_height" && height == "0.4" {
                check_exact(json!({key:height}), Some("Too small line width"));
            } else {
                check(json!({key:height}), false);
            }
        }
        check(json!({key:"0.400001"}), true);
    }
}

#[test]
fn print_layer_nozzle_unused_smaller_nozzle_is_not_minimum() {
    check_exact(
        json!({"nozzle_diameter":["0.1","0.4"],"filament_map":["2"]}),
        Some("Too small line width"),
    );
    check(
        json!({"nozzle_diameter":["0.4","0.1"],"filament_map":["2"]}),
        true,
    );
}

#[test]
fn print_layer_nozzle_support_used_nozzle_enters_minimum() {
    let shared = json!({"nozzle_diameter":["0.4","0.2"],"filament_diameter":["1.75","1.75"],"filament_map":["1","2"],"layer_height":"0.3","initial_layer_print_height":"0.1","support_filament":"2","support_interface_filament":"2"});
    check(shared.clone(), false);
    let mut enabled = shared;
    enabled
        .as_object_mut()
        .unwrap()
        .insert("enable_support".to_owned(), json!("1"));
    check(enabled, true);
}

#[test]
fn print_layer_nozzle_raft_one_selects_interface_other_counts_select_support() {
    let shared = json!({"nozzle_diameter":["0.2","0.4"],"filament_diameter":["1.75","1.75"],"filament_map":["1","2"],"layer_height":"0.1","initial_layer_print_height":"0.3","support_filament":"1","support_interface_filament":"2"});
    for (layers, rejected) in [("1", false), ("2", true), ("3", true)] {
        let mut fields = shared.clone();
        fields
            .as_object_mut()
            .unwrap()
            .insert("raft_layers".to_owned(), json!(layers));
        check(fields, rejected);
    }
}

#[test]
fn print_layer_nozzle_raft_zero_selector_uses_used_minimum_not_front() {
    for (layers, support, interface) in [("1", "1", "0"), ("2", "0", "1")] {
        check(
            json!({"nozzle_diameter":["0.2","0.4"],"filament_diameter":["1.75","1.75"],"filament_map":["2","2"],"layer_height":"0.1","initial_layer_print_height":"0.3","raft_layers":layers,"support_filament":support,"support_interface_filament":interface}),
            false,
        );
    }
}

#[test]
fn print_layer_nozzle_raft_direct_index_ignores_filament_map() {
    check(
        json!({"nozzle_diameter":["0.4","0.2"],"filament_diameter":["1.75","1.75"],"filament_map":["1","1"],"layer_height":"0.1","initial_layer_print_height":"0.3","raft_layers":"2","support_filament":"2","support_interface_filament":"2"}),
        true,
    );
}

#[test]
fn print_layer_nozzle_raft_out_of_range_uses_front_not_last_or_minimum() {
    check(
        json!({"nozzle_diameter":["0.4","0.2"],"filament_diameter":["1.75","1.75","1.75"],"filament_map":["2","2","2"],"layer_height":"0.1","initial_layer_print_height":"0.3","raft_layers":"2","support_filament":"3","support_interface_filament":"3"}),
        false,
    );
}

#[test]
fn print_layer_nozzle_error_precedes_width_and_layer_reset_check() {
    check(json!({"layer_height":"0.5","line_width":"3"}), true);
    check(
        json!({"initial_layer_print_height":"0.5","line_width":"0.2"}),
        true,
    );
    check(
        json!({"layer_height":"0.5","use_relative_e_distances":"1","layer_change_gcode":"g92 e0"}),
        true,
    );
}

#[test]
fn print_layer_nozzle_stl_adapter_preserves_initial_height() {
    let input = include_bytes!("../../../../../tests/parity/cube10.stl");
    let options: ares_core::SliceOptions =
        serde_json::from_value(json!({"layer_height":0.2,"initial_layer_print_height":0.5}))
            .unwrap();
    let actual = ares_core::validate_stl(input, &options);
    let directory = super::evidence_directory("print-layer-stl-");
    std::fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({"actual":actual.as_ref().map_err(ToString::to_string)}))
            .unwrap(),
    )
    .unwrap();
    assert!(
        matches!(actual,Err(SliceError::InvalidInput(ref text)) if text=="Layer height cannot exceed nozzle diameter."),
        "{actual:?}; {directory:?}"
    );
}
