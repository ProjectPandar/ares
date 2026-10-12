mod meta;

use crate::{ExtrusionRole, SliceError};

use super::super::LayerContext;

pub(super) fn projected_angle(
    context: &LayerContext<'_>,
    role: ExtrusionRole,
) -> Result<(f32, bool), SliceError> {
    let sparse = role == ExtrusionRole::InternalInfill;
    let template = if sparse {
        &context.region.sparse_infill_rotate_template.0
    } else {
        &context.region.solid_infill_rotate_template.0
    };
    let fixed = !template.is_empty();
    let degrees = if template.is_empty() {
        if sparse {
            context.region.infill_direction.0
        } else {
            context.region.solid_infill_direction.0
        }
    } else {
        meta::template_rotation_angle(
            template,
            context.layers,
            context.object,
            context.default_region,
            context.planned.id,
        )
    };
    let mut angle = (std::f64::consts::PI * degrees / 180.0) as f32;
    if context.region.align_infill_direction_to_model.0 {
        angle += context.model_rotation_offset;
    }
    Ok((angle, fixed))
}

pub(in crate::project_slice) use meta::{reset_rand_state, template_rotation_angle};

/// Non-meta list path (`Fill.cpp:206-210`): upstream
/// `ConfigOptionFloats::deserialize` tokenizes on `[\s,]+` and feeds each
/// token to `strtod`-style parsing, so unparseable tails are silently
/// dropped (`"45!"` → 45) and unparseable tokens become `0.0` — never an
/// error. Returns the cyclic angle for `layer_id`.
pub(super) fn simple_rotation_angle(template: &str, layer_id: usize) -> f64 {
    let angles: Vec<f64> = template
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|token| !token.is_empty())
        .map(|token| meta::strtod_prefix(token.as_bytes()).0)
        .collect();
    if angles.is_empty() {
        // Upstream UB domain (`layer_id % 0`); unreachable for non-empty
        // non-meta templates since every nonempty token contributes a slot.
        return 0.0;
    }
    angles[layer_id % angles.len()]
}
