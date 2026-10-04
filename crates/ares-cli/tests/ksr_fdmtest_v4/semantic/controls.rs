//! Control-event canonicalization and comparison: acceleration modes,
//! fan assignments, extruder-mode lines.

use super::SemanticGcode;

pub(super) fn canonicalize_controls(parsed: &SemanticGcode) -> Vec<Vec<String>> {
    let mut fans = std::collections::BTreeMap::<String, String>::new();
    let mut plane_xy = false;
    for line in &parsed.preamble {
        if line == "G17" {
            plane_xy = true;
        } else if line == "G18" || line == "G19" {
            plane_xy = false;
        } else if line.starts_with("M106 ") {
            apply_fan(&mut fans, line);
        } else if line == "M107" || line.starts_with("M107 ") {
            fans.insert(fan_target(line), "0".to_owned());
        }
    }
    parsed
        .layers
        .iter()
        .map(|layer| {
            layer
                .controls
                .iter()
                .filter(|line| keep_control(line, &mut fans, &mut plane_xy))
                .cloned()
                .collect()
        })
        .collect()
}

fn keep_control(
    line: &str,
    fans: &mut std::collections::BTreeMap<String, String>,
    plane_xy: &mut bool,
) -> bool {
    if line == "G17" {
        return !std::mem::replace(plane_xy, true);
    }
    if line == "G18" || line == "G19" {
        *plane_xy = false;
        return true;
    }
    if line.starts_with("M106 ") {
        if let Some((fan, speed)) = fan_assignment(line) {
            if fans.get(&fan) == Some(&speed) {
                return false;
            }
            fans.insert(fan, speed);
        }
        return true;
    }
    if line == "M107" || line.starts_with("M107 ") {
        let fan = fan_target(line);
        if fans.get(&fan).is_some_and(|s| s == "0") {
            return false;
        }
        fans.insert(fan, "0".to_owned());
    }
    true
}

fn fan_target(line: &str) -> String {
    let mut fan = "0".to_owned();
    for token in line.split_ascii_whitespace().skip(1) {
        if let Some(value) = token.strip_prefix('P') {
            fan = value.to_owned();
        }
    }
    fan
}

fn apply_fan(fans: &mut std::collections::BTreeMap<String, String>, line: &str) {
    if let Some((fan, speed)) = fan_assignment(line) {
        fans.insert(fan, speed);
    }
}

// `M106 P<n> S<v>` → (fan, speed); `M106 S<v>` targets fan 0. Parameter
// forms without S (e.g. `M106 P3 R1`) leave fan state untouched.
fn fan_assignment(line: &str) -> Option<(String, String)> {
    let mut fan = "0".to_owned();
    let mut speed = None;
    for token in line.split_ascii_whitespace().skip(1) {
        if let Some(value) = token.strip_prefix('P') {
            fan = value.to_owned();
        } else if let Some(value) = token.strip_prefix('S') {
            speed = Some(value.to_owned());
        }
    }
    speed.map(|speed| (fan, speed))
}

pub(super) fn compare_control_events(
    layer: usize,
    expected: &[String],
    actual: &[String],
) -> Result<(), String> {
    if expected != actual {
        let position = expected
            .iter()
            .zip(actual.iter())
            .position(|(expected, actual)| expected != actual);
        let shown = position.unwrap_or(expected.len().min(actual.len()));
        return Err(format!(
            "layer {layer} control events differs at index {shown}: \
             expected {:?} vs actual {:?} (lengths {} vs {})",
            expected.get(shown),
            actual.get(shown),
            expected.len(),
            actual.len()
        ));
    }
    Ok(())
}
