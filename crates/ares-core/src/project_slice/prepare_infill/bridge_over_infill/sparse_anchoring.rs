use crate::{
    FloatOrPercent, ProcessInfillPattern, RegionOptions, SliceError,
    fill::{
        cross_hatch::{CrossHatchFillParams, fill_surface as fill_cross_hatch},
        gyroid::{GyroidFillParams, fill_surface as fill_gyroid},
        multiline::{MultilineFillParams, Sweep, fill_surface as fill_multiline_surface},
        rectilinear::{MonotonicFillParams, fill_monotonic_surface},
        three_d_honeycomb::{Params as Honeycomb3dParams, fill_surface as fill_honeycomb_3d},
    },
    geometry::{Point, Polyline},
    project_slice::{
        group_fills::{SurfaceFillPattern, group_fills},
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
        region_slices::RegionSurfaceKind,
    },
};

const GRID_SWEEPS: [Sweep; 2] = [
    Sweep {
        angle: 0.0,
        shift: 0.0,
    },
    Sweep {
        angle: std::f32::consts::FRAC_PI_2,
        shift: 0.0,
    },
];
const TRIANGLE_SWEEPS: [Sweep; 3] = [
    Sweep {
        angle: 0.0,
        shift: 0.0,
    },
    Sweep {
        angle: std::f32::consts::FRAC_PI_3,
        shift: 0.0,
    },
    Sweep {
        angle: 2.0 * std::f32::consts::FRAC_PI_3,
        shift: 0.0,
    },
];

pub(in crate::project_slice) fn generate_sparse_infill_polylines_for_anchoring(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
) -> Result<Vec<Polyline>, SliceError> {
    let grouped = group_fills(prepared, object_index, layer_index)?;
    let traversal = &prepared.predecessor.predecessor;
    let traversal_object = &traversal.objects[object_index];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let (compensated, _) = prelude.as_parts();
    let (post_regions, object_slices) = compensated.as_parts();
    let (plan, _, _) = post_regions.as_parts();
    let z = plan.layers[layer_index].print_z;
    let object_reference = crate::project_slice::fill_entities::object_center(object_slices);
    let mut result = Vec::new();

    for fill in grouped.surface_fills {
        if fill.representative.kind != RegionSurfaceKind::Internal {
            continue;
        }
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
                            .map_err(super::transaction::geometry_error)?,
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
                            .map_err(super::transaction::geometry_error)?,
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
                        fill_multiline_surface(
                            &expolygon,
                            params,
                            &TRIANGLE_SWEEPS,
                            traversal.scale,
                        )
                        .map_err(super::transaction::geometry_error)?,
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
                            .map_err(super::transaction::geometry_error)?,
                    );
                }
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Gyroid) => {
                if fill.params.gyroid_optimized {
                    return Err(SliceError::UnsupportedProjectFeature(
                        "gyroid_optimized".to_owned(),
                    ));
                }
                let params = GyroidFillParams {
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
                        fill_gyroid(&expolygon, params, traversal.scale)
                            .map_err(super::transaction::geometry_error)?,
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
                            .map_err(super::transaction::geometry_error)?,
                    );
                }
            }
            SurfaceFillPattern::Configured(
                ProcessInfillPattern::Rectilinear | ProcessInfillPattern::ZigZag,
            ) => {
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
                    anchor_length_max: fill.params.anchor_length_max,
                    link_max_length: 0.0,
                };
                for expolygon in fill.expolygons {
                    result.extend(
                        fill_monotonic_surface(&expolygon, params, traversal.scale)
                            .map_err(super::transaction::geometry_error)?
                            .polylines,
                    );
                }
            }
            SurfaceFillPattern::Configured(
                ProcessInfillPattern::Monotonic
                | ProcessInfillPattern::MonotonicLine
                | ProcessInfillPattern::AlignedRectilinear
                | ProcessInfillPattern::CrossZag
                | ProcessInfillPattern::LockedZag
                | ProcessInfillPattern::Line
                | ProcessInfillPattern::TriHexagon
                | ProcessInfillPattern::AdaptiveCubic
                | ProcessInfillPattern::QuarterCubic
                | ProcessInfillPattern::SupportCubic
                | ProcessInfillPattern::Lightning
                | ProcessInfillPattern::Honeycomb
                | ProcessInfillPattern::LateralHoneycomb
                | ProcessInfillPattern::LateralLattice
                | ProcessInfillPattern::TpmsD
                | ProcessInfillPattern::TpmsFk
                | ProcessInfillPattern::Concentric
                | ProcessInfillPattern::HilbertCurve
                | ProcessInfillPattern::ArchimedeanChords
                | ProcessInfillPattern::OctagramSpiral,
            )
            | SurfaceFillPattern::ConcentricInternal => {
                unreachable!("bridge transaction admits only implemented sparse anchoring patterns")
            }
        }
    }
    Ok(result)
}

pub(super) fn projected_sparse_density(options: &RegionOptions) -> f32 {
    let density_percent = options.sparse_infill_density.0 as f32;
    (0.01_f64 * f64::from(density_percent)) as f32
}

pub(super) fn projected_anchor_lengths(options: &RegionOptions, spacing: f64) -> (f32, f32) {
    let anchor_length = projected_length(options.infill_anchor, spacing);
    let anchor_length_max = projected_length(options.infill_anchor_max, spacing);
    (anchor_length.min(anchor_length_max), anchor_length_max)
}

fn projected_length(value: FloatOrPercent, spacing: f64) -> f32 {
    match value {
        FloatOrPercent::Float(value) => value as f32,
        FloatOrPercent::Percent(value) => (f64::from(value.0 as f32) * 0.01_f64 * spacing) as f32,
    }
}
