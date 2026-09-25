use super::CONNECT_INFILL_FEATURES;
use crate::semantic::{motion_matches, numeric_matches, parser::Deposition};

const MAX_CROSS_TARGET_DEPOSITION_COORDINATE_DIFFERENCE_MM: f64 = 0.15;
const MAX_CROSS_TARGET_DEPOSITION_EXTRUSION_DIFFERENCE_MM: f64 = 0.005;
const MAX_CROSS_TARGET_SPLIT_EXTRUSION_MM: f64 = 0.0015;
const MAX_CROSS_TARGET_DEPOSITION_FEED_DIFFERENCE: f64 = 20.0;

pub(crate) fn compare_deposition_cross_target(
    layer: usize,
    expected: &[Deposition],
    actual: &[Deposition],
) -> Result<(), String> {
    let (expected, actual): (Vec<_>, Vec<_>) = if expected.len() == actual.len() {
        (expected.iter().collect(), actual.iter().collect())
    } else {
        (
            expected
                .iter()
                .filter(|deposition| !is_split_move(deposition))
                .collect(),
            actual
                .iter()
                .filter(|deposition| !is_split_move(deposition))
                .collect(),
        )
    };
    if expected.len() != actual.len() {
        return Err(format!(
            "layer {layer} deposition count differs across targets: expected {}, actual {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, (expected, actual)) in expected.iter().zip(&actual).enumerate() {
        if !deposition_matches_cross_target(expected, actual) {
            return Err(format!(
                "layer {layer} deposition {} differs across targets: expected {expected:?}, \
                 actual {actual:?}",
                index + 1
            ));
        }
    }
    Ok(())
}

fn is_split_move(deposition: &Deposition) -> bool {
    deposition.extrusion.parse::<f64>().unwrap().abs() <= MAX_CROSS_TARGET_SPLIT_EXTRUSION_MM
}

fn deposition_matches_cross_target(expected: &Deposition, actual: &Deposition) -> bool {
    expected.feature == actual.feature
        && expected.width == actual.width
        && expected.acceleration == actual.acceleration
        && expected.fans == actual.fans
        && cross_target_feed_matches(expected.feed, actual.feed)
        && numeric_matches(
            expected.extrusion.parse().unwrap(),
            actual.extrusion.parse().unwrap(),
            MAX_CROSS_TARGET_DEPOSITION_EXTRUSION_DIFFERENCE_MM,
        )
        && motion_matches(
            &expected.motion,
            &actual.motion,
            MAX_CROSS_TARGET_DEPOSITION_COORDINATE_DIFFERENCE_MM,
            MAX_CROSS_TARGET_DEPOSITION_COORDINATE_DIFFERENCE_MM,
            CONNECT_INFILL_FEATURES.contains(&expected.feature.as_str()),
        )
}

fn cross_target_feed_matches(expected: f64, actual: f64) -> bool {
    let absolute = (expected - actual).abs();
    let relative = absolute / expected.abs().max(f64::EPSILON);
    absolute <= MAX_CROSS_TARGET_DEPOSITION_FEED_DIFFERENCE && relative <= 0.01
}
