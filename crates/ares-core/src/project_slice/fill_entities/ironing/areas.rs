//! `Layer::make_ironing` area/enabled/angle/nozzle helpers (Fill.cpp:1573-1705).

use crate::{
    ProcessIroningType, RegionOptions, SliceError,
    geometry::{
        ExPolygon, JoinType, intersection_ex, offset_expolygons, union_safety_offset_expolygons,
    },
    project_slice::region_slices::RegionSurfaceKind,
};

use super::{Input, SpanContext, geometry_error};

pub(super) fn enabled(
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

pub(super) fn collect(
    input: &Input<'_>,
    polygons: &mut Vec<ExPolygon>,
    infills: &mut Vec<ExPolygon>,
) {
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
    if iron_completely {
        polygons.extend(
            input
                .record
                .slices
                .iter()
                .map(|surface| surface.as_parts().1.clone()),
        );
    } else {
        polygons.extend(input.record.slices.iter().filter_map(|surface| {
            let (kind, expolygon, ..) = surface.as_parts();
            let selected = kind == RegionSurfaceKind::Top
                && (input.region.top_shell_layers.0 > 0 || input.spiral_mode)
                || iron_everything
                    && kind == RegionSurfaceKind::Bottom
                    && input.region.bottom_shell_layers.0 > 0;
            selected.then(|| expolygon.clone())
        }));
    }
    if iron_everything && !iron_completely {
        for surface in &input.record.fill_surfaces {
            if surface.as_parts().0 == RegionSurfaceKind::InternalSolid {
                infills.push(surface.as_parts().1.clone());
            }
        }
    }
}

pub(super) fn ironing_areas(
    input: &Input<'_>,
    mut areas: Vec<ExPolygon>,
    infills: Vec<ExPolygon>,
    multiple_regions: bool,
) -> Result<Option<Vec<ExPolygon>>, SliceError> {
    let union = multiple_regions || !infills.is_empty();
    areas.extend(infills);
    if areas.is_empty() {
        return Ok(None);
    }
    if union {
        areas = union_safety_offset_expolygons(&areas).map_err(geometry_error)?;
    }

    let inset_mm = if input.region.filament_ironing_inset.0 == 0.0 {
        0.5 * selected_nozzle(input)
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

// Fill.cpp:1796-1806: the span key retains the angle before the filler narrows it.
pub(super) fn ironing_angle(region: &RegionOptions, context: &SpanContext<'_>) -> (f64, bool) {
    use std::f64::consts::PI;

    let template = &region.solid_infill_rotate_template.0;
    let top_direction = region.top_layer_direction.0;
    let top_direction_set = top_direction >= 0.0;
    let top_base_angle = if top_direction_set {
        PI * top_direction / 180.0
    } else if template.is_empty() {
        PI * region.solid_infill_direction.0 / 180.0
    } else {
        let degrees = crate::project_slice::group_fills::template_rotation_angle(
            template,
            context.layers,
            context.object,
            context.default_region,
            context.layer.id,
        );
        PI * degrees / 180.0
    };
    let base = if region.ironing_angle_fixed.0 {
        0.0
    } else {
        top_base_angle
    };
    (
        base + region.ironing_angle.0 * PI / 180.0 + context.model_rotation_offset,
        region.ironing_angle_fixed.0 || top_direction_set || !template.is_empty(),
    )
}

pub(super) fn selected_nozzle(input: &Input<'_>) -> f64 {
    let filament_id = input.region.top_surface_filament_id.0;
    let nozzles = &input.nozzles.0;
    let map = &input.filament_map.0;
    let extruder = if input.is_bbl
        && nozzles.len() > 1
        && filament_id >= 1
        && filament_id as usize <= map.len()
    {
        map[(filament_id - 1) as usize].0
    } else {
        filament_id
    };
    let index = (i64::from(extruder) - 1) as usize;
    nozzles.get(index).unwrap_or(&nozzles[0]).0
}
