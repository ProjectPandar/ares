use super::{DraftShield, SkirtOptions, SkirtType, min_length};

pub(super) fn merge_bounds(
    layer_bounds: min_length::Bounds,
    brim_bounds: Option<min_length::Bounds>,
    options: SkirtOptions,
) -> min_length::Bounds {
    if let (true, Some(brim_bounds)) = (uses_brim_envelope(options), brim_bounds) {
        return min_length::Bounds {
            min_x: layer_bounds.min_x.min(brim_bounds.min_x),
            min_y: layer_bounds.min_y.min(brim_bounds.min_y),
            max_x: layer_bounds.max_x.max(brim_bounds.max_x),
            max_y: layer_bounds.max_y.max(brim_bounds.max_y),
        };
    }
    layer_bounds
}

const fn uses_brim_envelope(options: SkirtOptions) -> bool {
    matches!(options.skirt_type(), SkirtType::Combined)
        && matches!(options.draft_shield(), DraftShield::Disabled)
}
