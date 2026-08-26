use super::{MAX_COORDINATE_ROUNDING_DIFFERENCE_MM, parser::Travel};

const MAX_ARC_CENTER_DIFFERENCE_MM: f64 = 0.011;

pub(super) fn compare(layer: usize, expected: &[Travel], actual: &[Travel]) -> Result<(), String> {
    if expected.len() != actual.len() {
        return Err(format!(
            "layer {layer} travel geometry count differs: expected {}, actual {}",
            expected.len(),
            actual.len()
        ));
    }
    match_each(expected, actual, travel_geometry_matches).map_err(|expected| {
        format!("layer {layer} travel geometry differs: no match for {expected:?}")
    })
}
fn match_each<'a>(
    expected: &'a [Travel],
    actual: &[Travel],
    matches: impl Fn(&Travel, &Travel) -> bool,
) -> Result<(), &'a Travel> {
    let mut matched = vec![false; actual.len()];
    for expected in expected {
        if let Some(index) = actual
            .iter()
            .enumerate()
            .position(|(index, actual)| !matched[index] && matches(expected, actual))
        {
            matched[index] = true;
        } else {
            return Err(expected);
        }
    }
    Ok(())
}

fn travel_geometry_matches(expected: &Travel, actual: &Travel) -> bool {
    let expected_fields = expected.key.split('|').collect::<Vec<_>>();
    let actual_fields = actual.key.split('|').collect::<Vec<_>>();
    if expected_fields.len() != 10 || actual_fields.len() != 10 {
        return false;
    }
    let command_matches = expected_fields[0] == actual_fields[0];
    let z_matches = [3, 6].into_iter().all(|index| {
        numeric_matches(
            expected_fields[index],
            actual_fields[index],
            MAX_COORDINATE_ROUNDING_DIFFERENCE_MM,
        )
    });
    let arc_matches = expected_fields[0] != "G3"
        || arc_radius_matches(
            expected_fields[7],
            expected_fields[8],
            actual_fields[7],
            actual_fields[8],
        ) && expected_fields[9] == actual_fields[9];
    command_matches && z_matches && arc_matches
}

fn arc_radius_matches(expected_i: &str, expected_j: &str, actual_i: &str, actual_j: &str) -> bool {
    let radius = |i: &str, j: &str| {
        i.parse::<f64>()
            .ok()
            .zip(j.parse::<f64>().ok())
            .map(|(i, j)| i.hypot(j))
    };
    radius(expected_i, expected_j)
        .zip(radius(actual_i, actual_j))
        .is_some_and(|(expected, actual)| (expected - actual).abs() <= MAX_ARC_CENTER_DIFFERENCE_MM)
}

fn numeric_matches(expected: &str, actual: &str, tolerance: f64) -> bool {
    if expected.is_empty() || actual.is_empty() {
        return expected == actual;
    }
    expected
        .parse::<f64>()
        .ok()
        .zip(actual.parse::<f64>().ok())
        .is_some_and(|(expected, actual)| (expected - actual).abs() <= tolerance)
}
