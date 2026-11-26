//! TPMS-D sparse infill — `FillTpmsD::fill_surface` → `_fill_surface_single`
//! (`FillTpmsD.cpp:113-193`): regular analytic waves, continuous adaptive
//! Schwarz-D, and `SteppedShells`.
use crate::project_slice::perimeters::classic::materialize::Polyline3;

use crate::{
    ProcessAdaptiveInfillMode, SliceError,
    fill::{
        adaptive_infill_field::{
            AdaptiveInfillField, AdaptiveInfillFields,
            shells::{ShellFillSpec, fill_adaptive_infill_shells},
            tpms::{AdaptiveTpmsFillParams, AdaptiveTpmsPattern, fill_adaptive_tpms_component},
        },
        tpms_d::{TpmsDFillParams, fill_tpmsd_surface},
    },
    geometry::{CoordinateScale, ExPolygon, JoinType, Polyline, offset_expolygon},
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

fn fill_component(
    component: &ExPolygon,
    mode: ProcessAdaptiveInfillMode,
    adaptive_field: Option<&AdaptiveInfillField>,
    (params, adaptive_params): (TpmsDFillParams, AdaptiveTpmsFillParams),
    (scale, polylines): (CoordinateScale, &mut Vec<Polyline>),
) -> Result<(), SliceError> {
    if mode == ProcessAdaptiveInfillMode::SteppedShells
        && let Some(field) = adaptive_field
    {
        let spec = ShellFillSpec {
            z: adaptive_params.z - 0.5 * adaptive_params.layer_height,
            spacing: params.spacing,
            surface_density: adaptive_params.density,
            interior_density: adaptive_params.interior_density,
            gradient: adaptive_params.gradient,
        };
        return fill_adaptive_infill_shells(
            field,
            component,
            spec,
            scale,
            |shell_density, part| {
                let mut shell_params = params;
                shell_params.density = shell_density;
                polylines.extend(fill_tpmsd_surface(part, &shell_params, scale)?);
                Ok(())
            },
        )
        .map_err(tpmsd_error);
    }
    if mode != ProcessAdaptiveInfillMode::Disabled
        && let Some(field) = adaptive_field
    {
        polylines.extend(
            fill_adaptive_tpms_component(
                component,
                AdaptiveTpmsPattern::TpmsD,
                adaptive_params,
                field,
                scale,
            )
            .map_err(tpmsd_error)?,
        );
        return Ok(());
    }
    polylines.extend(fill_tpmsd_surface(component, &params, scale).map_err(tpmsd_error)?);
    Ok(())
}

pub(super) fn polylines(
    fill: &SurfaceFill,
    (print_z, layer_height): (f64, f64),
    adaptive_field: Option<&AdaptiveInfillField>,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, SliceError> {
    let density = (0.01_f64 * f64::from(fill.params.density)) as f32;
    let interior_density =
        (0.01_f64 * f64::from(fill.params.adaptive_infill_interior_density)) as f32;
    let params = TpmsDFillParams {
        spacing: fill.params.spacing,
        density,
        // `FillTpmsD` reads `this->angle` (the un-alternated base angle), not
        // `direction.first` — same convention as `FillGyroid`.
        angle: fill.params.angle,
        layer_z: print_z,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let adaptive_params = AdaptiveTpmsFillParams {
        z: print_z,
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        angle: fill.params.angle,
        density,
        interior_density,
        gradient: fill.params.adaptive_infill_gradient,
        multiline: fill.params.multiline,
        layer_height,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    // `Fill::fill_surface` offsets the surface by `overlap - spacing/2`
    // (`FillBase.cpp:134`) before `_fill_surface_single`; each resulting
    // component is filled independently.
    let delta = ((fill.params.overlap - 0.5 * fill.params.spacing) / scale.factor()) as f32;
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        let components =
            offset_expolygon(expolygon, delta, JoinType::Miter, 3.0).map_err(tpmsd_error)?;
        for component in &components {
            fill_component(
                component,
                fill.params.adaptive_infill_mode,
                adaptive_field,
                (params, adaptive_params),
                (scale, &mut polylines),
            )?;
        }
    }
    Ok(polylines)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    mut fill: SurfaceFill,
    (print_z, layer_height): (f64, f64),
    adaptive_fields: &AdaptiveInfillFields,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let adaptive_field = adaptive_fields.get(fill.params.adaptive_infill_mode);
    let expolygons = std::mem::take(&mut fill.expolygons);
    for expolygon in expolygons {
        fill.expolygons = vec![expolygon];
        let polylines = polylines(&fill, (print_z, layer_height), adaptive_field, scale)?;
        if polylines.is_empty() {
            continue;
        }
        let polylines = super::join_close_ends::join(
            polylines,
            fill.params.extrusion_role,
            fill.params.spacing,
            (false, true),
            scale,
        );
        let flow = super::materialized_flow(fill.params, fill.params.spacing as f32)?;
        output.collections.push(FillExtrusionCollection {
            entities: polylines
                .into_iter()
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
                .collect(),
            no_sort: false,
        });
    }
    Ok(())
}

fn tpmsd_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("tpms-d infill generation failed: {error:?}"))
}
