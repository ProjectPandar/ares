use crate::{
    ProcessInfillPattern, SliceError,
    fill::rectilinear::{MonotonicFillParams, fill_monotonic_surface, fill_rectilinear_surface},
    geometry::{CoordinateScale, Point},
    project_slice::group_fills::SurfaceFill,
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

#[cfg(test)]
mod tests;

// Fill.cpp constructs fresh FillParams; raw group metadata remains sticky.
fn runtime_controls(pattern: ProcessInfillPattern, phase: f32, symmetric: bool) -> (i64, bool) {
    let phase = if pattern == ProcessInfillPattern::CrossZag {
        phase as i64
    } else {
        0
    };
    let symmetric = symmetric
        && matches!(
            pattern,
            ProcessInfillPattern::ZigZag | ProcessInfillPattern::CrossZag
        );
    (phase, symmetric)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    // `FillRectilinear::fill_surface` and its monotonic subclasses share
    // `fill_surface_by_lines` (`FillRectilinear.cpp:3386-3419`).
    pattern: ProcessInfillPattern,
    layer_id: usize,
    scale: CoordinateScale,
    extended_object_bounds_x: (i64, i64),
) -> Result<(), SliceError> {
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
        dont_adjust: fill.params.bridge,
        anchor_length_max,
        link_max_length: 0.0,
        consistent_pattern: pattern.has_consistent_pattern(),
        // `extended_object_bounding_box` x-bounds — only meaningful when
        // `consistent_pattern` (`FillRectilinear.cpp:2785` gates the swap).
        extended_object_min_x: pattern
            .has_consistent_pattern()
            .then_some(extended_object_bounds_x.0),
        extended_object_max_x: pattern
            .has_consistent_pattern()
            .then_some(extended_object_bounds_x.1),
        // `params.horiz_move` is already in scaled coord units (`f32`) —
        // truncating to `i64` matches the `coord_t`/`float` mix upstream.
        horiz_move,
    };
    let no_overlap_expolygons = fill.no_overlap_expolygons.clone();
    let fill_params = fill.params;
    let fill_kind = fill.representative.kind;
    // `Fill.cpp:1329-1331`: `symmetric_infill_y_axis` mirrors the surface X
    // about `extended_object_bounding_box().center().x()` BEFORE
    // `_fill_surface_single`; `no_overlap_expolygons` keeps the un-mirrored
    // surface (`Fill.cpp:1328` intersects before the mirror). The output
    // polylines are mirrored back by `FillRectilinear.cpp:2905`.
    let symmetric_y_axis = if symmetric {
        Some((extended_object_bounds_x.0 + extended_object_bounds_x.1) / 2)
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
    for expolygon in fill.expolygons {
        let mut fill_expolygon = expolygon.clone();
        if let Some(axis) = symmetric_y_axis {
            fill_expolygon.symmetric_y(axis);
        }
        let mut generated = generate(&fill_expolygon, params, scale).map_err(geometry_error)?;
        if let Some(axis) = symmetric_y_axis {
            for polyline in &mut generated.polylines {
                for point in polyline.points_mut() {
                    *point = point.symmetric_y(axis);
                }
            }
        }
        if generated.polylines.is_empty() {
            continue;
        }
        let flow = super::materialized_flow(fill.params, generated.spacing);
        let mut entities: Vec<FillExtrusionEntity> = generated
            .polylines
            .iter()
            .cloned()
            .map(|polyline| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline,
                    fitting: Vec::new(),
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
        if let Ok(path) = std::env::var("ARES_DUMP_GAPRES") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(file, "GAPRES kind={:?} n={}", fill_kind, entities.len());
                for entity in &entities {
                    if let FillExtrusionEntity::Path(path) = entity {
                        let _ = write!(file, "P {}:", path.polyline.points().len());
                        for point in path.polyline.points() {
                            let _ = write!(
                                file,
                                " ({:.6},{:.6})",
                                scale.unscale(point.x()),
                                scale.unscale(point.y())
                            );
                        }
                        let _ = writeln!(file);
                    }
                }
            }
        }
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
