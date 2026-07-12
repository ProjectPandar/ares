use crate::{PrintPathRole, ToolpathMoveKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccelerationOptions {
    pub default_mm_s2: f64,
    pub initial_layer_mm_s2: f64,
    pub outer_wall_mm_s2: f64,
    pub bridge_mm_s2: f64,
    pub inner_wall_mm_s2: f64,
    pub travel_mm_s2: f64,
    pub initial_layer_travel_mm_s2: f64,
    pub sparse_infill_mm_s2: f64,
    pub internal_solid_infill_mm_s2: f64,
    pub top_surface_mm_s2: f64,
}

impl AccelerationOptions {
    pub const fn acceleration_for_layer(
        &self,
        kind: ToolpathMoveKind,
        role: PrintPathRole,
        is_first_layer: bool,
    ) -> Option<f64> {
        if self.default_mm_s2 == 0.0 {
            return None;
        }
        match kind {
            ToolpathMoveKind::Travel if is_first_layer => {
                positive_option(self.initial_layer_travel_mm_s2)
            }
            ToolpathMoveKind::Travel => positive_option(self.travel_mm_s2),
            ToolpathMoveKind::Print if is_first_layer && self.initial_layer_mm_s2 > 0.0 => {
                Some(self.initial_layer_mm_s2)
            }
            ToolpathMoveKind::Print => match role {
                PrintPathRole::Bridge | PrintPathRole::InternalBridge => {
                    Some(positive_or_default(self.bridge_mm_s2, self.default_mm_s2))
                }
                PrintPathRole::SparseInfill => Some(positive_or_default(
                    self.sparse_infill_mm_s2,
                    self.default_mm_s2,
                )),
                PrintPathRole::SolidInfill => Some(positive_or_default(
                    self.internal_solid_infill_mm_s2,
                    self.default_mm_s2,
                )),
                PrintPathRole::TopSolidInfill => Some(positive_or_default(
                    self.top_surface_mm_s2,
                    self.default_mm_s2,
                )),
                PrintPathRole::ExternalPerimeter => Some(positive_or_default(
                    self.outer_wall_mm_s2,
                    self.default_mm_s2,
                )),
                PrintPathRole::InternalPerimeter => Some(positive_or_default(
                    self.inner_wall_mm_s2,
                    self.default_mm_s2,
                )),
                _ => Some(self.default_mm_s2),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JerkOptions {
    pub default_mm_s: f64,
    pub initial_layer_mm_s: f64,
    pub outer_wall_mm_s: f64,
    pub inner_wall_mm_s: f64,
    pub infill_mm_s: f64,
    pub top_surface_mm_s: f64,
    pub travel_mm_s: f64,
    pub initial_layer_travel_mm_s: f64,
}

impl JerkOptions {
    pub const fn jerk_for_layer(
        &self,
        kind: ToolpathMoveKind,
        role: PrintPathRole,
        is_first_layer: bool,
    ) -> Option<f64> {
        if self.default_mm_s == 0.0 {
            return None;
        }
        match kind {
            ToolpathMoveKind::Travel if is_first_layer => {
                positive_option(self.initial_layer_travel_mm_s)
            }
            ToolpathMoveKind::Travel => positive_option(self.travel_mm_s),
            ToolpathMoveKind::Print if is_first_layer && self.initial_layer_mm_s > 0.0 => {
                Some(self.initial_layer_mm_s)
            }
            ToolpathMoveKind::Print => match role {
                PrintPathRole::ExternalPerimeter => {
                    Some(positive_or_default(self.outer_wall_mm_s, self.default_mm_s))
                }
                PrintPathRole::InternalPerimeter => {
                    Some(positive_or_default(self.inner_wall_mm_s, self.default_mm_s))
                }
                PrintPathRole::SparseInfill
                | PrintPathRole::SolidInfill
                | PrintPathRole::BottomSurface
                | PrintPathRole::Bridge
                | PrintPathRole::InternalBridge => {
                    Some(positive_or_default(self.infill_mm_s, self.default_mm_s))
                }
                PrintPathRole::TopSolidInfill => Some(positive_or_default(
                    self.top_surface_mm_s,
                    self.default_mm_s,
                )),
                _ => Some(self.default_mm_s),
            },
        }
    }
}

const fn positive_option(value: f64) -> Option<f64> {
    if value > 0.0 { Some(value) } else { None }
}

const fn positive_or_default(value: f64, default: f64) -> f64 {
    if value > 0.0 { value } else { default }
}
