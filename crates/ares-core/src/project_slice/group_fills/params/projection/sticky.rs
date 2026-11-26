use crate::{
    ExtrusionRole, FloatOrPercent, ProcessAdaptiveInfillGradient, ProcessAdaptiveInfillMode,
    ProcessInfillPattern, RegionOptions,
    fill::adaptive_infill_field::is_adaptive_infill_pattern,
    geometry::CoordinateScale,
    project_slice::group_fills::{SurfaceFillParams, SurfaceFillPattern},
};

pub(super) fn apply_adaptive_infill_fields(params: &mut SurfaceFillParams, region: &RegionOptions) {
    // Fill.cpp:1024-1034: only TPMS sparse infill is graded; reset otherwise
    // because `params` is reused across surfaces. Adaptive density replaces
    // `gyroid_optimized`.
    let adaptive_pattern = matches!(
        params.pattern,
        SurfaceFillPattern::Configured(pattern) if is_adaptive_infill_pattern(pattern)
    );
    params.adaptive_infill_mode =
        if adaptive_pattern && params.extrusion_role == ExtrusionRole::InternalInfill {
            region.adaptive_infill_mode
        } else {
            ProcessAdaptiveInfillMode::Disabled
        };
    let adaptive_active = params.adaptive_infill_mode != ProcessAdaptiveInfillMode::Disabled;
    params.adaptive_infill_interior_density = if adaptive_active {
        (region.adaptive_infill_interior_density.0 as f32).max(1.0)
    } else {
        0.0
    };
    params.adaptive_infill_gradient = if adaptive_active {
        region.adaptive_infill_gradient
    } else {
        ProcessAdaptiveInfillGradient::Linear
    };
    params.gyroid_optimized = matches!(
        params.pattern,
        SurfaceFillPattern::Configured(ProcessInfillPattern::Gyroid)
    ) && region.gyroid_optimized.0
        && !adaptive_active;
}

pub(super) fn apply_pattern_fields(
    params: &mut SurfaceFillParams,
    region: &RegionOptions,
    scale: CoordinateScale,
    layer_id: usize,
) {
    if region.sparse_infill_pattern == ProcessInfillPattern::LockedZag {
        params.infill_lock_depth = (region.infill_lock_depth.0 / scale.factor()) as f32;
        params.skin_infill_depth = (region.skin_infill_depth.0 / scale.factor()) as f32;
    }
    if matches!(
        region.sparse_infill_pattern,
        ProcessInfillPattern::CrossZag | ProcessInfillPattern::LockedZag
    ) {
        params.symmetric_infill_y_axis = region.symmetric_infill_y_axis.0;
        // `Fill.cpp:1311-1316`: `horiz_move = ±scale_(infill_shift_step) *
        // (layer_id/2)` — sign flips by layer parity, magnitude grows with
        // the layer pair index.
        let shift = (region.infill_shift_step.0 / scale.factor() * (layer_id / 2) as f64) as f32;
        params.horiz_move = if layer_id.is_multiple_of(2) {
            -shift
        } else {
            shift
        };
    } else if region.sparse_infill_pattern == ProcessInfillPattern::ZigZag {
        params.symmetric_infill_y_axis = region.symmetric_infill_y_axis.0;
    }
}

pub(super) fn anchor_lengths(options: &RegionOptions, spacing: f64) -> (f32, f32) {
    let anchor_length = projected_length(options.infill_anchor, spacing);
    let anchor_length_max = projected_length(options.infill_anchor_max, spacing);
    let anchor_length = if anchor_length <= anchor_length_max {
        anchor_length
    } else {
        anchor_length_max
    };
    (anchor_length, anchor_length_max)
}

fn projected_length(value: FloatOrPercent, spacing: f64) -> f32 {
    match value {
        FloatOrPercent::Float(value) => value as f32,
        FloatOrPercent::Percent(value) => (f64::from(value.0 as f32) * 0.01 * spacing) as f32,
    }
}
