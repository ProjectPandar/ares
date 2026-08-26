use super::{MAX_COORDINATE_ROUNDING_DIFFERENCE_MM, MAX_WIPE_EXTRUSION_DIFFERENCE, exact_layer};

pub(super) fn compare(
    layer: usize,
    expected: &[Vec<String>],
    actual: &[Vec<String>],
) -> Result<(), String> {
    let matches = expected.len() == actual.len()
        && expected.iter().zip(actual).all(|(expected, actual)| {
            expected.len() == actual.len()
                && expected
                    .iter()
                    .zip(actual)
                    .all(|(expected, actual)| event_matches(expected, actual))
        });
    if matches {
        Ok(())
    } else {
        exact_layer(layer, "island lifecycle", &expected, &actual)
    }
}

fn event_matches(expected: &str, actual: &str) -> bool {
    if expected == actual {
        return true;
    }
    let expected = expected.split('|').collect::<Vec<_>>();
    let actual = actual.split('|').collect::<Vec<_>>();
    expected.len() == 14
        && actual.len() == 14
        && expected[0] == "WIPE"
        && actual[0] == "WIPE"
        && expected
            .iter()
            .zip(&actual)
            .enumerate()
            .all(|(index, (expected, actual))| match index {
                4..=9 => expected
                    .parse::<f64>()
                    .ok()
                    .zip(actual.parse::<f64>().ok())
                    .is_some_and(|(expected, actual)| {
                        (expected - actual).abs() <= MAX_COORDINATE_ROUNDING_DIFFERENCE_MM
                    }),
                12 => expected
                    .parse::<f64>()
                    .ok()
                    .zip(actual.parse::<f64>().ok())
                    .is_some_and(|(expected, actual)| {
                        (expected - actual).abs() <= MAX_WIPE_EXTRUSION_DIFFERENCE
                    }),
                _ => expected == actual,
            })
}
