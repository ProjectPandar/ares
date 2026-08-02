use crate::{
    SliceError,
    project_slice::perimeters::{
        classic::perimeter_append, prepare_post_classic_entity_collections,
        prepare_post_classic_perimeter_append,
    },
    slice_project,
};

use super::super::super::super::support::{ksr_project, metadata};

#[test]
fn task22o10_code_level_preparation_reaches_nested_perimeter_append() {
    let prepared = prepare_post_classic_perimeter_append(ksr_project()).unwrap();
    assert!(prepared.objects.iter().any(|object| {
        object.records.iter().flatten().any(|record| {
            record.surfaces.iter().any(|surface| {
                surface.appended.collections.len() == 1
                    && !surface.appended.collections[0].entities.is_empty()
            })
        })
    }));
}

#[test]
fn task22o10_finish_retains_the_exact_boxed_o5_predecessor() {
    let ordered = prepare_post_classic_entity_collections(ksr_project()).unwrap();
    let predecessor = std::ptr::from_ref(ordered.predecessor.as_ref());
    let appended = perimeter_append::finish(ordered);
    assert_eq!(
        std::ptr::from_ref(appended.predecessor.as_ref()),
        predecessor
    );
}

#[tokio::test]
async fn task22o10_public_lifecycle_executes_append_then_stays_incomplete() {
    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}
