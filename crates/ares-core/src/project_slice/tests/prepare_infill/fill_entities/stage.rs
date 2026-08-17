use crate::{
    project_slice::perimeters::classic::gap_extrusion::GapFillEntity,
    project_slice::{fill_entities, tests::prepare_infill::group_fills::focused::fixture::graph},
};

#[test]
fn task22o91_stage_materializes_all_objects_and_layers_in_order() {
    let graph = graph();
    fill_entities::reset_hooks();

    let prepared = fill_entities::prepare(graph).unwrap();

    assert_eq!(fill_entities::invocations(), 1);
    assert_eq!(fill_entities::disposals(), 0);
    assert_eq!(prepared.objects.len(), 1);
    assert!(!prepared.objects[0].is_empty());
    assert!(prepared.objects[0].iter().all(|layer| {
        layer
            .collections
            .iter()
            .all(|collection| collection.paths.iter().all(|path| path.polyline.is_valid()))
    }));
    let thin_inventory = prepared.objects[0]
        .iter()
        .flat_map(|layer| &layer.thin_fills)
        .fold((0_usize, 0_usize, 0_usize), |mut counts, entity| {
            counts.0 += 1;
            match entity {
                GapFillEntity::Path(path) => {
                    counts.1 += 1;
                    counts.2 += path.polyline.points.len();
                }
                GapFillEntity::Loop(paths) => {
                    counts.1 += paths.len();
                    counts.2 += paths
                        .iter()
                        .map(|path| path.polyline.points.len())
                        .sum::<usize>();
                }
            }
            counts
        });
    assert_eq!(thin_inventory, (2_285, 2_285, 5_401));
    let perimeter_inventory = prepared.objects[0]
        .iter()
        .flat_map(|layer| &layer.perimeters)
        .fold(
            (0_usize, 0_usize, 0_usize, 0_usize),
            |mut counts, collection| {
                counts.0 += 1;
                counts.1 += collection.entities.len();
                counts.2 += collection
                    .entities
                    .iter()
                    .map(|entity| entity.extrusion_loop.paths.len())
                    .sum::<usize>();
                counts.3 += collection
                    .entities
                    .iter()
                    .flat_map(|entity| &entity.extrusion_loop.paths)
                    .map(|path| path.polyline.points.len())
                    .sum::<usize>();
                counts
            },
        );
    assert_eq!(perimeter_inventory, (2_881, 5_243, 5_483, 111_933));
    let source = &prepared.predecessor.predecessor.predecessor.predecessor;
    assert!(
        source
            .objects
            .iter()
            .all(|object| object.records.iter().all(|record| {
                record.as_ref().is_none_or(|record| {
                    record.perimeters.is_empty() && record.thin_fills.is_empty()
                })
            }))
    );
    fill_entities::dispose(prepared);
    assert_eq!(fill_entities::disposals(), 1);
}

#[test]
fn task22o91_stage_is_repeatable_for_independent_graphs() {
    let first = fill_entities::prepare(graph()).unwrap();
    let second = fill_entities::prepare(graph()).unwrap();

    assert_eq!(first.objects, second.objects);

    fill_entities::dispose(first);
    fill_entities::dispose(second);
}
