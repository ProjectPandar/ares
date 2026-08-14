use crate::{
    SliceError,
    project_slice::{
        fill_entities,
        tests::{
            prepare_infill::group_fills::focused::fixture::graph,
            support::{KsrArchive, metadata},
        },
    },
    slice_project,
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

#[tokio::test]
async fn task22o91_public_lifecycle_materializes_and_disposes_once_before_incomplete() {
    fill_entities::reset_hooks();

    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(fill_entities::invocations(), 1);
    assert_eq!(fill_entities::disposals(), 1);

    fill_entities::reset_hooks();
}
