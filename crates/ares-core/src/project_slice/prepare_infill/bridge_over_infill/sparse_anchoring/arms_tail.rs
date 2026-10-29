//! `sparse_anchoring` per-pattern `match` dispatch, split from `mod.rs` for
//! the <400 LOC rule. Upstream `generate_sparse_infill_polylines_for_anchoring`
//! (`Fill.cpp:1394`) dispatches one generator per sparse infill pattern.

use crate::{
    ProcessInfillPattern, SliceError,
    fill::multiline::{MultilineFillParams, fill_surface as fill_multiline_surface},
    geometry::{JoinType, Point, Polyline, offset_expolygon},
    project_slice::group_fills::SurfaceFillPattern,
};

mod lightning;

#[cfg(test)]
mod tests;

/// `match fill.params.pattern` dispatch — tail arms (TriHexagon onward).
/// Extracted verbatim from `generate_sparse_infill_polylines_for_anchoring`.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_tail(
    pattern: SurfaceFillPattern,
    fill: crate::project_slice::group_fills::SurfaceFill,
    traversal: &crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal,
    layer_index: usize,
    z: f64,
    object_reference: Point,
    lightning_generator: Option<&crate::fill::lightning::generator::Generator>,
    stars_bounds: Option<(crate::geometry::BoundingBox, usize)>,
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
    match pattern {
        SurfaceFillPattern::Configured(ProcessInfillPattern::TriHexagon) => {
            // `ipStars` — 3×60° with the third family shifted by
            // (3/2)·spacing·multiline/density (FillRectilinear.cpp:3487).
            let sweeps = crate::fill::multiline::stars_sweeps(
                fill.params.spacing,
                fill.params.multiline,
                (0.01_f64 * f64::from(fill.params.density)) as f32,
            );
            let geometry = match (fill.params.multiline > 1, stars_bounds) {
                (true, Some(bounds)) => Some(bounds),
                (true, None) => unreachable!("Stars anchoring retains its PrintObject bounds"),
                (false, _) => None,
            };
            for expolygon in fill.expolygons {
                let lines = if let Some((bounds, layer_id)) = geometry {
                    crate::fill::stars_trapezoidal::fill_surface(
                        &expolygon,
                        multiline_params(fill.params),
                        crate::fill::stars_trapezoidal::SurfaceGeometry {
                            layer_id,
                            thickness_layers: fill.representative.thickness_layers,
                            bounds,
                        },
                        traversal.scale,
                    )
                } else {
                    fill_multiline_surface(
                        &expolygon,
                        multiline_params(fill.params),
                        &sweeps,
                        traversal.scale,
                    )
                };
                result.extend(lines.map_err(super::super::transaction::geometry_error)?);
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::QuarterCubic) => {
            let sweeps = crate::fill::multiline::quartercubic_sweeps(
                fill.params.spacing,
                fill.params.multiline,
                (0.01_f64 * f64::from(fill.params.density)) as f32,
                z,
                traversal.scale,
            );
            for expolygon in fill.expolygons {
                result.extend(
                    fill_multiline_surface(
                        &expolygon,
                        multiline_params(fill.params),
                        &sweeps,
                        traversal.scale,
                    )
                    .map_err(super::super::transaction::geometry_error)?,
                );
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::LateralLattice) => {
            let sweeps = crate::fill::multiline::lateral_lattice_sweeps(
                fill.params.lateral_lattice_angle_1,
                fill.params.lateral_lattice_angle_2,
                z,
            );
            for expolygon in fill.expolygons {
                result.extend(
                    fill_multiline_surface(
                        &expolygon,
                        multiline_params(fill.params),
                        &sweeps,
                        traversal.scale,
                    )
                    .map_err(super::super::transaction::geometry_error)?,
                );
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::LateralHoneycomb) => {
            let (sweeps, density) = crate::fill::multiline::lateral_honeycomb_sweeps(
                fill.params.spacing,
                fill.params.multiline,
                (0.01_f64 * f64::from(fill.params.density)) as f32,
                fill.params.infill_overhang_angle,
                z,
            );
            let mut params = multiline_params(fill.params);
            params.density = density;
            for expolygon in fill.expolygons {
                result.extend(
                    fill_multiline_surface(&expolygon, params, &sweeps, traversal.scale)
                        .map_err(super::super::transaction::geometry_error)?,
                );
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::TpmsD) => {
            let params = crate::fill::tpms_d::TpmsDFillParams {
                spacing: fill.params.spacing,
                density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                angle: fill.params.angle,
                layer_z: z,
                multiline: fill.params.multiline,
                anchor_length: fill.params.anchor_length,
                anchor_length_max: fill.params.anchor_length_max,
                dont_sort: false,
            };
            // FillBase.cpp:105-118 contracts each zero-overlap anchor surface.
            let delta = (-0.5 * params.spacing / traversal.scale.factor()) as f32;
            for expolygon in fill.expolygons {
                for domain in offset_expolygon(&expolygon, delta, JoinType::Miter, 3.0)
                    .map_err(super::super::transaction::geometry_error)?
                {
                    result.extend(
                        crate::fill::tpms_d::fill_tpmsd_surface(&domain, &params, traversal.scale)
                            .map_err(super::super::transaction::geometry_error)?,
                    );
                }
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::TpmsFk) => {
            let params = crate::fill::tpms_fk::TpmsFKFillParams {
                spacing: fill.params.spacing,
                density: (0.01_f64 * f64::from(fill.params.density)) as f32,
                angle: fill.params.angle,
                layer_z: z,
                multiline: fill.params.multiline,
                anchor_length: fill.params.anchor_length,
                anchor_length_max: fill.params.anchor_length_max,
                dont_sort: false,
            };
            // Fill::fill_surface offsets the zero-overlap anchor domain
            // before FillTpmsFK::_fill_surface_single, as emission does.
            let delta = (-0.5 * params.spacing / traversal.scale.factor()) as f32;
            for expolygon in fill.expolygons {
                for domain in offset_expolygon(&expolygon, delta, JoinType::Miter, 3.0)
                    .map_err(super::super::transaction::geometry_error)?
                {
                    result.extend(
                        crate::fill::tpms_fk::fill_tpmsfk_surface(
                            &domain,
                            &params,
                            traversal.scale,
                        )
                        .map_err(super::super::transaction::geometry_error)?,
                    );
                }
            }
        }
        SurfaceFillPattern::Configured(ProcessInfillPattern::Lightning) => {
            // `FillLightning::Filler::_fill_surface_single` — the anchor
            // lines are the tree's convertToLines clipped to the surface
            // (`FillLightning.cpp:17`), same as the emit path.
            let Some(generator) = lightning_generator else {
                return Ok(());
            };
            lightning::append(&fill, generator, layer_index, traversal.scale, result)?;
        }
        SurfaceFillPattern::ConcentricInternal => {
            unreachable!("bridge transaction admits only implemented sparse anchoring patterns")
        }
        _ => unreachable!("sparse anchoring dispatch_tail received a head pattern"),
    }
    Ok(())
}
