use crate::Layer;

use super::{InfillOptions, InfillPattern};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InfillLayerRole {
    Sparse,
    BottomSurface,
    InternalSolid,
    TopSurface,
}

impl InfillLayerRole {
    pub(crate) const fn is_sparse(self) -> bool {
        matches!(self, Self::Sparse)
    }

    pub(crate) const fn infill_role(self) -> crate::InfillRole {
        match self {
            Self::Sparse => crate::InfillRole::Sparse,
            Self::BottomSurface | Self::InternalSolid | Self::TopSurface => {
                crate::InfillRole::Solid
            }
        }
    }

    pub(crate) const fn pattern(self, options: &InfillOptions) -> InfillPattern {
        match self {
            Self::Sparse => options.pattern,
            Self::BottomSurface => options.bottom_surface_pattern,
            Self::InternalSolid => options.internal_solid_infill_pattern,
            Self::TopSurface => options.top_surface_pattern,
        }
    }

    pub(crate) fn rotate_template_degrees(self, options: &InfillOptions) -> &[f64] {
        match self {
            Self::Sparse => &options.sparse_infill_rotate_template_degrees,
            Self::BottomSurface | Self::InternalSolid | Self::TopSurface => {
                &options.solid_infill_rotate_template_degrees
            }
        }
    }

    pub(crate) const fn direction_degrees(self, options: &InfillOptions) -> f64 {
        match self {
            Self::Sparse => options.direction_degrees,
            Self::BottomSurface | Self::InternalSolid | Self::TopSurface => {
                options.solid_direction_degrees
            }
        }
    }
}

pub(crate) fn layer_role(
    options: &InfillOptions,
    layer_index: usize,
    layer_count: usize,
) -> InfillLayerRole {
    if let Some(role) = spiral_base_role(options, layer_index, layer_count) {
        return role;
    }
    if options.sparse_density_percent == 100.0 {
        dense_role(options, layer_index, layer_count)
    } else if options.sparse_density_percent > 0.0
        && layer_index < options.shell_layers.bottom_shell_layers()
    {
        InfillLayerRole::BottomSurface
    } else if options.sparse_density_percent > 0.0 && count_only_top_shell(options, layer_index, layer_count)
    {
        InfillLayerRole::TopSurface
    } else {
        sparse_or_extra_solid(options, layer_index)
    }
}

pub(crate) fn layer_role_for_layers(
    options: &InfillOptions,
    layers: &[Layer],
    layer_index: usize,
) -> InfillLayerRole {
    if let Some(role) = spiral_base_role(options, layer_index, layers.len()) {
        return role;
    }
    if options.sparse_density_percent == 100.0 {
        dense_role_for_layers(options, layers, layer_index)
    } else if options.sparse_density_percent > 0.0
        && options.shell_layers.is_bottom_shell(layers, layer_index)
    {
        InfillLayerRole::BottomSurface
    } else if options.sparse_density_percent > 0.0
        && options.shell_layers.is_top_shell(layers, layer_index)
    {
        InfillLayerRole::TopSurface
    } else {
        sparse_or_extra_solid(options, layer_index)
    }
}

fn sparse_or_extra_solid(options: &InfillOptions, layer_index: usize) -> InfillLayerRole {
    if options.sparse_density_percent > 0.0
        && options.extra_solid_infills.matches_layer(layer_index)
    {
        InfillLayerRole::InternalSolid
    } else {
        InfillLayerRole::Sparse
    }
}

fn spiral_base_role(
    options: &InfillOptions,
    layer_index: usize,
    layer_count: usize,
) -> Option<InfillLayerRole> {
    let base_count = options.spiral_base_layer_count(layer_count);
    if base_count == 0 {
        return None;
    }
    if layer_index >= base_count {
        return Some(InfillLayerRole::Sparse);
    }
    Some(if base_count > 1 && layer_index + 1 == base_count {
        InfillLayerRole::TopSurface
    } else {
        InfillLayerRole::BottomSurface
    })
}

const fn dense_role(
    options: &InfillOptions,
    layer_index: usize,
    layer_count: usize,
) -> InfillLayerRole {
    if layer_index < options.shell_layers.bottom_shell_layers() {
        InfillLayerRole::BottomSurface
    } else if count_only_top_shell(options, layer_index, layer_count) {
        InfillLayerRole::TopSurface
    } else {
        InfillLayerRole::InternalSolid
    }
}

fn dense_role_for_layers(
    options: &InfillOptions,
    layers: &[Layer],
    layer_index: usize,
) -> InfillLayerRole {
    if options.shell_layers.is_bottom_shell(layers, layer_index) {
        InfillLayerRole::BottomSurface
    } else if options.shell_layers.is_top_shell(layers, layer_index) {
        InfillLayerRole::TopSurface
    } else {
        InfillLayerRole::InternalSolid
    }
}

const fn count_only_top_shell(
    options: &InfillOptions,
    layer_index: usize,
    layer_count: usize,
) -> bool {
    options.shell_layers.top_shell_layers() > 0
        && layer_index >= layer_count.saturating_sub(options.shell_layers.top_shell_layers())
}
