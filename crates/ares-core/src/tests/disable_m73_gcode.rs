use super::*;

#[tokio::test]
async fn default_slice_emits_first_and_last_m73_progress_lines() {
    let output = m73_output(json!({})).await.unwrap();

    assert_line_before(&output, "M73 P0 R0", ";LAYER_CHANGE");
    assert_line_before(&output, "M73 P100 R0", "M2");
    assert_eq!(m73_lines(&output), vec!["M73 P0 R0", "M73 P100 R0"]);
}

#[tokio::test]
async fn silent_mode_marlin_legacy_emits_stealth_m73_progress_lines() {
    let output = m73_output(json!({
        "silent_mode": true
    }))
    .await
    .unwrap();

    assert_eq!(
        m73_lines(&output),
        vec!["M73 P0 R0", "M73 Q0 S0", "M73 P100 R0", "M73 Q100 S0"]
    );
    assert_line_before(&output, "M73 P0 R0", "M73 Q0 S0");
    assert_line_before(&output, "M73 P100 R0", "M73 Q100 S0");
}

#[tokio::test]
async fn silent_mode_marlin2_emits_stealth_m73_progress_lines() {
    let output = m73_output(json!({
        "gcode_flavor": "marlin2",
        "silent_mode": true
    }))
    .await
    .unwrap();

    assert_eq!(
        m73_lines(&output),
        vec!["M73 P0 R0", "M73 Q0 S0", "M73 P100 R0", "M73 Q100 S0"]
    );
}

#[tokio::test]
async fn silent_mode_non_marlin_flavor_does_not_emit_stealth_m73_lines() {
    let output = m73_output(json!({
        "gcode_flavor": "reprapfirmware",
        "silent_mode": true
    }))
    .await
    .unwrap();

    assert_eq!(m73_lines(&output), vec!["M73 P0 R0", "M73 P100 R0"]);
}

#[tokio::test]
async fn disable_m73_true_suppresses_ares_m73_progress_lines() {
    let output = m73_output(json!({
        "disable_m73": true
    }))
    .await
    .unwrap();

    assert!(!output.lines().any(|line| line.starts_with("M73 ")));
}

#[tokio::test]
async fn disable_m73_true_suppresses_silent_mode_m73_progress_lines() {
    let output = m73_output(json!({
        "disable_m73": true,
        "silent_mode": true
    }))
    .await
    .unwrap();

    assert!(m73_lines(&output).is_empty());
}

#[tokio::test]
async fn disable_m73_rejects_non_boolean_values() {
    let err = m73_output(json!({
        "disable_m73": "true"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("disable_m73 must be a boolean"));
}

#[tokio::test]
async fn silent_mode_rejects_non_boolean_values() {
    let err = m73_output(json!({
        "silent_mode": "true"
    }))
    .await
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("silent_mode must be a boolean"));
}

async fn m73_output(extra: serde_json::Value) -> Result<String, SliceError> {
    let options: SliceOptions = serde_json::from_value(extra).unwrap();
    let output = slice(square_pyramid_ascii_stl(), options).await?;
    Ok(String::from_utf8(output).unwrap())
}

fn m73_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with("M73 "))
        .collect()
}

fn assert_line_before(output: &str, before: &str, after: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let before_index = lines.iter().position(|line| *line == before).unwrap();
    let after_index = lines.iter().position(|line| *line == after).unwrap();

    assert!(
        before_index < after_index,
        "expected {before:?} before {after:?}"
    );
}
