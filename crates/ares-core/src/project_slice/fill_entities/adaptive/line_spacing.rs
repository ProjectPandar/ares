use crate::{FloatOrPercent, OrcaFloats, RegionOptions};

// FillAdaptive.cpp:304-323: each region's config width and density are double.
// Flow.cpp:21-36 auto width is float.
pub(in crate::project_slice) fn configured(
    region: &RegionOptions,
    nozzles: &OrcaFloats,
    multiline: i32,
) -> f64 {
    let nozzle = nozzles
        .0
        .iter()
        .map(|value| value.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let width = match region.sparse_infill_line_width {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => nozzle * value.0 / 100.0,
    };
    let width = if width == 0.0 {
        f64::from(1.125_f32 * nozzle as f32)
    } else {
        width
    };
    // Source 0.333333333f rounds to this f32, then promotes to double.
    width / ((region.sparse_infill_density.0 / 100.0) * f64::from(0.333_333_34_f32))
        * f64::from(multiline)
}
