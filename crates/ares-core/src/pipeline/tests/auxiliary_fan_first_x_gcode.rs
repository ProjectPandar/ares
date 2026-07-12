use super::*;
use crate::{PrintPathRole, gcode::format_gcode, pipeline::test_support::single_path_pipeline};
use serde_json::json;

#[test]
fn first_x_layer_auxiliary_fan_speed_emits_initial_p2_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "first_x_layer_fan_speed": 12.5,
        "additional_cooling_fan_speed": 70,
        "close_additional_fan_first_x_layers": 2
    }))
    .unwrap();
    let pipeline = single_path_pipeline(&options, PrintPathRole::ExternalPerimeter, 0);

    let output = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(
        auxiliary_fan_lines(&output),
        vec!["M106 P2 S33", "M106 P2 S0"]
    );
    assert_line_after(&output, ";LAYER:0", "M106 P2 S33");
    assert_line_before(&output, "M106 P2 S0", "M2");
}

fn auxiliary_fan_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M106 P2 S"))
        .collect()
}

fn assert_line_after(output: &str, first: &str, second: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let first_index = lines
        .iter()
        .position(|line| *line == first)
        .unwrap_or_else(|| panic!("missing first line {first:?} in:\n{output}"));
    let second_index = lines
        .iter()
        .position(|line| *line == second)
        .unwrap_or_else(|| panic!("missing second line {second:?} in:\n{output}"));

    assert!(
        second_index > first_index,
        "expected {second:?} after {first:?} in:\n{output}"
    );
}

fn assert_line_before(output: &str, first: &str, second: &str) {
    assert_line_after(output, first, second);
}
