use crate::{FloatOrPercent, OrcaFloats, ProcessInfillPattern, RegionOptions};

// FillAdaptive.cpp:296-309,344-354: the single contributing region's
// config width and density are double. Flow.cpp:21-36 auto width is float.
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

/// FillAdaptive.cpp:314-354 averages the contributing regions first, then
/// applies the multiline count from printing_region(0) to that object-wide
/// spacing. The region order is the get_create_region order from
/// PrintApply.cpp:1000-1012.
pub(in crate::project_slice) fn object_configured(
    regions: &[&RegionOptions],
    nozzles: &OrcaFloats,
    kind: ProcessInfillPattern,
    active_region_ids: &[usize],
) -> f64 {
    let contributing: Vec<_> = regions
        .iter()
        .enumerate()
        .filter(|(index, region)| {
            active_region_ids.contains(index)
                && region.sparse_infill_pattern == kind
                && region.sparse_infill_density.0 > 0.0
        })
        .map(|(_, region)| *region)
        .collect();
    if contributing.is_empty() {
        return 0.0;
    }
    let width = contributing
        .iter()
        .map(|region| absolute_width(region, nozzles))
        .sum::<f64>()
        / contributing.len() as f64;
    let density = contributing
        .iter()
        .map(|region| region.sparse_infill_density.0)
        .sum::<f64>()
        / contributing.len() as f64;
    let multiline = regions.first().map_or(1, |region| region.fill_multiline.0);
    width / ((density / 100.0) * f64::from(0.333_333_34_f32)) * f64::from(multiline)
}

fn absolute_width(region: &RegionOptions, nozzles: &OrcaFloats) -> f64 {
    let nozzle = nozzles
        .0
        .iter()
        .map(|value| value.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let width = match region.sparse_infill_line_width {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => nozzle * value.0 / 100.0,
    };
    if width == 0.0 {
        f64::from(1.125_f32 * nozzle as f32)
    } else {
        width
    }
}
