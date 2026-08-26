use super::parser::Travel;

const MAX_ARC_CENTER_DIFFERENCE_MM: f64 = 0.011;
const MAX_COORDINATE_ROUNDING_DIFFERENCE_MM: f64 = 0.001_1;

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
    let expected = &expected.motion;
    let actual = &actual.motion;
    let command_matches = expected.command == actual.command;
    let z_matches = [
        (&expected.start.z, &actual.start.z),
        (&expected.end.z, &actual.end.z),
    ]
    .into_iter()
    .all(|(expected, actual)| {
        numeric_matches(expected, actual, MAX_COORDINATE_ROUNDING_DIFFERENCE_MM)
    });
    let arc_matches = expected.command != "G3"
        || arc_radius_matches(&expected.arc_center, &actual.arc_center)
            && expected.turns == actual.turns;
    command_matches && z_matches && arc_matches
}

fn arc_radius_matches(expected: &[Option<String>; 2], actual: &[Option<String>; 2]) -> bool {
    let radius = |center: &[Option<String>; 2]| -> Option<f64> {
        Some(
            center[0]
                .as_deref()?
                .parse::<f64>()
                .ok()?
                .hypot(center[1].as_deref()?.parse::<f64>().ok()?),
        )
    };
    radius(expected)
        .zip(radius(actual))
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
