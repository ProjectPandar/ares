use crate::{InfillOptions, InfillRole, LayerContours, LayerInfills, options::InfillLayerRole};

pub(super) fn sparse_spacing(options: &InfillOptions) -> f64 {
    if options.sparse_density_percent() > 0.0 {
        options.line_width() / (options.sparse_density_percent() / 100.0)
    } else {
        options.line_width()
    }
}

pub(super) fn empty_if_zero_density_sparse(
    layer: &LayerContours,
    options: &InfillOptions,
    role: InfillLayerRole,
) -> Option<LayerInfills> {
    (options.sparse_density_percent() == 0.0 && role.is_sparse())
        .then(|| LayerInfills::new(layer.layer_id(), layer.print_z(), Vec::new()))
}

pub(super) fn path_role(
    role: InfillLayerRole,
    options: &InfillOptions,
    uses_internal_bridge_density: bool,
) -> InfillRole {
    if uses_internal_bridge_density {
        InfillRole::InternalBridge
    } else if options.sparse_density_percent() == 0.0 {
        match role {
            InfillLayerRole::BottomSurface => InfillRole::BottomSurface,
            InfillLayerRole::TopSurface => InfillRole::TopSurface,
            _ => role.infill_role(),
        }
    } else {
        role.infill_role()
    }
}
