use crate::{Layer, ShellLayerOptions, SliceError};

mod extra_solid;
mod internal_bridge_filter;
mod layer_role;
mod overlap;
mod parse;
pub(crate) mod patterns;
mod scalars;
mod top_surface;
#[cfg(test)]
mod test_support;

pub(crate) use internal_bridge_filter::InternalBridgeFilter;
pub(crate) use layer_role::InfillLayerRole;
pub(crate) use overlap::{InfillWallBoundaryOptions, InfillWallOverlapOptions};
pub(super) use parse::parse_infill_options;

impl super::SliceOptions {
    pub(crate) fn effective_sparse_infill_density_percent(&self) -> Result<f64, SliceError> {
        let density = self.percent("sparse_infill_density", 20.0)?;
        Ok(if self.bool_option("spiral_mode", false)? {
            0.0
        } else {
            density
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfillPattern {
    Rectilinear,
    AlignedRectilinear,
    Line,
    Grid,
    ZigZag,
    CrossZag,
    LockedZag,
    CrossHatch,
    Monotonic,
    MonotonicLine,
    Concentric,
    ConcentricInternal,
}

impl InfillPattern {
    pub const fn is_grid(self) -> bool {
        matches!(self, Self::Grid)
    }

    pub const fn is_zigzag(self) -> bool {
        matches!(self, Self::ZigZag | Self::CrossZag | Self::LockedZag)
    }

    pub const fn keeps_layer_angle_fixed(self) -> bool {
        matches!(
            self,
            Self::AlignedRectilinear | Self::Grid | Self::CrossZag | Self::LockedZag
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InfillOptions {
    pub(super) sparse_density_percent: f64,
    pub(super) direction_degrees: f64,
    pub(super) sparse_infill_rotate_template_degrees: Vec<f64>,
    pub(super) line_width: f64,
    pub(super) fill_multiline: usize,
    pub(super) solid_line_width: f64,
    pub(super) minimum_sparse_infill_area_mm2: f64,
    pub(super) pattern: InfillPattern,
    pub(super) solid_direction_degrees: f64,
    pub(super) bridge_angle_degrees: f64,
    pub(super) internal_bridge_angle_degrees: f64,
    pub(super) bridge_density_percent: f64,
    pub(super) internal_bridge_density_percent: f64,
    pub(super) internal_bridge_filter: InternalBridgeFilter,
    pub(super) top_surface_density_percent: f64,
    pub(super) min_width_top_surface_mm: f64,
    pub(super) calib_flowrate_topinfill_special_order: bool,
    pub(super) bottom_surface_density_percent: f64,
    pub(super) elephant_foot_layers_density_percent: f64,
    pub(super) elephant_foot_compensation_layers: usize,
    pub(super) solid_infill_rotate_template_degrees: Vec<f64>,
    pub(super) internal_solid_infill_pattern: InfillPattern,
    pub(super) bottom_surface_pattern: InfillPattern,
    pub(super) top_surface_pattern: InfillPattern,
    pub(super) extra_solid_infills: extra_solid::ExtraSolidInfills,
    pub(super) detect_narrow_internal_solid_infill: bool,
    pub(super) shell_layers: ShellLayerOptions,
    pub(super) spiral_mode: bool,
    pub(super) symmetric_infill_y_axis: bool,
    pub(super) infill_combination: bool,
    pub(super) infill_combination_max_layer_height_mm: f64,
    pub(super) infill_anchor_length_mm: f64,
    pub(super) infill_shift_step_mm: f64,
    pub(super) wall_overlap: InfillWallOverlapOptions,
    pub(super) wall_boundary: InfillWallBoundaryOptions,
}

impl InfillOptions {
    pub const fn sparse_density_percent(&self) -> f64 {
        self.sparse_density_percent
    }

    pub const fn direction_degrees(&self) -> f64 {
        self.direction_degrees
    }

    pub fn sparse_infill_rotate_template_degrees(&self) -> &[f64] {
        &self.sparse_infill_rotate_template_degrees
    }

    pub const fn line_width(&self) -> f64 {
        self.line_width
    }

    pub const fn fill_multiline(&self) -> usize {
        self.fill_multiline
    }

    pub(crate) const fn solid_line_width(&self) -> f64 {
        self.solid_line_width
    }

    pub const fn minimum_sparse_infill_area_mm2(&self) -> f64 {
        self.minimum_sparse_infill_area_mm2
    }

    pub const fn pattern(&self) -> InfillPattern {
        self.pattern
    }

    pub const fn solid_direction_degrees(&self) -> f64 {
        self.solid_direction_degrees
    }

    pub const fn bridge_angle_degrees(&self) -> f64 {
        self.bridge_angle_degrees
    }

    pub const fn internal_bridge_angle_degrees(&self) -> f64 {
        self.internal_bridge_angle_degrees
    }

    pub const fn bridge_density_percent(&self) -> f64 {
        self.bridge_density_percent
    }

    pub const fn internal_bridge_density_percent(&self) -> f64 {
        self.internal_bridge_density_percent
    }

    pub(crate) const fn internal_bridge_filter(&self) -> InternalBridgeFilter {
        self.internal_bridge_filter
    }

    pub const fn top_surface_density_percent(&self) -> f64 {
        self.top_surface_density_percent
    }

    pub const fn min_width_top_surface_mm(&self) -> f64 {
        self.min_width_top_surface_mm
    }

    pub(crate) const fn calib_flowrate_topinfill_special_order(&self) -> bool {
        self.calib_flowrate_topinfill_special_order
    }

    pub const fn bottom_surface_density_percent(&self) -> f64 {
        self.bottom_surface_density_percent
    }

    pub const fn elephant_foot_layers_density_percent(&self) -> f64 {
        self.elephant_foot_layers_density_percent
    }

    pub const fn elephant_foot_compensation_layers(&self) -> usize {
        self.elephant_foot_compensation_layers
    }

    pub fn solid_infill_rotate_template_degrees(&self) -> &[f64] {
        &self.solid_infill_rotate_template_degrees
    }

    pub const fn internal_solid_infill_pattern(&self) -> InfillPattern {
        self.internal_solid_infill_pattern
    }

    pub const fn bottom_surface_pattern(&self) -> InfillPattern {
        self.bottom_surface_pattern
    }

    pub const fn top_surface_pattern(&self) -> InfillPattern {
        self.top_surface_pattern
    }

    pub const fn detect_narrow_internal_solid_infill(&self) -> bool {
        self.detect_narrow_internal_solid_infill
    }

    pub const fn symmetric_infill_y_axis(&self) -> bool {
        self.symmetric_infill_y_axis
    }

    pub const fn infill_combination(&self) -> bool {
        self.infill_combination
    }

    pub const fn infill_combination_max_layer_height_mm(&self) -> f64 {
        self.infill_combination_max_layer_height_mm
    }

    pub const fn infill_anchor_length_mm(&self) -> f64 {
        self.infill_anchor_length_mm
    }

    pub(crate) const fn infill_shift_step_mm(&self) -> f64 {
        self.infill_shift_step_mm
    }

    pub const fn infill_wall_overlap_percent(&self) -> f64 {
        self.wall_overlap.infill_percent()
    }

    pub const fn top_bottom_infill_wall_overlap_percent(&self) -> f64 {
        self.wall_overlap.top_bottom_percent()
    }

    pub(crate) const fn wall_overlap(&self) -> InfillWallOverlapOptions {
        self.wall_overlap
    }

    pub(crate) const fn wall_boundary(&self) -> InfillWallBoundaryOptions {
        self.wall_boundary
    }

    pub(crate) const fn has_shell_layers(&self) -> bool {
        self.shell_layers.bottom_shell_layers() > 0 || self.shell_layers.top_shell_layers() > 0
    }

    pub(crate) const fn spiral_base_layer_count(&self, layer_count: usize) -> usize {
        if self.spiral_mode && self.sparse_density_percent == 0.0 {
            let bottom_shell_layers = self.shell_layers.bottom_shell_layers();
            if bottom_shell_layers < layer_count {
                bottom_shell_layers
            } else {
                layer_count
            }
        } else {
            0
        }
    }

    pub const fn effective_role(&self) -> crate::InfillRole {
        if self.sparse_density_percent == 100.0 {
            crate::InfillRole::Solid
        } else {
            crate::InfillRole::Sparse
        }
    }

    pub fn effective_pattern(&self, layer_index: usize, layer_count: usize) -> InfillPattern {
        self.layer_role(layer_index, layer_count).pattern(self)
    }

    pub const fn effective_direction_degrees(&self) -> f64 {
        if self.sparse_density_percent == 100.0 {
            self.solid_direction_degrees
        } else {
            self.direction_degrees
        }
    }

    pub fn effective_rotate_template_degrees(&self) -> &[f64] {
        if self.sparse_density_percent == 100.0 {
            &self.solid_infill_rotate_template_degrees
        } else {
            &self.sparse_infill_rotate_template_degrees
        }
    }

    pub(crate) fn layer_role(&self, layer_index: usize, layer_count: usize) -> InfillLayerRole {
        layer_role::layer_role(self, layer_index, layer_count)
    }

    pub(crate) fn layer_role_for_layers(
        &self,
        layers: &[Layer],
        layer_index: usize,
    ) -> InfillLayerRole {
        layer_role::layer_role_for_layers(self, layers, layer_index)
    }
}
