use crate::{
    SliceError,
    project_slice::{
        extrusion_islands::{self, IslandInfillEntity},
        fill_entities,
        tests::{
            prepare_infill::group_fills::focused::fixture::graph,
            support::{KsrArchive, metadata},
        },
    },
    slice_project,
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
                for infill in &island.infills {
                    match infill {
                        IslandInfillEntity::Fill(_) => counts.3 += 1,
                        IslandInfillEntity::Thin(_) => counts.4 += 1,
                    }
                }
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
        inventory,
        (3_350, 2_881, 0, 1_658, 2_285, 2_881, 0, 1_835, 1_046)
    );

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

#[tokio::test]
async fn task22o94_public_lifecycle_assigns_and_disposes_once_before_incomplete() {
    extrusion_islands::reset_hooks();

    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(extrusion_islands::invocations(), 1);
    assert_eq!(extrusion_islands::disposals(), 1);

    extrusion_islands::reset_hooks();
}
