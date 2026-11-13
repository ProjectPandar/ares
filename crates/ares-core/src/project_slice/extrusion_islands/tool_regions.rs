//! `ToolOrdering::collect_extruders` role presence before fill flattening.

use crate::ExtrusionRole;
use crate::project_slice::{
    fill_entities::{FillExtrusionEntity, LayerFillEntities},
    perimeters::classic::{entity_collections::ExtrusionEntity, gap_extrusion::GapFillEntity},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::project_slice) struct RegionToolUsage {
    pub(in crate::project_slice) perimeters: bool,
    pub(in crate::project_slice) outer_wall_fill: bool,
    pub(in crate::project_slice) internal_solid: bool,
    pub(in crate::project_slice) top: bool,
    pub(in crate::project_slice) bottom: bool,
    pub(in crate::project_slice) infill: bool,
}

impl RegionToolUsage {
    fn record(&mut self, role: ExtrusionRole) {
        if matches!(role, ExtrusionRole::TopSolidInfill | ExtrusionRole::Ironing) {
            self.top = true;
        } else if role == ExtrusionRole::BottomSurface {
            self.bottom = true;
        } else if role.is_solid_infill() {
            self.internal_solid = true;
        } else if role != ExtrusionRole::None {
            self.infill = true;
        }
    }
}

pub(in crate::project_slice) fn wall_role(entity: &ExtrusionEntity) -> ExtrusionRole {
    let role = match entity {
        ExtrusionEntity::Loop(entity) => entity.extrusion_loop.paths[0].role,
        ExtrusionEntity::MultiPath(entity) => entity.paths[0].role,
        ExtrusionEntity::Path(entity) => entity.role,
    };
    use crate::project_slice::perimeters::classic::materialize::ExtrusionRole as WallRole;
    match role {
        WallRole::ExternalPerimeter => ExtrusionRole::ExternalPerimeter,
        WallRole::Perimeter => ExtrusionRole::Perimeter,
        WallRole::OverhangPerimeter => ExtrusionRole::OverhangPerimeter,
        WallRole::GapFill => ExtrusionRole::GapFill,
        WallRole::SolidInfill => ExtrusionRole::SolidInfill,
        WallRole::TopSolidInfill => ExtrusionRole::TopSolidInfill,
        WallRole::BottomSurface => ExtrusionRole::BottomSurface,
    }
}

pub(super) fn collect(layer: &LayerFillEntities) -> Vec<RegionToolUsage> {
    let mut regions = layer
        .perimeter_ranges
        .iter()
        .map(|range| RegionToolUsage {
            perimeters: !range.is_empty(),
            ..RegionToolUsage::default()
        })
        .collect::<Vec<_>>();
    for (region, range) in &layer.collection_ranges {
        for collection in &layer.collections[range.clone()] {
            if let Some(first) = collection.entities.first() {
                regions[*region].record(fill_role(first));
            }
        }
    }
    // `Fill.cpp:1528–1537` wraps each thin fill independently.
    // `ToolOrdering.cpp:160–163` selects the outer wall for standalone GapFill.
    for (region, range) in layer.thin_fill_ranges.iter().enumerate() {
        for entity in &layer.thin_fills[range.clone()] {
            let role = gap_role(entity);
            if role == ExtrusionRole::GapFill {
                regions[region].outer_wall_fill = true;
            } else {
                regions[region].record(role);
            }
        }
    }
    regions
}

pub(in crate::project_slice) fn fill_role(entity: &FillExtrusionEntity) -> ExtrusionRole {
    match entity {
        FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => path.role,
        FillExtrusionEntity::VariableWidth(entity) => gap_role(entity),
    }
}

pub(in crate::project_slice) fn gap_role(entity: &GapFillEntity) -> ExtrusionRole {
    use crate::project_slice::perimeters::classic::materialize::ExtrusionRole as Role;
    let role = match entity {
        GapFillEntity::Path(path) => path.role,
        GapFillEntity::Loop(paths) => paths[0].role,
    };
    match role {
        Role::ExternalPerimeter => ExtrusionRole::ExternalPerimeter,
        Role::Perimeter => ExtrusionRole::Perimeter,
        Role::OverhangPerimeter => ExtrusionRole::OverhangPerimeter,
        Role::GapFill => ExtrusionRole::GapFill,
        Role::SolidInfill => ExtrusionRole::SolidInfill,
        Role::TopSolidInfill => ExtrusionRole::TopSolidInfill,
        Role::BottomSurface => ExtrusionRole::BottomSurface,
    }
}
