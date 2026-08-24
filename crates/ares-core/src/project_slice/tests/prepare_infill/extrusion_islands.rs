use crate::project_slice::{
    extrusion_islands::{self, IslandInfillEntity},
    fill_entities,
    tests::prepare_infill::group_fills::focused::fixture::graph,
};

#[test]
fn task22o94_assigns_ksr_entities_to_ordered_layer_islands() {
    let filled = fill_entities::prepare(graph()).unwrap();
    let prepared = extrusion_islands::prepare(filled);

    let inventory = prepared.objects[0].iter().fold(
        (
            0_usize, 0_usize, 0_usize, 0_usize, 0_usize, 0_usize, 0_usize, 0_usize, 0_usize,
        ),
        |mut counts, layer| {
            counts.0 += layer.islands.len();
            counts.1 += layer
                .islands
                .iter()
                .filter(|island| !island.infills.is_empty() || !island.perimeters.is_empty())
                .count();
            let fallback = layer.islands.last().unwrap();
            counts.2 +=
                usize::from(!fallback.infills.is_empty() || !fallback.perimeters.is_empty());
            for island in &layer.islands {
                counts.3 += island
                    .infills
                    .iter()
                    .filter(|infill| {
                        matches!(
                            infill,
                            IslandInfillEntity::Fill(_) | IslandInfillEntity::FillCollection(_)
                        )
                    })
                    .count();
                counts.4 += island
                    .infills
                    .iter()
                    .filter(|infill| matches!(infill, IslandInfillEntity::Thin(_)))
                    .count();
                counts.5 += island.perimeters.len();
                match (island.infills.is_empty(), island.perimeters.is_empty()) {
                    (false, true) => counts.6 += 1,
                    (true, false) => counts.7 += 1,
                    (false, false) => counts.8 += 1,
                    (true, true) => {}
                }
            }
            counts
        },
    );
    assert_eq!(
        inventory.2, 0,
        "no entities may fall into the fallback island"
    );
    assert_eq!(inventory.1, inventory.5);
    assert_eq!(inventory.6, 0);
    assert_eq!(inventory.7 + inventory.8, inventory.1);
    assert!(inventory.3 > 0 && inventory.4 > 0);

    extrusion_islands::dispose(prepared);
}

#[test]
fn task22o94_is_repeatable_for_independent_graphs() {
    let first = extrusion_islands::prepare(fill_entities::prepare(graph()).unwrap());
    let second = extrusion_islands::prepare(fill_entities::prepare(graph()).unwrap());

    assert_eq!(first.objects, second.objects);

    extrusion_islands::dispose(first);
    extrusion_islands::dispose(second);
}

#[test]
fn task22o211_layer_two_thin_fills_follow_first_point_island_assignment() {
    let filled = fill_entities::prepare(graph()).unwrap();
    let prepared = extrusion_islands::prepare(filled);

    let occupied = prepared.objects[0][2]
        .islands
        .iter()
        .filter(|island| {
            island
                .infills
                .iter()
                .any(|entity| matches!(entity, IslandInfillEntity::Thin(_)))
        })
        .count();

    assert_eq!(occupied, 1);
    extrusion_islands::dispose(prepared);
}
