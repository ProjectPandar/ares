use crate::project_slice::{
    extrusion_islands::{self, ExtrusionIsland, IslandInfillEntity},
    fill_entities::{self, FillExtrusionCollection},
    island_print_order::{self, IslandPrintEntity},
    perimeters::classic::entity_collections::ExtrusionEntityCollection,
    tests::prepare_infill::group_fills::focused::fixture::graph,
};

#[test]
fn task22o95_orders_first_and_later_layer_phases_from_option() {
    let island = || ExtrusionIsland {
        infills: vec![IslandInfillEntity::Fill(FillExtrusionCollection {
            paths: Vec::new(),
            no_sort: false,
        })],
        perimeters: vec![ExtrusionEntityCollection::default()],
    };

    let first = island_print_order::order_island(island(), true, true);
    let later_wall_first = island_print_order::order_island(island(), false, false);
    let later_infill_first = island_print_order::order_island(island(), false, true);

    assert!(matches!(first.entities[0], IslandPrintEntity::Perimeter(_)));
    assert!(matches!(
        later_wall_first.entities[0],
        IslandPrintEntity::Perimeter(_)
    ));
    assert!(matches!(
        later_infill_first.entities[0],
        IslandPrintEntity::Fill(_)
    ));
}

#[test]
fn task22o95_orders_ksr_islands_without_losing_entities() {
    let filled = fill_entities::prepare(graph()).unwrap();
    let islands = extrusion_islands::prepare(filled);
    let expected = source_inventory(&islands.objects[0]);
    let prepared = island_print_order::prepare(islands);

    let inventory = prepared.objects[0].iter().fold(
        (
            0_usize, 0_usize, 0_usize, 0_usize, 0_usize, 0_usize, 0_usize, 0_usize,
        ),
        |mut counts, layer| {
            counts.0 += layer.islands.len();
            for island in &layer.islands {
                if let Some(first) = island.entities.first() {
                    counts.1 += 1;
                    counts.2 += usize::from(matches!(first, IslandPrintEntity::Perimeter(_)));
                }
                for entity in &island.entities {
                    record_inventory(entity, &mut counts);
                }
            }
            counts
        },
    );
    assert_eq!(inventory, expected);

    island_print_order::dispose(prepared);
}

fn source_inventory(
    layers: &[crate::project_slice::extrusion_islands::LayerExtrusionIslands],
) -> (usize, usize, usize, usize, usize, usize, usize, usize) {
    let mut counts = (0, 0, 0, 0, 0, 0, 0, 0);
    for layer in layers {
        counts.0 += layer.islands.len();
        for island in &layer.islands {
            record_source_inventory(island, &mut counts);
        }
    }
    counts
}

fn record_source_inventory(
    island: &ExtrusionIsland,
    counts: &mut (usize, usize, usize, usize, usize, usize, usize, usize),
) {
    counts.1 += usize::from(!island.perimeters.is_empty() || !island.infills.is_empty());
    counts.2 += usize::from(!island.perimeters.is_empty());
    counts.3 += island.perimeters.len();
    for infill in &island.infills {
        match infill {
            IslandInfillEntity::Fill(collection) => {
                counts.4 += 1;
                counts.6 += usize::from(collection.no_sort);
                counts.7 += usize::from(!collection.no_sort);
            }
            IslandInfillEntity::Thin(_) => counts.5 += 1,
        }
    }
}

fn record_inventory(
    entity: &IslandPrintEntity,
    counts: &mut (usize, usize, usize, usize, usize, usize, usize, usize),
) {
    match entity {
        IslandPrintEntity::Perimeter(_) => counts.3 += 1,
        IslandPrintEntity::Fill(collection) => {
            assert!(
                collection.paths.iter().all(|path| {
                    path.polyline.front().is_some() && path.polyline.back().is_some()
                })
            );
            counts.4 += 1;
            counts.6 += usize::from(collection.no_sort);
            counts.7 += usize::from(!collection.no_sort);
        }
        IslandPrintEntity::Thin(_) => counts.5 += 1,
    }
}

#[test]
fn task22o95_is_repeatable_for_independent_graphs() {
    let prepare = || {
        island_print_order::prepare(extrusion_islands::prepare(
            fill_entities::prepare(graph()).unwrap(),
        ))
    };
    let first = prepare();
    let second = prepare();

    assert_eq!(first.objects, second.objects);

    island_print_order::dispose(first);
    island_print_order::dispose(second);
}
