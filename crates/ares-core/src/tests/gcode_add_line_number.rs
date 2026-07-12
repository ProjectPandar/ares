use super::*;

#[tokio::test]
async fn enabled_gcode_add_line_number_numbers_every_output_line() {
    let output = numbered_output(json!({
        "gcode_add_line_number": true,
        "sparse_infill_density": 0,
        "filament_max_volumetric_speed": 0.0
    }))
    .await
    .unwrap();
    let lines = output.lines().collect::<Vec<_>>();

    assert!(lines[0].starts_with("N1 "));
    assert!(
        lines
            .iter()
            .any(|line| numbered_payload(line) == Some("G90"))
    );
    assert!(
        lines
            .iter()
            .any(|line| numbered_payload(line) == Some("M73 P0 R0"))
    );
    assert!(
        lines
            .iter()
            .any(|line| numbered_payload(line) == Some("M2"))
    );
    assert!(output.ends_with('\n'));

    for (index, line) in lines.iter().enumerate() {
        assert!(
            line.starts_with(&format!("N{} ", index + 1)),
            "line {} was not sequentially numbered: {line}",
            index + 1
        );
    }
}

#[tokio::test]
async fn disabled_gcode_add_line_number_keeps_command_lines_unnumbered() {
    let output = numbered_output(json!({
        "gcode_add_line_number": false,
        "sparse_infill_density": 0
    }))
    .await
    .unwrap();

    assert!(output.lines().any(|line| line == "G90"));
    assert!(output.lines().any(|line| line == "M2"));
    assert!(!output.lines().any(|line| line.starts_with("N1 ")));
}

#[tokio::test]
async fn gcode_add_line_number_rejects_non_boolean_values() {
    let err = numbered_output(json!({
        "gcode_add_line_number": "true"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(
        err.to_string()
            .contains("gcode_add_line_number must be a boolean")
    );
}

async fn numbered_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let options: SliceOptions = serde_json::from_value(extra).unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await?;
    Ok(String::from_utf8(output).unwrap())
}

fn numbered_payload(line: &str) -> Option<&str> {
    let (prefix, payload) = line.split_once(' ')?;
    prefix
        .strip_prefix('N')?
        .parse::<usize>()
        .ok()
        .map(|_| payload)
}
