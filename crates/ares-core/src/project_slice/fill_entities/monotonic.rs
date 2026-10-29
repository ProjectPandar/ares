use crate::project_slice::perimeters::classic::materialize::Polyline3;
use crate::{
    ProcessInfillPattern, SliceError,
    fill::rectilinear::{MonotonicFillParams, fill_monotonic_surface, fill_rectilinear_surface},
    geometry::Point,
    project_slice::group_fills::SurfaceFill,
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

#[cfg(test)]
mod tests;

// Fill.cpp constructs fresh FillParams; raw group metadata remains sticky.
fn runtime_controls(pattern: ProcessInfillPattern, phase: f32, symmetric: bool) -> (f32, bool) {
    let phase = if pattern == ProcessInfillPattern::CrossZag {
        phase
    } else {
        0.0
    };
    let symmetric = symmetric
        && matches!(
            pattern,
            ProcessInfillPattern::ZigZag | ProcessInfillPattern::CrossZag
        );
    (phase, symmetric)
}

/// Geometry entry shared with the support-ironing surface dispatch.
/// `link_max_length` is the caller-set `Fill::link_max_length`
/// (`SupportCommon.cpp:1886` sets `3 × spacing` for support ironing; sparse
/// callers leave the `FillParams` default 0).
pub(super) fn expolygon_polylines(
    fill: &SurfaceFill,
    // `FillRectilinear::fill_surface` and its monotonic subclasses share
    // `fill_surface_by_lines` (`FillRectilinear.cpp:3386-3419`).
    pattern: ProcessInfillPattern,
    object_geometry: &super::types::ObjectFillGeometry,
    call: &super::types::SurfaceCall,
) -> Result<Vec<crate::fill::rectilinear::MonotonicFillOutput>, SliceError> {
    let super::types::ObjectFillGeometry {
        scale,
        extended_object_min_x,
        extended_object_max_x,
    } = *object_geometry;
    let layer_id = call.layer_id;
    let (horiz_move, symmetric) = runtime_controls(
        pattern,
        fill.params.horiz_move,
        fill.params.symmetric_infill_y_axis,
    );
    let density = (0.01_f64 * f64::from(fill.params.density)) as f32;
    let anchor_length_max = if pattern == ProcessInfillPattern::MonotonicLine {
        0.0
    } else {
        fill.params.anchor_length_max
    };
    let params = MonotonicFillParams {
        dont_alternate_fill_direction: call.dont_alternate_fill_direction,
        layer_angle_fixed: pattern == ProcessInfillPattern::AlignedRectilinear,
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        density,
        angle: fill.params.angle,
        layer_index: layer_id,
        thickness_layers: fill.representative.thickness_layers.max(1),
        fixed_angle: fill.params.fixed_angle,
        bridge_angle: fill.params.bridge.then_some(fill.params.bridge_angle),
        // PrintObject::bounding_box() is symmetric about the centered object origin.
        reference_point: Point::new(0, 0),
        dont_adjust: call.dont_adjust,
        anchor_length_max,
        link_max_length: call.link_max_length,
        consistent_pattern: pattern.has_consistent_pattern(),
        // `extended_object_bounding_box` x-bounds — only meaningful when
        // `consistent_pattern` (`FillRectilinear.cpp:2785` gates the swap).
        extended_object_min_x: pattern
            .has_consistent_pattern()
            .then_some(extended_object_min_x),
        extended_object_max_x: pattern
            .has_consistent_pattern()
            .then_some(extended_object_max_x),
        // Keep the upstream f32 shift in scaled coordinate units.
        horiz_move,
        locked_zag: false,
    };
    // `Fill.cpp:1329-1331`: `symmetric_infill_y_axis` mirrors the surface X
    // about `extended_object_bounding_box().center().x()` BEFORE
    // `_fill_surface_single`; `no_overlap_expolygons` keeps the un-mirrored
    // surface (`Fill.cpp:1328` intersects before the mirror). The output
    // polylines are mirrored back by `FillRectilinear.cpp:2905`.
    let symmetric_y_axis = if symmetric {
        Some((extended_object_min_x + extended_object_max_x) / 2)
    } else {
        None
    };
    let generate = if matches!(
        pattern,
        ProcessInfillPattern::Monotonic | ProcessInfillPattern::MonotonicLine
    ) {
        fill_monotonic_surface
    } else {
        fill_rectilinear_surface
    };
    let mut surfaces = Vec::with_capacity(fill.expolygons.len());
    for expolygon in &fill.expolygons {
        let mut fill_expolygon = expolygon.clone();
        if let Some(axis) = symmetric_y_axis {
            fill_expolygon.symmetric_y(axis);
        }
        let mut generated =
            if crate::fill::rectilinear_multiline::enabled(pattern, fill.params.multiline, density)
            {
                crate::fill::rectilinear_multiline::fill_surface(
                    &fill_expolygon,
                    params,
                    fill.params.multiline,
                    fill.params.anchor_length,
                    scale,
                )
            } else {
                generate(&fill_expolygon, params, scale)
            }
            .map_err(geometry_error)?;
        if let Some(axis) = symmetric_y_axis {
            for point in generated
                .polylines
                .iter_mut()
                .flat_map(|polyline| polyline.points_mut())
            {
                *point = point.symmetric_y(axis);
            }
        }
        surfaces.push(generated);
    }
    Ok(surfaces)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    // `FillRectilinear::fill_surface` and its monotonic subclasses share
    // `fill_surface_by_lines` (`FillRectilinear.cpp:3386-3419`).
    pattern: ProcessInfillPattern,
    call: super::types::SurfaceCall,
    super::types::ObjectFillGeometry {
        scale,
        extended_object_min_x,
        extended_object_max_x,
    }: super::types::ObjectFillGeometry,
) -> Result<(), SliceError> {
    let no_overlap_expolygons = fill.no_overlap_expolygons.clone();
    let fill_params = fill.params;
    let fill_kind = fill.representative.kind;
    let surfaces = expolygon_polylines(
        &fill,
        pattern,
        &super::types::ObjectFillGeometry {
            scale,
            extended_object_min_x,
            extended_object_max_x,
        },
        &call,
    )?;
    for (expolygon, generated) in fill.expolygons.into_iter().zip(surfaces) {
        if generated.polylines.is_empty() {
            continue;
        }
        let flow = super::materialized_flow(fill_params, generated.spacing)?;
        let mut entities: Vec<FillExtrusionEntity> = generated
            .polylines
            .iter()
            .cloned()
            .map(|polyline| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline: Polyline3::from_xy(polyline, Vec::new()),
                    z_contoured: false,
                    role: fill.params.extrusion_role,
                    mm3_per_mm: flow.mm3_per_mm,
                    width: flow.width,
                    height: flow.height,
                })
            })
            .collect();
        super::gap_residual::append_residual(super::gap_residual::ResidualInput {
            output_entities: &mut entities,
            no_overlap_expolygons: &no_overlap_expolygons,
            params: fill_params,
            kind: fill_kind,
            expolygon: &expolygon,
            scale,
        })?;
        output.collections.push(FillExtrusionCollection {
            entities,
            no_sort: matches!(
                pattern,
                ProcessInfillPattern::Monotonic | ProcessInfillPattern::MonotonicLine
            ),
        });
    }
    Ok(())
}
