//! `GCode.cpp:6398–6418`: external-perimeter and infill candidate ownership.

use super::*;
use crate::project_slice::perimeters::classic::entity_collections::{
    ExtrusionEntity, ExtrusionEntityCollection,
};

pub(super) fn collect(
    search: &mut Search<'_>,
    layer: &OrderedExtrusionLayer,
    placement: Placement,
) {
    for island in &layer.islands {
        let mut walls = island.perimeter_regions.iter();
        let mut fills = island.infill_regions.iter().zip(&island.infill_collections);
        let mut previous = None;
        let mut leaf = 0;
        for entity in &island.entities {
            if let IslandPrintEntity::Perimeter(collection) = entity {
                let region = *walls.next().expect("perimeter retains region");
                perimeter(search, collection, placement, region);
                continue;
            }
            let (&region, &collection) = fills.next().expect("fill retains source owner");
            if previous != Some(collection) {
                previous = Some(collection);
                leaf = 0;
            }
            infill(search, entity, placement, (region, collection), &mut leaf);
        }
    }
}

fn perimeter(
    search: &mut Search<'_>,
    collection: &ExtrusionEntityCollection,
    (object, copy, shift): Placement,
    region: usize,
) {
    let regions = search.traversal.objects[object]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .as_parts()
        .0
        .as_parts()
        .0
        .as_parts()
        .2;
    let physical = search.physical((regions[region].options.outer_wall_filament_id.0 - 1) as usize);
    let mut leaf = 0;
    for entity in &collection.entities {
        let paths = match entity {
            ExtrusionEntity::Loop(loop_) => loop_.extrusion_loop.paths.as_slice(),
            ExtrusionEntity::MultiPath(multi) => multi.paths.as_slice(),
            ExtrusionEntity::Path(path) => std::slice::from_ref(path),
        };
        for path in paths {
            if path.role == WallRole::ExternalPerimeter {
                search.external.collect(
                    &path.polyline,
                    shift,
                    physical,
                    (object, copy, region, collection.source_order, leaf),
                );
            }
            leaf += 1;
        }
    }
}

fn infill(
    search: &mut Search<'_>,
    entity: &IslandPrintEntity,
    (object, copy, shift): Placement,
    (region, collection): (usize, usize),
    leaf: &mut usize,
) {
    let regions = search.traversal.objects[object]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .as_parts()
        .0
        .as_parts()
        .0
        .as_parts()
        .2;
    let config = &regions[region].options;
    visit_fill(entity, &mut |polyline, role| {
        let filament = match role {
            ExtrusionRole::InternalInfill => Some(config.sparse_infill_filament_id.0),
            ExtrusionRole::SolidInfill | ExtrusionRole::TopSolidInfill => {
                Some(config.internal_solid_filament_id.0)
            }
            _ => None,
        };
        if let Some(filament) = filament {
            let physical = search.physical((filament - 1) as usize);
            search.fallback.collect(
                polyline,
                shift,
                physical,
                (object, copy, region, collection, *leaf),
            );
        }
        *leaf += 1;
    });
}
