use crate::{PrintPathRole, SpeedOptions, ToolpathMoveKind};

pub(super) fn speed_for_layer_id(
    options: &SpeedOptions,
    kind: ToolpathMoveKind,
    role: PrintPathRole,
    layer_id: usize,
) -> f64 {
    let is_first_layer = layer_id == 0;
    let speed = options.speed_for_layer(kind, role, is_first_layer);
    if is_first_layer || kind == ToolpathMoveKind::Travel || options.slow_down_layers() <= 1 {
        return speed;
    }
    if kind == ToolpathMoveKind::Print
        && role == PrintPathRole::ExternalPerimeter
        && options.dont_slow_down_outer_wall()
    {
        return speed;
    }
    if layer_id >= options.slow_down_layers() as usize {
        return speed;
    }
    let Some(first_layer_speed) = first_layer_reference_speed_for_role(options, role) else {
        return speed;
    };
    if first_layer_speed < speed {
        first_layer_speed
            + (speed - first_layer_speed) * (layer_id as f64 / options.slow_down_layers() as f64)
    } else {
        speed
    }
}

fn first_layer_reference_speed_for_role(
    options: &SpeedOptions,
    role: PrintPathRole,
) -> Option<f64> {
    match role {
        PrintPathRole::ExternalPerimeter
        | PrintPathRole::OverhangPerimeter
        | PrintPathRole::InternalPerimeter
        | PrintPathRole::Brim => Some(options.first_layer_speed_mm_s()),
        PrintPathRole::SparseInfill
        | PrintPathRole::SolidInfill
        | PrintPathRole::TopSolidInfill
        | PrintPathRole::BottomSurface
        | PrintPathRole::SupportMaterial
        | PrintPathRole::SupportMaterialInterface
        | PrintPathRole::Ironing
        | PrintPathRole::Bridge
        | PrintPathRole::InternalBridge => Some(options.first_layer_infill_speed_mm_s()),
        PrintPathRole::GapFill | PrintPathRole::Skirt => None,
    }
}

#[cfg(test)]
mod tests;
