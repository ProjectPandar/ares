use crate::{InfillOptions, InfillPattern, options::InfillLayerRole};

#[derive(Clone, Copy)]
pub(super) struct InfillLayerPosition {
    pub(super) index: usize,
    pub(super) count: usize,
    pub(super) id: usize,
}

pub(super) struct InfillPasses {
    pub(super) angles_degrees: Vec<f64>,
    pub(super) normalize_segments: bool,
    pub(super) alternate_segments: bool,
    pub(super) scanline_shift_mm: f64,
}

impl InfillPasses {
    pub(super) fn new(
        role: InfillLayerRole,
        layer: InfillLayerPosition,
        options: &InfillOptions,
        fixed_angle_degrees: Option<f64>,
    ) -> Self {
        let pattern = role.pattern(options);
        let base_angle =
            base_angle_degrees(role, pattern, layer.index, options, fixed_angle_degrees);
        let mut angles_degrees = vec![base_angle];
        let is_grid = pattern.is_grid();
        if is_grid {
            angles_degrees.push((base_angle + 90.0) % 360.0);
        }
        Self {
            angles_degrees,
            normalize_segments: is_grid,
            alternate_segments: pattern.is_zigzag(),
            scanline_shift_mm: crosszag_scanline_shift(layer.id, pattern, options),
        }
    }
}

fn crosszag_scanline_shift(
    layer_id: usize,
    pattern: InfillPattern,
    options: &InfillOptions,
) -> f64 {
    if !matches!(pattern, InfillPattern::CrossZag | InfillPattern::LockedZag) {
        return 0.0;
    }
    let shift = options.infill_shift_step_mm() * (layer_id / 2) as f64;
    if layer_id.is_multiple_of(2) {
        -shift
    } else {
        shift
    }
}

fn base_angle_degrees(
    role: InfillLayerRole,
    pattern: InfillPattern,
    layer_index: usize,
    options: &InfillOptions,
    fixed_angle_degrees: Option<f64>,
) -> f64 {
    if let Some(angle) = fixed_angle_degrees {
        return angle;
    }
    if let Some(angle) = template_angle(role, layer_index, options) {
        return angle;
    }
    let angle = role.direction_degrees(options);
    if pattern.keeps_layer_angle_fixed() || layer_index.is_multiple_of(2) {
        angle
    } else {
        (angle + 90.0) % 360.0
    }
}

fn template_angle(
    role: InfillLayerRole,
    layer_index: usize,
    options: &InfillOptions,
) -> Option<f64> {
    let template = role.rotate_template_degrees(options);
    (!template.is_empty()).then(|| template[layer_index % template.len()])
}
