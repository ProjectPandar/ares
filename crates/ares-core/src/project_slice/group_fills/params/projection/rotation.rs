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
        simple_rotation_angle(template, context.planned.id).ok_or_else(|| {
            SliceError::UnsupportedProjectFeature(
                if sparse {
                    "sparse_infill_rotate_template"
                } else {
                    "solid_infill_rotate_template"
                }
                .to_owned(),
            )
        })?
    };
    let mut angle = (std::f64::consts::PI * degrees / 180.0) as f32;
    if context.region.align_infill_direction_to_model.0 {
        angle += context.model_rotation_offset;
    }
    Ok((angle, fixed))
}

pub(in crate::project_slice) fn simple_rotation_angle(
    template: &str,
    layer_id: usize,
) -> Option<f64> {
    let angles = template
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|token| !token.is_empty())
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (!angles.is_empty()).then(|| angles[layer_id % angles.len()])
}
