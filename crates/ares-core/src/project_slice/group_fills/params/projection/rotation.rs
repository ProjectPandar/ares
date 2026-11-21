mod meta;

use crate::{ExtrusionRole, SliceError, project_slice::region_slices::RegionSurfaceKind};

use super::super::LayerContext;

pub(super) fn projected_angle(
    context: &LayerContext<'_>,
    kind: RegionSurfaceKind,
    role: ExtrusionRole,
) -> Result<(f32, bool), SliceError> {
    let sparse = role == ExtrusionRole::InternalInfill;
    // Fill.cpp:1046–1055 checks the surface type before evaluating the solid template.
    let surface_direction = match (sparse, kind) {
        (false, RegionSurfaceKind::Top) => Some(context.region.top_layer_direction.0),
        (false, RegionSurfaceKind::Bottom | RegionSurfaceKind::BottomBridge) => {
            Some(context.region.bottom_layer_direction.0)
        }
        _ => None,
    };
    let (degrees, fixed) = if let Some(degrees) = surface_direction.filter(|&angle| angle >= 0.0) {
        (degrees, true)
    } else {
        let (direction, template) = if sparse {
            (
                context.region.infill_direction.0,
                &context.region.sparse_infill_rotate_template.0,
            )
        } else {
            (
                context.region.solid_infill_direction.0,
                &context.region.solid_infill_rotate_template.0,
            )
        };
        if template.is_empty() {
            (direction, false)
        } else {
            (
                meta::template_rotation_angle(
                    template,
                    context.layers,
                    context.object,
                    context.default_region,
                    context.planned.id,
                ),
                true,
            )
        }
    };
    let mut angle = (std::f64::consts::PI * degrees / 180.0) as f32;
    if context.region.align_infill_direction_to_model.0 {
        angle += context.model_rotation_offset;
    }
    Ok((angle, fixed))
}

pub(in crate::project_slice) use meta::{reset_rand_state, template_rotation_angle};

/// Non-meta list path (`Fill.cpp:206-210`):
/// `ConfigOptionFloats::deserialize` (`Config.hpp:868-890`) splits comma
/// items with `getline`, trims each item, then reads its numeric prefix.
/// Empty input yields one zero angle. Returns the cyclic angle for `layer_id`.
pub(super) fn simple_rotation_angle(template: &str, layer_id: usize) -> f64 {
    if template.is_empty() {
        return 0.0;
    }
    let angles: Vec<f64> = template
        .split_terminator(',')
        .map(|token| meta::strtod_prefix(token.trim_ascii().as_bytes()).0)
        .collect();
    angles[layer_id % angles.len()]
}
