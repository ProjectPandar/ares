//! Gyroid sparse infill (`FillGyroid.cpp:311-403`): regular, continuous
//! adaptive, and `SteppedShells`.
use crate::project_slice::perimeters::classic::materialize::Polyline3;

use crate::{
    ProcessAdaptiveInfillMode, SliceError,
    fill::{
        adaptive_infill_field::{
            AdaptiveInfillField, AdaptiveInfillFields,
            shells::{ShellFillSpec, fill_adaptive_infill_shells},
            tpms::{AdaptiveTpmsFillParams, AdaptiveTpmsPattern, fill_adaptive_tpms},
        },
        gyroid::{GyroidFillParams, fill_component, fill_surface},
    },
    geometry::{CoordinateScale, ExPolygon, JoinType, Polyline, offset_expolygon},
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

fn fill_stepped_shells(
    expolygons: &[ExPolygon],
    field: &AdaptiveInfillField,
    base_params: GyroidFillParams,
    spec: ShellFillSpec,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, SliceError> {
    let delta = ((base_params.overlap - 0.5 * base_params.spacing) / scale.factor()) as f32;
    let mut polylines = Vec::new();
    for expolygon in expolygons {
        let components =
            offset_expolygon(expolygon, delta, JoinType::Miter, 3.0).map_err(gyroid_error)?;
        for component in components {
            fill_adaptive_infill_shells(field, &component, spec, scale, |shell_density, part| {
                let mut shell_params = base_params;
                shell_params.density = shell_density;
                polylines.extend(fill_component(part, shell_params, scale)?);
                Ok(())
            })
            .map_err(gyroid_error)?;
        }
    }
    Ok(polylines)
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
    let base_params = GyroidFillParams {
        z: print_z,
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        angle: fill.params.angle,
        density,
        multiline: fill.params.multiline,
        // `params.layer_height` (Fill.cpp:1432) — the planned layer thickness,
        // NOT `flow.height` (which combined infill multiplies).
        layer_height: layer_height as f32,
        gyroid_optimized: fill.params.gyroid_optimized,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    if fill.params.adaptive_infill_mode == ProcessAdaptiveInfillMode::SteppedShells
        && let Some(field) = adaptive_field
    {
        let spec = ShellFillSpec {
            z: print_z - 0.5 * layer_height,
            spacing: fill.params.spacing,
            surface_density: density,
            interior_density,
            gradient: fill.params.adaptive_infill_gradient,
        };
        return fill_stepped_shells(&fill.expolygons, field, base_params, spec, scale);
    }
    if fill.params.adaptive_infill_mode != ProcessAdaptiveInfillMode::Disabled
        && let Some(field) = adaptive_field
    {
        let params = AdaptiveTpmsFillParams {
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
        let mut polylines = Vec::new();
        for expolygon in &fill.expolygons {
            polylines.extend(
                fill_adaptive_tpms(expolygon, AdaptiveTpmsPattern::Gyroid, params, field, scale)
                    .map_err(gyroid_error)?,
            );
        }
        return Ok(polylines);
    }
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        polylines.extend(fill_surface(expolygon, base_params, scale).map_err(gyroid_error)?);
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

fn gyroid_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("gyroid infill generation failed: {error:?}"))
}
