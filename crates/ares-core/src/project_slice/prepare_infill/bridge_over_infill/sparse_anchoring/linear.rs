use crate::{
    ProcessInfillPattern, SliceError,
    fill::rectilinear::{MonotonicFillParams, fill_monotonic_surface, fill_rectilinear_surface},
    geometry::{BoundingBox, Point, Polyline},
    project_slice::{fill_entities::SurfaceCall, group_fills::SurfaceFill},
};

pub(super) fn append(
    (fill, pattern): (SurfaceFill, ProcessInfillPattern),
    call: SurfaceCall,
    object_bounding_box: BoundingBox,
    result: &mut Vec<Polyline>,
) -> Result<(), SliceError> {
    // Upstream `generate_sparse_infill_polylines_for_anchoring`
    // (`Fill.cpp:1394`) creates `Fill::new_from_type(pattern)` for
    // every linear pattern — `FillRectilinear`/`FillMonotonic`/
    // `FillLine`/`FillZigZag`/`FillCrossZag`/`FillLockedZag`/
    // `FillMonotonicLines`/`FillAlignedRectilinear` all share the
    // rectilinear/monotonic core (`fill_surface` `monotonic` flag).
    let params = MonotonicFillParams {
        dont_alternate_fill_direction: call.dont_alternate_fill_direction,
        layer_angle_fixed: pattern == ProcessInfillPattern::AlignedRectilinear,
        spacing: fill.params.spacing,
        overlap: 0.0,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        angle: fill.params.angle,
        layer_index: call.layer_id,
        thickness_layers: fill.representative.thickness_layers.max(1),
        fixed_angle: fill.params.fixed_angle,
        bridge_angle: None,
        reference_point: Point::new(0, 0),
        dont_adjust: false,
        // `FillMonotonicLines` forces `anchor_length_max = 0`
        // (`FillRectilinear.cpp:3676-3682`).
        anchor_length_max: if pattern == ProcessInfillPattern::MonotonicLine {
            0.0
        } else {
            fill.params.anchor_length_max
        },
        link_max_length: 0.0,
        consistent_pattern: pattern.has_consistent_pattern(),
        horiz_move: fill.params.horiz_move,
        // `extended_object_bounding_box` (FillRectilinear.cpp:2741-2750) —
        // same transpose-merge + sqrt(2) origin-scale as the main fill
        // path, so anchored zig-zag/locked-zag keeps the object-wide
        // grid parity.
        extended_object_min_x: pattern.has_consistent_pattern().then(|| {
            crate::project_slice::fill_entities::extended_object_bounding_box_x(object_bounding_box)
                .0
        }),
        extended_object_max_x: pattern.has_consistent_pattern().then(|| {
            crate::project_slice::fill_entities::extended_object_bounding_box_x(object_bounding_box)
                .1
        }),
        locked_zag: false,
    };
    let generate = if matches!(
        pattern,
        ProcessInfillPattern::Monotonic | ProcessInfillPattern::MonotonicLine
    ) {
        fill_monotonic_surface
    } else {
        fill_rectilinear_surface
    };
    for expolygon in fill.expolygons {
        let generated = if crate::fill::rectilinear_multiline::enabled(
            pattern,
            fill.params.multiline,
            params.density,
        ) {
            crate::fill::rectilinear_multiline::fill_surface(
                &expolygon,
                params,
                fill.params.multiline,
                fill.params.anchor_length,
                call.scale,
            )
        } else {
            generate(&expolygon, params, call.scale)
        };
        result.extend(
            generated
                .map_err(super::super::transaction::geometry_error)?
                .polylines,
        );
    }
    Ok(())
}
