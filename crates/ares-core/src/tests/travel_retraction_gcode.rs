use super::*;

mod filament_minimum_travel;
mod filament_restart_extra;
mod filament_retract_before_wipe;
mod filament_retract_lift_gates;
mod filament_speed;
mod filament_wipe;
mod filament_wipe_distance;
mod filament_z_hop;
mod reduce_infill;
mod wipe;
mod z_hop;
mod z_hop_type;

#[tokio::test]
async fn default_minimum_travel_retracts_before_long_ordinary_travel() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    let travel = line_index(&output, "G1 X-0.5 Y0 F7200 ; travel");
    let retract = previous_line_index(&output, travel, "G1 E-0.8 F1800 ; retract");
    let unretract = next_line_index(&output, travel, "G1 E0.8 F1800 ; unretract");
    let next_print = next_print_line_index(&output, travel);

    assert!(retract < travel);
    assert!(travel < unretract);
    assert!(unretract < next_print);
}

#[tokio::test]
async fn high_minimum_travel_suppresses_ordinary_travel_retraction() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 100,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    assert!(
        output
            .lines()
            .any(|line| line == "G1 X-0.5 Y0 F7200 ; travel")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E-0.8 F1800 ; retract")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E0.8 F1800 ; unretract")
    );
}

#[tokio::test]
async fn minimum_travel_uses_first_vector_value() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": [100, 0],
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    assert!(
        output
            .lines()
            .any(|line| line == "G1 X-0.5 Y0 F7200 ; travel")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E-0.8 F1800 ; retract")
    );
}

#[tokio::test]
async fn restart_extra_changes_travel_unretract_only() {
    let output = output_for(json!({
        "retract_when_changing_layer": false,
        "retraction_length": 0.5,
        "retract_restart_extra": 0.2,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;

    assert!(
        output
            .lines()
            .any(|line| line == "G1 E-0.5 F1800 ; retract")
    );
    assert!(
        output
            .lines()
            .any(|line| line == "G1 E0.7 F1800 ; unretract")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E-0.7 F1800 ; retract")
    );
}

#[tokio::test]
async fn invalid_minimum_travel_values_are_rejected_with_option_key() {
    for value in [
        json!([]),
        json!(-0.1),
        json!("inf"),
        json!("bad"),
        json!([2, "bad"]),
        json!([2, -0.1]),
    ] {
        let err = output_result(json!({
            "retraction_minimum_travel": value
        }))
        .await
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains("retraction_minimum_travel"),
            "retraction_minimum_travel was missing from {err}"
        );
    }
}

#[tokio::test]
async fn pending_travel_retraction_crosses_layer_change_without_double_retract() {
    let output = boundary_output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 0.25,
        "z_hop": 0,
        "gcode_comments": true
    }));
    let first_retract = line_index(&output, "G1 E-0.8 F1800 ; retract");
    let layer_change = next_line_index(&output, first_retract, ";LAYER_CHANGE");
    let unretract = next_line_index(&output, layer_change, "G1 E0.8 F1800 ; unretract");
    let next_print = next_print_line_index(&output, layer_change);

    assert_eq!(
        output
            .lines()
            .take(unretract)
            .filter(|line| *line == "G1 E-0.8 F1800 ; retract")
            .count(),
        1
    );
    assert!(first_retract < layer_change);
    assert!(layer_change < unretract);
    assert!(unretract < next_print);
}

#[tokio::test]
async fn pending_layer_change_retraction_suppresses_initial_travel_retract() {
    let output = output_for(json!({
        "retract_when_changing_layer": true,
        "retraction_minimum_travel": 0.25,
        "z_hop": 0,
        "gcode_comments": true
    }))
    .await;
    let second = layer_section(&output, 1);
    let layer_retract = line_index(second, "G1 E-0.8 F1800 ; retract");
    let travel = next_line_index(second, layer_retract, "G1 X0.25 Y-0.75 F7200 ; travel");
    let unretract = next_line_index(second, travel, "G1 E0.8 F1800 ; unretract");
    let first_print = next_print_line_index(second, travel);

    assert_eq!(
        second
            .lines()
            .take(unretract)
            .filter(|line| *line == "G1 E-0.8 F1800 ; retract")
            .count(),
        1
    );
    assert!(layer_retract < travel);
    assert!(travel < unretract);
    assert!(unretract < first_print);
}

async fn output_result(extra: serde_json::Value) -> Result<String, SliceError> {
    let mut options = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.25,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "is_infill_first": true,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    options
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let output = slice(
        square_pyramid_ascii_stl(),
        serde_json::from_value(options).unwrap(),
    )
    .await?;

    Ok(String::from_utf8(output).unwrap())
}

async fn output_for(extra: serde_json::Value) -> String {
    output_result(extra).await.unwrap()
}

fn boundary_output_for(extra: serde_json::Value) -> String {
    let options = options_for(extra);
    let pipeline =
        crate::pipeline::layer_change_test_support::pending_travel_layer_boundary_pipeline(
            &options,
        );
    String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap()
}

fn options_for(extra: serde_json::Value) -> SliceOptions {
    let mut options = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    options
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_value(options).unwrap()
}

fn line_index(output: &str, expected: &str) -> usize {
    output
        .lines()
        .position(|line| line == expected)
        .unwrap_or_else(|| panic!("{expected} missing from output"))
}

fn layer_section(output: &str, layer: u32) -> &str {
    let marker = format!(";LAYER:{layer}\n");
    let start = output.find(&marker).unwrap();
    let rest = &output[start..];
    let next = rest[marker.len()..]
        .find("\n;LAYER_CHANGE\n")
        .map(|next| marker.len() + next)
        .unwrap_or(rest.len());
    &rest[..next]
}

fn previous_line_index(output: &str, before: usize, expected: &str) -> usize {
    output
        .lines()
        .take(before)
        .collect::<Vec<_>>()
        .iter()
        .rposition(|line| *line == expected)
        .unwrap_or_else(|| panic!("{expected} missing before line {before}"))
}

fn next_line_index(output: &str, after: usize, expected: &str) -> usize {
    output
        .lines()
        .enumerate()
        .skip(after + 1)
        .find_map(|(index, line)| (line == expected).then_some(index))
        .unwrap_or_else(|| panic!("{expected} missing after line {after}"))
}

fn next_print_line_index(output: &str, after: usize) -> usize {
    output
        .lines()
        .enumerate()
        .skip(after + 1)
        .find_map(|(index, line)| {
            (line.starts_with("G1 X") && line.contains(" E")).then_some(index)
        })
        .unwrap_or_else(|| panic!("print move missing after line {after}"))
}
