use crate::{InfillOptions, options::InfillLayerRole};

use super::{elephant_foot, internal_bridge};

pub(super) struct SpacingRequest<'a> {
    pub(super) role: InfillLayerRole,
    pub(super) options: &'a InfillOptions,
    pub(super) sparse_spacing: f64,
    pub(super) bridge_override: Option<internal_bridge::BridgeInfillOverride>,
    pub(super) uses_internal_bridge_density: bool,
    pub(super) layer_index: usize,
}

pub(super) fn for_role(request: SpacingRequest<'_>) -> Option<f64> {
    let options = request.options;
    if let Some(bridge) = request.bridge_override {
        return Some(options.solid_line_width() / (bridge.density_percent / 100.0));
    }
    match request.role {
        InfillLayerRole::Sparse => Some(request.sparse_spacing),
        InfillLayerRole::BottomSurface => {
            Some(options.solid_line_width() / (options.bottom_surface_density_percent() / 100.0))
        }
        InfillLayerRole::InternalSolid if request.uses_internal_bridge_density => {
            Some(options.solid_line_width() / (options.internal_bridge_density_percent() / 100.0))
        }
        InfillLayerRole::InternalSolid => {
            elephant_foot::internal_solid_spacing(options, request.layer_index)
        }
        InfillLayerRole::TopSurface if options.top_surface_density_percent() == 0.0 => None,
        InfillLayerRole::TopSurface => {
            Some(options.solid_line_width() / (options.top_surface_density_percent() / 100.0))
        }
    }
}
