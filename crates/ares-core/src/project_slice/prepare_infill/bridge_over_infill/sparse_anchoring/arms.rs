//! `sparse_anchoring` per-pattern `match` dispatch, split from `mod.rs` for
//! the <400 LOC rule. Upstream `generate_sparse_infill_polylines_for_anchoring`
//! (`Fill.cpp:1394`) dispatches one generator per sparse infill pattern.

use crate::{
    ProcessInfillPattern, SliceError,
    fill::{
        cross_hatch::{CrossHatchFillParams, fill_surface as fill_cross_hatch},
        gyroid::{GyroidFillParams, fill_surface as fill_gyroid},
        multiline::{MultilineFillParams, Sweep, fill_surface as fill_multiline_surface},
        rectilinear::{MonotonicFillParams, fill_monotonic_surface, fill_rectilinear_surface},
        three_d_honeycomb::{Params as Honeycomb3dParams, fill_surface as fill_honeycomb_3d},
    },
    geometry::{Point, Polyline},
    project_slice::{
        group_fills::{SurfaceFillPattern, group_fills},
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
        region_slices::RegionSurfaceKind,
    },
};

use super::projected::projected_anchor_lengths;
use super::{GRID_SWEEPS, TRIANGLE_SWEEPS};

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
    result: &mut Vec<Polyline>,
) -> Result<(), SliceError> {
    let multiline_params =
        |params: crate::project_slice::group_fills::SurfaceFillParams| MultilineFillParams {
            spacing: params.spacing,
            overlap: 0.0,
            angle: params.angle + std::f32::consts::FRAC_PI_2,
            reference: object_reference,
            density: (0.01_f64 * f64::from(params.density)) as f32,
            multiline: params.multiline,
            anchor_length: params.anchor_length,
            anchor_length_max: params.anchor_length_max,
            dont_sort: false,
        };
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
                result.extend(
                    fill_multiline_surface(&expolygon, params, &GRID_SWEEPS, traversal.scale)
                        .map_err(super::super::transaction::geometry_error)?,
                );
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
                result.extend(
                    fill_multiline_surface(&expolygon, params, &TRIANGLE_SWEEPS, traversal.scale)
                        .map_err(super::super::transaction::geometry_error)?,
                );
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
                result.extend(
                    fill_multiline_surface(&expolygon, params, &sweeps, traversal.scale)
                        .map_err(super::super::transaction::geometry_error)?,
                );
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
                layer_height: fill.params.flow.height as f32,
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
            for expolygon in fill.expolygons {
                result.extend(
                    crate::fill::line::fill_line(
                        &expolygon,
                        crate::fill::line::LineFillParams {
                            spacing: fill.params.spacing,
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
                    .map_err(super::super::transaction::geometry_error)?,
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
            // Upstream `generate_sparse_infill_polylines_for_anchoring`
            // (`Fill.cpp:1394`) creates `Fill::new_from_type(pattern)` for
            // every linear pattern — `FillRectilinear`/`FillMonotonic`/
            // `FillLine`/`FillZigZag`/`FillCrossZag`/`FillLockedZag`/
            // `FillMonotonicLines`/`FillAlignedRectilinear` all share the
            // rectilinear/monotonic core (`fill_surface` `monotonic` flag).
            let params = MonotonicFillParams {
                spacing: fill.params.spacing,
                overlap: 0.0,
                density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                angle: fill.params.angle,
                layer_index,
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
                horiz_move: fill.params.horiz_move as i64,
                // Sparse-anchoring only carries the object center
                // (`dispatch`'s `object_reference`), not the full object
                // bounding box the extended grid anchor needs; left `None`
                // pending bbox plumbing (main `monotonic` path is covered).
                extended_object_min_x: None,
                extended_object_max_x: None,
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
                result.extend(
                    generate(&expolygon, params, traversal.scale)
                        .map_err(super::super::transaction::geometry_error)?
                        .polylines,
                );
            }
        }
        SurfaceFillPattern::Configured(
            pattern @ (ProcessInfillPattern::AdaptiveCubic | ProcessInfillPattern::SupportCubic),
        ) => {
            for expolygon in &fill.expolygons {
                let _ = expolygon;
            }
            let anchored = crate::project_slice::fill_entities::adaptive::anchoring_lines(
                traversal,
                object_index,
                &fill,
                z,
                traversal.scale,
            )?;
            result.extend(anchored);
            let _ = pattern;
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::Honeycomb) => {
            for expolygon in fill.expolygons {
                result.extend(
                    crate::fill::honeycomb::fill_honeycomb(
                        &expolygon,
                        crate::fill::honeycomb::HoneycombFillParams {
                            spacing: fill.params.spacing,
                            density: (0.01 * f64::from(fill.params.density)) as f32,
                            multiline: fill.params.multiline,
                            angle: fill.params.angle,
                            layer_index,
                            thickness_layers: fill.representative.thickness_layers.max(1),
                            fixed_angle: fill.params.fixed_angle,
                            bridge_angle: fill.params.bridge.then_some(fill.params.bridge_angle),
                            anchor_length: fill.params.anchor_length,
                            anchor_length_max: fill.params.anchor_length_max,
                        },
                        traversal.scale,
                    )
                    .map_err(super::super::transaction::geometry_error)?,
                );
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::Concentric) => {
            for expolygon in fill.expolygons {
                result.extend(
                    crate::fill::concentric_sparse::fill_concentric_sparse(
                        &expolygon,
                        fill.params.spacing,
                        // upstream `params.density` is a 0..1 fraction; the
                        // SurfaceFill param is a percent.
                        0.01 * fill.params.density,
                        fill.params.multiline,
                        fill.params.loop_clipping,
                        traversal.scale,
                    )
                    .map_err(super::super::transaction::geometry_error)?,
                );
            }
        }
        SurfaceFillPattern::Configured(
            pattern @ (ProcessInfillPattern::HilbertCurve
            | ProcessInfillPattern::ArchimedeanChords
            | ProcessInfillPattern::OctagramSpiral),
        ) => {
            let plane_path_pattern = match pattern {
                ProcessInfillPattern::HilbertCurve => {
                    crate::fill::plane_path::PlanePathPattern::HilbertCurve
                }
                ProcessInfillPattern::ArchimedeanChords => {
                    crate::fill::plane_path::PlanePathPattern::ArchimedeanChords
                }
                ProcessInfillPattern::OctagramSpiral => {
                    crate::fill::plane_path::PlanePathPattern::OctagramSpiral
                }
                _ => unreachable!("plane-path arm only admits its three patterns"),
            };
            let resolution = traversal.resolved.views.full.process.print.resolution.0;
            let params = fill.params;
            let fill_params = crate::fill::plane_path::PlanePathFillParams {
                spacing: params.spacing,
                overlap: params.overlap,
                density: (0.01 * f64::from(params.density)) as f32,
                angle: params.angle,
                multiline: params.multiline,
                resolution,
                anchor_length: params.anchor_length,
                anchor_length_max: params.anchor_length_max,
                object_bounding_box,
                // `calibration_pattern_order` only applies to top-surface
                // (ironing) fill — sparse internal infill never takes it.
                calibration_order: false,
            };
            for expolygon in fill.expolygons {
                let components = crate::fill::plane_path::fill_surface(
                    &expolygon,
                    plane_path_pattern,
                    fill_params,
                    traversal.scale,
                )
                .map_err(super::super::transaction::geometry_error)?;
                result.extend(components.into_iter().flatten());
            }
        }
        pattern => {
            return super::arms_tail::dispatch_tail(
                pattern,
                fill,
                traversal,
                object_index,
                layer_index,
                z,
                object_reference,
                lightning_generator,
                result,
            );
        }
    }
    Ok(())
}
