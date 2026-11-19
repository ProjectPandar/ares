//! `sparse_anchoring` per-pattern `match` dispatch, split from `mod.rs` for
//! the <400 LOC rule. Upstream `generate_sparse_infill_polylines_for_anchoring`
//! (`Fill.cpp:1394`) dispatches one generator per sparse infill pattern.

use crate::{
    ProcessInfillPattern, SliceError,
    fill::{
        cross_hatch::{CrossHatchFillParams, fill_surface as fill_cross_hatch},
        gyroid::{GyroidFillParams, fill_surface as fill_gyroid},
        multiline::{MultilineFillParams, Sweep, fill_surface as fill_multiline_surface},
        three_d_honeycomb::{Params as Honeycomb3dParams, fill_surface as fill_honeycomb_3d},
    },
    geometry::{JoinType, Point, Polyline, offset_expolygon},
    project_slice::group_fills::SurfaceFillPattern,
};

use super::{GRID_SWEEPS, TRIANGLE_SWEEPS};

fn orient_grid_anchors(mut lines: Vec<Polyline>, layer_index: usize) -> Vec<Polyline> {
    // Fill.cpp:1611 assigns the object-relative layer ID, excluding raft layers.
    // FillRectilinear.cpp:3543–3545 reverses only ordinary Grid.
    if layer_index % 2 == 1 {
        for line in &mut lines {
            line.reverse();
        }
    }
    lines
}

/// `match fill.params.pattern` dispatch — first half of the sparse-anchoring
/// pattern arms. Extracted verbatim from
/// `generate_sparse_infill_polylines_for_anchoring`.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    fill: crate::project_slice::group_fills::SurfaceFill,
    traversal: &crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal,
    object_index: usize,
    layer_index: usize,
    z: f64,
    object_reference: Point,
    object_bounding_box: crate::geometry::BoundingBox,
    lightning_generator: Option<&crate::fill::lightning::generator::Generator>,
    (grid_bounds, layer_id): (Option<crate::geometry::BoundingBox>, usize),
    object_spacing: &crate::project_slice::fill_entities::adaptive::ObjectSpacing,
    result: &mut Vec<Polyline>,
) -> Result<(), SliceError> {
    match fill.params.pattern {
        SurfaceFillPattern::Configured(ProcessInfillPattern::CrossHatch) => {
            let params = CrossHatchFillParams {
                z,
                spacing: fill.params.spacing,
                overlap: 0.0,
                angle: fill.params.angle,
                density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                multiline: fill.params.multiline,
                anchor_length: fill.params.anchor_length,
                anchor_length_max: fill.params.anchor_length_max,
                dont_sort: false,
            };
            for expolygon in fill.expolygons {
                result.extend(
                    fill_cross_hatch(&expolygon, params, traversal.scale)
                        .map_err(super::super::transaction::geometry_error)?,
                );
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::Grid) => {
            let params = MultilineFillParams {
                spacing: fill.params.spacing,
                overlap: 0.0,
                // `Fill::_infill_direction` (FillBase.cpp:329) adds π/2
                // to the frame angle once per fill call.
                angle: fill.params.angle + std::f32::consts::FRAC_PI_2,
                reference: object_reference,
                density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                multiline: fill.params.multiline,
                anchor_length: fill.params.anchor_length,
                anchor_length_max: fill.params.anchor_length_max,
                dont_sort: false,
            };
            for expolygon in fill.expolygons {
                let lines = match (params.multiline > 1, grid_bounds) {
                    (true, Some(bounds)) => crate::fill::grid_trapezoidal::fill_surface(
                        &expolygon,
                        params,
                        crate::fill::grid_trapezoidal::SurfaceGeometry {
                            layer_id,
                            thickness_layers: fill.representative.thickness_layers,
                            bounds,
                        },
                        traversal.scale,
                    ),
                    (true, None) => {
                        unreachable!("Grid anchoring retains its object box")
                    }
                    (false, _) => {
                        fill_multiline_surface(&expolygon, params, &GRID_SWEEPS, traversal.scale)
                            .map(|lines| orient_grid_anchors(lines, layer_index))
                    }
                };
                result.extend(lines.map_err(super::super::transaction::geometry_error)?);
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::Triangles) => {
            let params = MultilineFillParams {
                spacing: fill.params.spacing,
                overlap: 0.0,
                angle: fill.params.angle + std::f32::consts::FRAC_PI_2,
                reference: object_reference,
                density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                multiline: fill.params.multiline,
                anchor_length: fill.params.anchor_length,
                anchor_length_max: fill.params.anchor_length_max,
                dont_sort: false,
            };
            for expolygon in fill.expolygons {
                let lines = if params.multiline > 1 {
                    crate::fill::triangles_trapezoidal::fill_surface(
                        &expolygon,
                        params,
                        crate::fill::triangles_trapezoidal::SurfaceGeometry {
                            layer_id,
                            thickness_layers: fill.representative.thickness_layers,
                        },
                        traversal.scale,
                    )
                } else {
                    fill_multiline_surface(&expolygon, params, &TRIANGLE_SWEEPS, traversal.scale)
                };
                result.extend(lines.map_err(super::super::transaction::geometry_error)?);
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::Cubic) => {
            let shift = (std::f64::consts::FRAC_1_SQRT_2 * z) as f32;
            let sweeps = [
                Sweep { angle: 0.0, shift },
                Sweep {
                    angle: std::f32::consts::FRAC_PI_3,
                    shift: -shift,
                },
                Sweep {
                    angle: 2.0 * std::f32::consts::FRAC_PI_3,
                    shift,
                },
            ];
            let params = MultilineFillParams {
                spacing: fill.params.spacing,
                overlap: 0.0,
                // `Fill::_infill_direction` (FillBase.cpp:329) adds π/2 to
                // the frame angle once per fill call.
                angle: fill.params.angle + std::f32::consts::FRAC_PI_2,
                reference: object_reference,
                density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                multiline: fill.params.multiline,
                anchor_length: fill.params.anchor_length,
                anchor_length_max: fill.params.anchor_length_max,
                dont_sort: false,
            };
            for expolygon in fill.expolygons {
                // FillCubic::fill_surface (FillRectilinear.cpp:3603-3618).
                let lines = if params.multiline > 1 {
                    crate::fill::cubic_trapezoidal::fill_surface(
                        &expolygon,
                        params,
                        crate::fill::cubic_trapezoidal::SurfaceGeometry {
                            layer_id: layer_index,
                            thickness_layers: fill.representative.thickness_layers,
                            print_z: z,
                        },
                        traversal.scale,
                    )
                } else {
                    fill_multiline_surface(&expolygon, params, &sweeps, traversal.scale)
                };
                result.extend(lines.map_err(super::super::transaction::geometry_error)?);
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::Gyroid) => {
            let params = GyroidFillParams {
                z,
                spacing: fill.params.spacing,
                overlap: 0.0,
                angle: fill.params.angle,
                density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                multiline: fill.params.multiline,
                layer_height: fill.params.flow.height,
                gyroid_optimized: fill.params.gyroid_optimized,
                anchor_length: fill.params.anchor_length,
                anchor_length_max: fill.params.anchor_length_max,
                dont_sort: false,
            };
            for expolygon in fill.expolygons {
                result.extend(
                    fill_gyroid(&expolygon, params, traversal.scale)
                        .map_err(super::super::transaction::geometry_error)?,
                );
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::ThreeDHoneycomb) => {
            let params = Honeycomb3dParams {
                z,
                layer_height: {
                    let prepared = &traversal.objects[object_index].predecessor.predecessor;
                    let prelude = &prepared.predecessor.predecessor.object;
                    let (plan, _, _) = prelude.as_parts().0.as_parts().0.as_parts();
                    plan.layers[layer_index].height
                },
                thickness_layers: fill.representative.thickness_layers,
                complete_top: false,
                smooth_factor: fill.params.smooth_factor,
                resolution: traversal.resolved.views.full.process.print.resolution.0,
                spacing: fill.params.spacing,
                overlap: 0.0,
                angle: fill.params.angle,
                density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                multiline: fill.params.multiline,
                anchor_length: fill.params.anchor_length,
                anchor_length_max: fill.params.anchor_length_max,
                dont_sort: false,
            };
            for expolygon in fill.expolygons {
                result.extend(
                    fill_honeycomb_3d(&expolygon, params, traversal.scale)
                        .map_err(super::super::transaction::geometry_error)?,
                );
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::Line) => {
            // `FillLine` (FillLine.cpp) — oscillating vertical lines, a
            // distinct generator from the rectilinear family.
            // Fill.cpp:1444-1446: use the anchoring fill's printing region.
            let (resolved_object, _) = traversal.print_object_config(object_index);
            let region = &resolved_object.all_regions[fill.region_id];
            for expolygon in fill.expolygons {
                result.extend(
                    crate::fill::line::fill_line(
                        &expolygon,
                        crate::fill::line::LineFillParams {
                            dont_alternate_fill_direction: region.zaa_enabled.0
                                && region.zaa_dont_alternate_fill_direction.0,
                            spacing: fill.params.spacing,
                            overlap: 0.0,
                            density: (0.01 * f64::from(fill.params.density)) as f32,
                            angle: fill.params.angle,
                            layer_index,
                            thickness_layers: fill.representative.thickness_layers.max(1),
                            fixed_angle: fill.params.fixed_angle,
                            bridge_angle: fill.params.bridge.then_some(fill.params.bridge_angle),
                            reference: object_reference,
                            dont_adjust: fill.params.bridge,
                        },
                        traversal.scale,
                    )
                    .map_err(super::super::transaction::geometry_error)?
                    .polylines,
                );
            }
        }
        SurfaceFillPattern::Configured(
            pattern @ (ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::ZigZag
            | ProcessInfillPattern::Monotonic
            | ProcessInfillPattern::MonotonicLine
            | ProcessInfillPattern::AlignedRectilinear
            | ProcessInfillPattern::CrossZag
            | ProcessInfillPattern::LockedZag),
        ) => {
            let prelude = &traversal.objects[object_index]
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object;
            let resolved_object = traversal
                .resolved
                .objects
                .iter()
                .find(|object| object.source_object_index == prelude.identity().0)
                .expect("anchoring retains its resolved object");
            // Fill.cpp:1440-1447: use the anchoring fill's printing region.
            let region = &resolved_object.all_regions[fill.region_id];
            let call = crate::project_slice::fill_entities::SurfaceCall {
                dont_alternate_fill_direction: region.zaa_enabled.0
                    && region.zaa_dont_alternate_fill_direction.0,
                layer_id: layer_index,
                reference: object_reference,
                scale: traversal.scale,
                link_max_length: 0.0,
                dont_adjust: false,
            };
            super::linear::append((fill, pattern), call, object_bounding_box, result)?;
        }
        SurfaceFillPattern::Configured(
            pattern @ (ProcessInfillPattern::AdaptiveCubic | ProcessInfillPattern::SupportCubic),
        ) => {
            for expolygon in &fill.expolygons {
                let _ = expolygon;
            }
            let prelude = &traversal.objects[object_index]
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object;
            let (plan, _, _) = prelude.as_parts().0.as_parts().0.as_parts();
            let anchored = crate::project_slice::fill_entities::adaptive::anchoring_lines(
                crate::project_slice::fill_entities::adaptive::AdaptiveObject::new(
                    traversal,
                    object_index,
                    // Fill.cpp:1619 uses the same region selection for anchoring.
                    object_spacing.for_pattern(pattern, fill.region_id),
                ),
                &fill,
                z,
                plan.layers[layer_index].id,
                traversal.scale,
            )?;
            result.extend(anchored);
            let _ = pattern;
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::Honeycomb) => {
            super::honeycomb::append(fill, traversal, object_index, layer_index, result)?;
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::Concentric) => {
            // Fill.cpp:1479,1497 / FillBase.cpp:105-117: anchoring uses the
            // zero-overlap fill_surface contraction before the pattern generator.
            let params = crate::fill::FillParams {
                density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                multiline: fill.params.multiline,
                dont_adjust: false,
            };
            let delta = (-0.5 * fill.params.spacing / traversal.scale.factor()) as f32;
            for expolygon in fill.expolygons {
                let domains = offset_expolygon(&expolygon, delta, JoinType::Miter, 3.0)
                    .map_err(super::super::transaction::geometry_error)?;
                for domain in domains {
                    result.extend(
                        crate::fill::concentric_sparse::fill_concentric_sparse(
                            &domain,
                            fill.params.spacing,
                            &params,
                            fill.params.loop_clipping,
                            traversal.scale,
                        )
                        .map_err(super::super::transaction::geometry_error)?,
                    );
                }
            }
        }
        SurfaceFillPattern::Configured(
            pattern @ (ProcessInfillPattern::HilbertCurve
            | ProcessInfillPattern::ArchimedeanChords
            | ProcessInfillPattern::OctagramSpiral),
        ) => {
            super::plane_path::append(fill, pattern, traversal, object_index, result)?;
        }
        pattern => {
            return super::arms_tail::dispatch_tail(
                pattern,
                fill,
                traversal,
                layer_index,
                z,
                object_reference,
                lightning_generator,
                layer_id,
                result,
            );
        }
    }
    Ok(())
}
