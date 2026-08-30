// Source boundary: OrcaSlicer 2.4.2 `Fill/Fill.cpp::Layer::make_ironing`.

use crate::{
    ExtrusionRole, ObjectOptions, OrcaFloats, ProcessInfillPattern, ProcessIroningType,
    RegionOptions, SliceError,
    fill::rectilinear::{MonotonicFillParams, fill_monotonic_surface},
    geometry::{
        CoordinateScale, ExPolygon, JoinType, Point, intersection_ex, offset_expolygons,
        union_safety_offset_expolygons,
    },
    project_slice::{
        layers::PlannedLayer,
        prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord,
        region_slices::RegionSurfaceKind,
    },
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

const ROUNDED_RECTANGLE_FACTOR: f64 = 1.0 - 0.25 * std::f64::consts::PI;

pub(super) struct Input<'a> {
    pub(super) record: &'a PreparedSurfaceTypeRecord,
    pub(super) layer_slices: &'a [ExPolygon],
    pub(super) region: &'a RegionOptions,
    pub(super) object: &'a ObjectOptions,
    pub(super) nozzles: &'a OrcaFloats,
    pub(super) layer: &'a PlannedLayer,
    pub(super) last_layer: bool,
    pub(super) spiral_mode: bool,
    pub(super) model_rotation_offset: f32,
    pub(super) scale: CoordinateScale,
}

pub(super) fn append(output: &mut LayerFillEntities, input: Input<'_>) -> Result<(), SliceError> {
    if !enabled(
        input.region.ironing_type,
        input.region,
        input.last_layer,
        input.spiral_mode,
    ) {
        return Ok(());
    }
    let Some(areas) = ironing_areas(&input)? else {
        return Ok(());
    };
    if input.region.ironing_pattern != ProcessInfillPattern::Rectilinear {
        return Err(SliceError::UnsupportedProjectFeature(
            "ironing_pattern".to_owned(),
        ));
    }
    let spacing = input.region.filament_ironing_spacing.0;
    if spacing <= 0.0 {
        return Ok(());
    }
    let nozzle = selected_nozzle(input.region, input.nozzles)?;
    let (angle, fixed_angle) = ironing_angle(&input);
    let extrusion_height =
        input.object.layer_height.0 * 0.01 * input.region.filament_ironing_flow.0 * spacing
            / nozzle;
    let height = extrusion_height as f32;
    let width = (f64::from(nozzle as f32) + f64::from(height) * ROUNDED_RECTANGLE_FACTOR) as f32;
    let mm3_per_mm = nozzle * extrusion_height;

    for area in areas {
        let generated = fill_monotonic_surface(
            &area,
            MonotonicFillParams {
                spacing,
                overlap: 0.0,
                density: 1.0,
                angle,
                layer_index: input.layer.id,
                thickness_layers: 1,
                fixed_angle,
                bridge_angle: None,
                reference_point: Point::new(0, 0),
                dont_adjust: true,
                anchor_length_max: 1000.0,
                link_max_length: 3.0 * spacing,
            },
            input.scale,
        )
        .map_err(geometry_error)?;
        if generated.polylines.is_empty() {
            continue;
        }
        output.collections.push(FillExtrusionCollection {
            entities: generated
                .polylines
                .into_iter()
                .map(|polyline| {
                    FillExtrusionEntity::Path(FillExtrusionPath {
                        polyline,
                        fitting: Vec::new(),
                        role: ExtrusionRole::Ironing,
                        mm3_per_mm,
                        width,
                        height,
                    })
                })
                .collect(),
            no_sort: true,
        });
    }
    Ok(())
}

fn enabled(
    ironing_type: ProcessIroningType,
    region: &RegionOptions,
    last_layer: bool,
    spiral_mode: bool,
) -> bool {
    match ironing_type {
        ProcessIroningType::NoIroning => false,
        ProcessIroningType::Solid => true,
        ProcessIroningType::Top => {
            region.top_shell_layers.0 > 0 || spiral_mode && region.bottom_shell_layers.0 > 1
        }
        ProcessIroningType::Topmost => {
            last_layer
                && (region.top_shell_layers.0 > 0
                    || spiral_mode && region.bottom_shell_layers.0 > 1)
        }
    }
}

fn ironing_areas(input: &Input<'_>) -> Result<Option<Vec<ExPolygon>>, SliceError> {
    let iron_everything = input.region.ironing_type == ProcessIroningType::Solid;
    let internal_infill_solid = input.region.sparse_infill_density.0 > 95.0;
    let iron_completely = iron_everything
        && !input.record.fill_surfaces.iter().any(|surface| {
            let kind = surface.as_parts().0;
            (!internal_infill_solid && kind == RegionSurfaceKind::Internal)
                || matches!(
                    kind,
                    RegionSurfaceKind::InternalBridge | RegionSurfaceKind::InternalVoid
                )
        });
    let mut areas = if iron_completely {
        input
            .record
            .slices
            .iter()
            .map(|surface| surface.as_parts().1.clone())
            .collect::<Vec<_>>()
    } else {
        input
            .record
            .slices
            .iter()
            .filter_map(|surface| {
                let (kind, expolygon, ..) = surface.as_parts();
                let selected = kind == RegionSurfaceKind::Top
                    && (input.region.top_shell_layers.0 > 0 || input.spiral_mode)
                    || iron_everything
                        && kind == RegionSurfaceKind::Bottom
                        && input.region.bottom_shell_layers.0 > 0;
                selected.then(|| expolygon.clone())
            })
            .collect::<Vec<_>>()
    };
    let mut added_internal_solid = false;
    if iron_everything && !iron_completely {
        for surface in &input.record.fill_surfaces {
            if surface.as_parts().0 == RegionSurfaceKind::InternalSolid {
                areas.push(surface.as_parts().1.clone());
                added_internal_solid = true;
            }
        }
    }
    if areas.is_empty() {
        return Ok(None);
    }
    if added_internal_solid {
        areas = union_safety_offset_expolygons(&areas).map_err(geometry_error)?;
    }

    let inset_mm = if input.region.filament_ironing_inset.0 == 0.0 {
        0.5 * selected_nozzle(input.region, input.nozzles)?
    } else {
        input.region.filament_ironing_inset.0
    };
    let scaled_inset = inset_mm / input.scale.factor();
    if !scaled_inset.is_finite() || !(i64::MIN as f64..-(i64::MIN as f64)).contains(&scaled_inset) {
        return Err(SliceError::InvalidInput(
            "ironing inset is out of range".to_owned(),
        ));
    }
    let inset = scaled_inset as f32;
    let inset_slices = offset_expolygons(input.layer_slices, -inset, JoinType::Miter, 3.0)
        .map_err(geometry_error)?;
    let clipped = intersection_ex(&areas, &inset_slices).map_err(geometry_error)?;
    Ok((!clipped.is_empty()).then_some(clipped))
}

fn ironing_angle(input: &Input<'_>) -> (f32, bool) {
    let template = &input.region.solid_infill_rotate_template.0;
    let template_angle = super::super::group_fills::simple_rotation_angle(template, input.layer.id);
    let base = if input.region.ironing_angle_fixed.0 {
        0.0
    } else {
        template_angle.unwrap_or(input.region.solid_infill_direction.0)
    };
    let angle =
        ((base + input.region.ironing_angle.0).to_radians() as f32) + input.model_rotation_offset;
    (
        angle,
        input.region.ironing_angle_fixed.0 || !template.is_empty(),
    )
}

fn selected_nozzle(region: &RegionOptions, nozzles: &OrcaFloats) -> Result<f64, SliceError> {
    let index = region
        .top_surface_filament_id
        .0
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok())
        .unwrap_or(0);
    nozzles
        .0
        .get(index)
        .map(|value| value.0)
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| SliceError::InvalidInput("invalid Orca option nozzle_diameter".to_owned()))
}

#[cfg(test)]
mod tests;
