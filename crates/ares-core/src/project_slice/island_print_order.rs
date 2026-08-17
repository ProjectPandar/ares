use crate::project_slice::{
    extrusion_islands::{ExtrusionIsland, IslandInfillEntity, PreparedPostExtrusionIslands},
    fill_entities::FillExtrusionCollection,
    perimeters::classic::{
        entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
    },
};

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) enum IslandPrintEntity {
    Perimeter(ExtrusionEntityCollection),
    Fill(FillExtrusionCollection),
    Thin(GapFillEntity),
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct OrderedExtrusionIsland {
    pub(in crate::project_slice) entities: Vec<IslandPrintEntity>,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct OrderedExtrusionLayer {
    pub(in crate::project_slice) islands: Vec<OrderedExtrusionIsland>,
}

pub(in crate::project_slice) struct PreparedPostIslandPrintOrder {
    pub(in crate::project_slice) predecessor: PreparedPostExtrusionIslands,
    pub(in crate::project_slice) objects: Vec<Vec<OrderedExtrusionLayer>>,
}

pub(in crate::project_slice) fn prepare(
    mut predecessor: PreparedPostExtrusionIslands,
) -> PreparedPostIslandPrintOrder {
    let infill_first = {
        let traversal = &predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        traversal
            .resolved
            .objects
            .iter()
            .map(|object| {
                object.layer_candidates[0].model_parts[0]
                    .region
                    .is_infill_first
                    .0
            })
            .collect::<Vec<_>>()
    };
    let objects = predecessor
        .objects
        .iter_mut()
        .enumerate()
        .map(|(object_index, layers)| {
            layers
                .iter_mut()
                .enumerate()
                .map(|(layer_index, layer)| OrderedExtrusionLayer {
                    islands: std::mem::take(&mut layer.islands)
                        .into_iter()
                        .map(|island| {
                            order_island(island, layer_index == 0, infill_first[object_index])
                        })
                        .collect(),
                })
                .collect()
        })
        .collect();
    PreparedPostIslandPrintOrder {
        predecessor,
        objects,
    }
}

pub(in crate::project_slice) fn order_island(
    island: ExtrusionIsland,
    first_layer: bool,
    infill_first: bool,
) -> OrderedExtrusionIsland {
    let ExtrusionIsland {
        infills,
        perimeters,
    } = island;
    let perimeters = perimeters.into_iter().map(IslandPrintEntity::Perimeter);
    let infills = infills.into_iter().map(|entity| match entity {
        IslandInfillEntity::Fill(collection) => IslandPrintEntity::Fill(collection),
        IslandInfillEntity::Thin(entity) => IslandPrintEntity::Thin(entity),
    });
    let entities = if !first_layer && infill_first {
        infills.chain(perimeters).collect()
    } else {
        perimeters.chain(infills).collect()
    };
    OrderedExtrusionIsland { entities }
}

pub(in crate::project_slice) fn internal_surfaces(
    prepared: &PreparedPostIslandPrintOrder,
    object_index: usize,
    layer_index: usize,
) -> &[crate::project_slice::region_slices::RegionSurface] {
    let external = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    external.predecessor.objects[object_index].records[layer_index]
        .as_ref()
        .map_or(&[], |record| record.fill_surfaces.as_slice())
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostIslandPrintOrder) {
    super::extrusion_islands::dispose(prepared.predecessor);
}
