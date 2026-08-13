use crate::RegionOptions;

#[cfg(test)]
mod tests;

pub(in crate::project_slice) fn apply_internal_bridge_angle_override(
    detected_angle: f64,
    region: &RegionOptions,
    model_rotation_rad: f64,
) -> f64 {
    if region.internal_bridge_angle.0 > 0.0 {
        let custom_angle_rad = std::f64::consts::PI * region.internal_bridge_angle.0 / 180.0;
        if region.relative_bridge_angle.0 {
            detected_angle + custom_angle_rad
        } else if region.align_infill_direction_to_model.0 {
            custom_angle_rad + model_rotation_rad
        } else {
            custom_angle_rad
        }
    } else {
        detected_angle
    }
}
