//! `LayerTools::extruder` and `GCode.cpp:5074–5095` mixed wall splitting.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::project_slice::{
    extrusion_islands::{fill_role, gap_role},
    island_print_order::{IslandPrintEntity, OrderedExtrusionIsland, OrderedExtrusionLayer},
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    perimeters::classic::{
        entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
        materialize::ExtrusionRole as WallRole,
    },
};
use crate::{ExtrusionRole, ObjectOptions, RegionOptions};

pub(super) fn split<'a>(
    layer: &mut OrderedExtrusionLayer,
    tools: &[usize],
    region_options: impl Fn(usize) -> &'a RegionOptions,
) -> BTreeMap<usize, OrderedExtrusionLayer> {
    let count = layer.islands.len();
    let mut buckets = BTreeMap::new();
    for (island_id, island) in layer.islands.iter_mut().enumerate() {
        let mut walls = std::mem::take(&mut island.perimeter_regions).into_iter();
        let mut fills = std::mem::take(&mut island.infill_regions)
            .into_iter()
            .zip(std::mem::take(&mut island.infill_collections));
        for entity in std::mem::take(&mut island.entities) {
            let (region, collection) = if let IslandPrintEntity::Perimeter(wall) = &entity {
                (
                    walls.next().expect("perimeter retains canonical region"),
                    wall.source_order,
                )
            } else {
                fills.next().expect("fill retains canonical owner")
            };
            let options = region_options(region);
            for (tool, entity) in route(entity, options, tools).into_iter().flatten() {
                append(
                    &mut buckets,
                    tool,
                    (count, island_id, region, collection),
                    entity,
                );
            }
        }
    }
    buckets
}

pub(super) fn raft(
    layer: &mut OrderedExtrusionLayer,
    options: &ObjectOptions,
    traversal: &PreparedPostClassicTraversal,
    tools: &[usize],
) -> BTreeMap<usize, OrderedExtrusionLayer> {
    let count = layer.islands.len();
    let mut buckets = BTreeMap::new();
    let selectors = super::support::selectors(
        layer
            .islands
            .iter()
            .flat_map(|island| &island.entities)
            .map(|entity| entity_kind(entity).0),
        options,
        traversal,
        tools,
    );
    for (index, island) in layer.islands.iter_mut().enumerate() {
        let mut regions = std::mem::take(&mut island.infill_regions)
            .into_iter()
            .zip(std::mem::take(&mut island.infill_collections));
        for entity in std::mem::take(&mut island.entities) {
            let role = entity_kind(&entity).0;
            let (region, collection) = regions.next().expect("raft retains owner");
            if let Some(&(tool, _)) = selectors.iter().find(|&&(_, selected)| {
                role == selected
                    || (selected == ExtrusionRole::Mixed && role != ExtrusionRole::Ironing)
                    || (role == ExtrusionRole::Ironing
                        && matches!(
                            selected,
                            ExtrusionRole::Mixed | ExtrusionRole::SupportMaterialInterface
                        ))
            }) {
                append(
                    &mut buckets,
                    tool,
                    (count, index, region, collection),
                    entity,
                );
            }
        }
    }
    buckets
}

fn route(
    entity: IslandPrintEntity,
    options: &RegionOptions,
    tools: &[usize],
) -> [Option<(usize, IslandPrintEntity)>; 2] {
    if let IslandPrintEntity::Perimeter(collection) = entity {
        split_walls(collection, options, tools).map(|entry| {
            entry.map(|(tool, collection)| (tool, IslandPrintEntity::Perimeter(collection)))
        })
    } else {
        let (role, infill, solid) = entity_kind(&entity);
        [
            Some((
                resolve(filament(role, infill, solid, options), tools),
                entity,
            )),
            None,
        ]
    }
}

fn split_walls(
    collection: ExtrusionEntityCollection,
    options: &RegionOptions,
    tools: &[usize],
) -> [Option<(usize, ExtrusionEntityCollection)>; 2] {
    let role =
        collection
            .entities
            .iter()
            .map(wall_role)
            .fold(ExtrusionRole::None, |current, next| {
                if current == ExtrusionRole::None || current == next {
                    next
                } else {
                    ExtrusionRole::Mixed
                }
            });
    if options.outer_wall_filament_id == options.inner_wall_filament_id
        || role != ExtrusionRole::Mixed
    {
        return [
            Some((
                resolve(filament(role, false, false, options), tools),
                collection,
            )),
            None,
        ];
    }
    let mut outer = ExtrusionEntityCollection {
        entities: Vec::new(),
        source_order: collection.source_order,
    };
    let mut inner = ExtrusionEntityCollection {
        entities: Vec::new(),
        source_order: collection.source_order,
    };
    for entity in collection.entities {
        match wall_role(&entity) {
            ExtrusionRole::ExternalPerimeter | ExtrusionRole::OverhangPerimeter => {
                outer.entities.push(entity)
            }
            ExtrusionRole::Perimeter => inner.entities.push(entity),
            _ => {}
        }
    }
    [
        (outer, options.outer_wall_filament_id.0),
        (inner, options.inner_wall_filament_id.0),
    ]
    .map(|(collection, id)| {
        (!collection.entities.is_empty()).then(|| (resolve(id, tools), collection))
    })
}

fn append(
    buckets: &mut BTreeMap<usize, OrderedExtrusionLayer>,
    tool: usize,
    position: (usize, usize, usize, usize),
    entity: IslandPrintEntity,
) {
    let (count, island, region, collection) = position;
    let target = buckets
        .entry(tool)
        .or_insert_with(|| OrderedExtrusionLayer {
            tool_regions: Vec::new(),
            islands: (0..count)
                .map(|_| OrderedExtrusionIsland {
                    entities: Vec::new(),
                    infill_regions: Vec::new(),
                    infill_collections: Vec::new(),
                    perimeter_regions: Vec::new(),
                })
                .collect(),
        });
    let target = &mut target.islands[island];
    if matches!(entity, IslandPrintEntity::Perimeter(_)) {
        target.perimeter_regions.push(region);
    } else {
        target.infill_regions.push(region);
        target.infill_collections.push(collection);
    }
    target.entities.push(entity);
}

fn resolve(id: i32, tools: &[usize]) -> usize {
    let tool = if id == 0 { 0 } else { (id - 1) as usize };
    if tools.contains(&tool) {
        tool
    } else {
        *tools.last().unwrap()
    }
}

fn filament(role: ExtrusionRole, infill: bool, solid: bool, options: &RegionOptions) -> i32 {
    if infill {
        if !solid {
            options.sparse_infill_filament_id.0
        } else if matches!(role, ExtrusionRole::TopSolidInfill | ExtrusionRole::Ironing) {
            options.top_surface_filament_id.0
        } else if role == ExtrusionRole::BottomSurface {
            options.bottom_surface_filament_id.0
        } else {
            options.internal_solid_filament_id.0
        }
    } else if role == ExtrusionRole::Perimeter {
        options.inner_wall_filament_id.0
    } else {
        options.outer_wall_filament_id.0
    }
}

fn entity_kind(entity: &IslandPrintEntity) -> (ExtrusionRole, bool, bool) {
    let role = match entity {
        IslandPrintEntity::Fill(entity) => fill_role(entity),
        IslandPrintEntity::FillCollection(collection) => {
            return collection.entities.iter().map(fill_role).fold(
                (ExtrusionRole::None, false, false),
                |(current, infill, solid), role| {
                    (
                        if current == ExtrusionRole::None || current == role {
                            role
                        } else {
                            ExtrusionRole::Mixed
                        },
                        infill || role.is_infill(),
                        solid || role.is_solid_infill(),
                    )
                },
            );
        }
        IslandPrintEntity::Thin(entity) => gap_role(entity),
        IslandPrintEntity::Perimeter(_) => unreachable!("wall dispatch uses collection roles"),
    };
    (role, role.is_infill(), role.is_solid_infill())
}

fn wall_role(entity: &ExtrusionEntity) -> ExtrusionRole {
    let role = match entity {
        ExtrusionEntity::Loop(entity) => entity.extrusion_loop.paths[0].role,
        ExtrusionEntity::MultiPath(entity) => entity.paths[0].role,
        ExtrusionEntity::Path(entity) => entity.role,
    };
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
