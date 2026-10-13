use crate::{LayerConfigRange, SliceError};

use super::parameters::SlicingParameters;

const EPSILON: f64 = 1e-4;

// PrintObject.cpp:3805-3833: validate stored data before regenerating from ranges.
pub(super) fn update_layer_height_profile(
    parameters: &SlicingParameters,
    stored: &[f64],
    ranges: &[LayerConfigRange],
) -> Result<Vec<f64>, SliceError> {
    let mut profile = stored.to_vec();
    if !profile.is_empty()
        && (!profile.len().is_multiple_of(2)
            || (profile[profile.len() - 2] - parameters.object_print_z_uncompensated_max
                + parameters.object_print_z_min)
                .abs()
                > 1e-3)
    {
        profile.clear();
    }
    if profile.is_empty() || profile[1] != parameters.first_object_layer_height {
        // Retain the existing raw range rejection before Slicing.cpp:249 would
        // dereference a missing height. A retained profile bypasses this path.
        if ranges.iter().any(|range| range.layer_height().is_none()) {
            return Err(SliceError::UnsupportedProjectFeature(
                "layer_config_ranges".to_owned(),
            ));
        }
        return Ok(layer_height_profile_from_ranges(parameters, ranges));
    }
    Ok(profile)
}

pub(super) fn layer_height_profile_from_ranges(
    parameters: &SlicingParameters,
    ranges: &[LayerConfigRange],
) -> Vec<f64> {
    let first = parameters.first_object_layer_height;
    let top = parameters.object_print_z_uncompensated_max - parameters.object_print_z_min;
    let compensated_top = parameters.object_print_z_max - parameters.object_print_z_min;
    let mut profile = Vec::with_capacity(ranges.len() * 4 + 8);
    let mut previous_high = None;

    if parameters.first_object_layer_height_fixed() {
        append(&mut profile, 0.0, first);
        append(&mut profile, first, first);
        previous_high = Some(first);
    }
    for range in ranges {
        let Some(height) = range.layer_height() else {
            continue;
        };
        let mut low = range.min_z();
        let high = range.max_z().min(compensated_top);
        if let Some(previous) = previous_high {
            low = low.max(previous);
        }
        if low + EPSILON < high {
            let last_z = if profile.is_empty() {
                0.0
            } else {
                profile[profile.len() - 2]
            };
            if low > last_z + EPSILON {
                append(&mut profile, last_z, parameters.layer_height);
                append(&mut profile, low, parameters.layer_height);
            }
            append(&mut profile, low, height.0);
            append(&mut profile, high, height.0);
            previous_high = Some(high);
        }
    }
    let remaining_start = if profile.is_empty() {
        0.0
    } else {
        profile[profile.len() - 2]
    };
    if remaining_start < top {
        append(&mut profile, remaining_start, parameters.layer_height);
        append(&mut profile, top, parameters.layer_height);
    }
    profile
}

fn append(profile: &mut Vec<f64>, z: f64, height: f64) {
    if !profile.is_empty() {
        let last_z = profile.len() - 2;
        let last_height = profile.len() - 1;
        if approximately_equal(profile[last_height], height)
            && approximately_equal(profile[last_z], z)
        {
            return;
        }
        if approximately_equal(profile[last_height], height)
            && profile.len() >= 4
            && approximately_equal(profile[profile.len() - 3], height)
        {
            profile[last_z] = z;
            return;
        }
    }
    profile.extend([z, height]);
}

fn approximately_equal(left: f64, right: f64) -> bool {
    (left - right).abs() < EPSILON
}
