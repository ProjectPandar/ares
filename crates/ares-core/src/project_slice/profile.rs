use super::parameters::SlicingParameters;

const EPSILON: f64 = 1e-4;

pub(super) fn fixed_layer_height_profile(parameters: &SlicingParameters) -> Vec<f64> {
    let first = parameters.first_object_layer_height;
    let top = parameters.object_print_z_uncompensated_max;
    let mut profile = Vec::with_capacity(8);

    append(&mut profile, 0.0, first);
    append(&mut profile, first, first);
    let remaining_start = profile[profile.len() - 2];
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
