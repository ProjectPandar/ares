use super::*;

#[test]
fn filament_change_extrusion_role_gcode_is_noop_when_absent_empty_or_empty_first_array_entry() {
    let absent = role_change_output(json!({}));
    let empty = role_change_output(json!({
        "filament_change_extrusion_role_gcode": ""
    }));
    let empty_array = role_change_output(json!({
        "filament_change_extrusion_role_gcode": []
    }));
    let empty_first_array_entry = role_change_output(json!({
        "filament_change_extrusion_role_gcode": [""]
    }));

    assert_eq!(without_option_count(&absent), without_option_count(&empty));
    assert_eq!(
        without_option_count(&absent),
        without_option_count(&empty_array)
    );
    assert_eq!(
        without_option_count(&absent),
        without_option_count(&empty_first_array_entry)
    );
    assert!(!absent.contains(";FILAMENT-ROLE"));
}

#[test]
fn filament_change_extrusion_role_gcode_uses_first_string_array_entry() {
    let output = role_change_output(json!({
        "filament_change_extrusion_role_gcode": [
            ";FILAMENT-FIRST [last_extrusion_role]->[extrusion_role]",
            ";FILAMENT-SECOND [last_extrusion_role]->[extrusion_role]"
        ]
    }));

    assert_line_before(
        &output,
        ";FILAMENT-FIRST internal_perimeter->external_perimeter",
        ";EXTRUSION:print:external_perimeter:",
    );
    assert!(!output.contains(";FILAMENT-SECOND"));
}

#[test]
fn filament_change_extrusion_role_gcode_emits_on_print_role_change_before_print_move() {
    let output = role_change_output(json!({
        "filament_change_extrusion_role_gcode": ";FILAMENT-ROLE [last_extrusion_role]->[extrusion_role] L[layer_num] Z[layer_z]"
    }));

    assert_line_before(
        &output,
        ";FILAMENT-ROLE internal_perimeter->external_perimeter L1 Z0.2",
        ";EXTRUSION:print:external_perimeter:",
    );
    assert_line_before(
        &output,
        ";FILAMENT-ROLE external_perimeter->sparse_infill L1 Z0.2",
        ";EXTRUSION:print:sparse_infill:",
    );
}

#[test]
fn filament_change_extrusion_role_gcode_does_not_emit_before_first_print_role() {
    let output = role_change_output(json!({
        "filament_change_extrusion_role_gcode": ";FILAMENT-ROLE [last_extrusion_role]->[extrusion_role]"
    }));

    let first_role_change = output
        .lines()
        .position(|line| line.starts_with(";FILAMENT-ROLE"));
    let first_internal_print = output
        .lines()
        .position(|line| line.starts_with(";EXTRUSION:print:internal_perimeter:"));

    assert!(first_internal_print.is_some());
    assert!(first_role_change.unwrap() > first_internal_print.unwrap());
    assert!(!output.contains(";FILAMENT-ROLE ->internal_perimeter"));
}

#[test]
fn filament_change_extrusion_role_gcode_emits_between_machine_and_process_role_change_gcode() {
    let output = role_change_output(json!({
        "change_extrusion_role_gcode": ";MACHINE-ROLE [last_extrusion_role]->[extrusion_role]",
        "filament_change_extrusion_role_gcode": ";FILAMENT-ROLE [last_extrusion_role]->[extrusion_role]",
        "process_change_extrusion_role_gcode": ";PROCESS-ROLE [last_extrusion_role]->[extrusion_role]"
    }));

    assert_line_before(
        &output,
        ";MACHINE-ROLE internal_perimeter->external_perimeter",
        ";FILAMENT-ROLE internal_perimeter->external_perimeter",
    );
    assert_line_before(
        &output,
        ";FILAMENT-ROLE internal_perimeter->external_perimeter",
        ";PROCESS-ROLE internal_perimeter->external_perimeter",
    );
    assert_line_before(
        &output,
        ";PROCESS-ROLE internal_perimeter->external_perimeter",
        ";EXTRUSION:print:external_perimeter:",
    );
}

#[test]
fn filament_change_extrusion_role_gcode_ignores_travel_role_changes() {
    let output = role_change_output(json!({
        "filament_change_extrusion_role_gcode": ";FILAMENT-ROLE [last_extrusion_role]->[extrusion_role]"
    }));

    assert_line_before(
        &output,
        ";EXTRUSION:travel:external_perimeter:",
        ";FILAMENT-ROLE internal_perimeter->external_perimeter",
    );
    assert_line_before(
        &output,
        ";EXTRUSION:travel:sparse_infill:",
        ";FILAMENT-ROLE external_perimeter->sparse_infill",
    );
}

#[test]
fn filament_change_extrusion_role_gcode_replaces_brace_placeholders() {
    let output = role_change_output(json!({
        "filament_change_extrusion_role_gcode": ";FILAMENT-BRACE {last_extrusion_role}->{extrusion_role} {layer_num} {layer_z}"
    }));

    assert_line_before(
        &output,
        ";FILAMENT-BRACE internal_perimeter->external_perimeter 1 0.2",
        ";EXTRUSION:print:external_perimeter:",
    );
}

#[test]
fn filament_change_extrusion_role_gcode_keeps_unknown_conditionals_and_expression_placeholders() {
    let output = role_change_output(json!({
        "filament_change_extrusion_role_gcode": "{if extrusion_role == \"sparse_infill\"}\n;FILAMENT {layer_num+1} [future] [extrusion_role]\n{endif}"
    }));

    assert_line_before(
        &output,
        "{if extrusion_role == \"sparse_infill\"}",
        ";FILAMENT {layer_num+1} [future] external_perimeter",
    );
    assert_line_before(
        &output,
        ";FILAMENT {layer_num+1} [future] external_perimeter",
        "{endif}",
    );
}

#[test]
fn filament_change_extrusion_role_gcode_rejects_invalid_values() {
    for value in [
        json!(7),
        json!([7]),
        json!(["", 7]),
        json!(["; first", false]),
    ] {
        let err = role_change_result(json!({
            "filament_change_extrusion_role_gcode": value
        }))
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string()
                .contains("filament_change_extrusion_role_gcode")
        );
    }
}

fn role_change_output(extra: serde_json::Value) -> String {
    role_change_result(extra).unwrap()
}

fn role_change_result(extra: serde_json::Value) -> Result<String, SliceError> {
    let options = merged_options(
        json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "wall_loops": 2,
            "line_width": 0.4,
            "sparse_infill_density": 50,
            "sparse_infill_line_width": 0.4,
            "minimum_sparse_infill_area": 0,
            "infill_direction": 0,
            "brim_width": 0,
            "skirt_loops": 0,
            "bottom_shell_layers": 0,
            "top_shell_layers": 0
        }),
        extra,
    );
    crate::gcode::format_gcode(
        &crate::pipeline::test_support::rectangular_pipeline(&options),
        &options,
    )
    .map(|bytes| String::from_utf8(bytes).unwrap())
}

fn without_option_count(output: &str) -> String {
    output
        .lines()
        .filter(|line| !line.starts_with("; option_count = "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_line_before(output: &str, first_prefix: &str, second_prefix: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines
        .iter()
        .position(|line| line.starts_with(first_prefix))
        .unwrap_or_else(|| panic!("missing {first_prefix}\n{}", role_lines(output)));
    let second_index = lines
        .iter()
        .position(|line| line.starts_with(second_prefix))
        .unwrap_or_else(|| panic!("missing {second_prefix}\n{}", role_lines(output)));
    assert!(
        first_index < second_index,
        "{first_index} !< {second_index}"
    );
}

fn role_lines(output: &str) -> String {
    output
        .lines()
        .filter(|line| {
            line.starts_with(";ROLE")
                || line.starts_with(";BRACE")
                || line.starts_with(";MACHINE-ROLE")
                || line.starts_with(";FILAMENT")
                || line.starts_with(";PROCESS")
                || line.starts_with(";EXTRUSION:")
                || line.starts_with(";MOVE:")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn merged_options(base: serde_json::Value, extra: serde_json::Value) -> SliceOptions {
    let mut base = base.as_object().unwrap().clone();
    for (key, value) in extra.as_object().unwrap() {
        base.insert(key.clone(), value.clone());
    }
    serde_json::from_value(serde_json::Value::Object(base)).unwrap()
}
