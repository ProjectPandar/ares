//! `LayerTools::extruder` and `GCode.cpp:5074–5095` mixed wall splitting.

use crate::project_slice::gcode_emit::tool_ordering::wiping::CollectionId;
use std::collections::BTreeMap;

use crate::project_slice::{
    extrusion_islands::{fill_role, gap_role, wall_role},
    island_print_order::{IslandPrintEntity, OrderedExtrusionIsland, OrderedExtrusionLayer},
    perimeters::classic::entity_collections::ExtrusionEntityCollection,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};
use crate::{ExtrusionRole, ObjectOptions, RegionOptions};

pub(super) fn split<'a>(
    layer: &mut OrderedExtrusionLayer,
    (tools, layer_override): (&[usize], Option<usize>),
    region_options: impl Fn(usize) -> &'a RegionOptions,
    override_tool: impl Fn(CollectionId) -> Option<usize>,
) -> BTreeMap<(usize, bool), OrderedExtrusionLayer> {
    let count = layer.islands.len();
    let mut buckets = BTreeMap::new();
    // `LayerTools::extruder` selects the override before membership fallback.
    let select_tool = |id: i32| {
        let tool = layer_override.unwrap_or_else(|| if id == 0 { 0 } else { (id - 1) as usize });
        if tools.contains(&tool) {
            tool
        } else {
            *tools.last().unwrap()
        }
    };
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
            let id = if matches!(entity, IslandPrintEntity::Perimeter(_)) {
                CollectionId::Perimeter(collection)
            } else {
                CollectionId::Infill(collection)
            };
            if let Some(tool) = override_tool(id) {
                append(
                    &mut buckets,
                    (tool, true),
                    (count, island_id, region, collection),
                    entity,
                );
                continue;
            }
            let options = region_options(region);
            for (tool, entity) in route(entity, options, select_tool).into_iter().flatten() {
                append(
                    &mut buckets,
                    (tool, false),
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
        [None; 2],
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
    select_tool: impl Fn(i32) -> usize,
) -> [Option<(usize, IslandPrintEntity)>; 2] {
    if let IslandPrintEntity::Perimeter(collection) = entity {
        split_walls(collection, options, select_tool).map(|entry| {
            entry.map(|(tool, collection)| (tool, IslandPrintEntity::Perimeter(collection)))
        })
    } else {
        let (role, infill, solid) = entity_kind(&entity);
        [
            Some((select_tool(filament(role, infill, solid, options)), entity)),
            None,
        ]
    }
}

fn split_walls(
    collection: ExtrusionEntityCollection,
    options: &RegionOptions,
    select_tool: impl Fn(i32) -> usize,
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
                select_tool(filament(role, false, false, options)),
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
        (!collection.entities.is_empty()).then(|| (select_tool(id), collection))
    })
}

fn append<K: Ord>(
    buckets: &mut BTreeMap<K, OrderedExtrusionLayer>,
    tool: K,
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
