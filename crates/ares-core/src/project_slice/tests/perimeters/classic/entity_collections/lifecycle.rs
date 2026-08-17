use crate::project_slice::perimeters::{
    classic::entity_collections, prepare_post_classic_chained_loops,
    prepare_post_classic_entity_collections,
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o9_code_level_preparation_constructs_ordered_collections() {
    let prepared = prepare_post_classic_entity_collections(ksr_project()).unwrap();
    assert!(prepared.objects.iter().any(|object| {
        object.records.iter().flatten().any(|record| {
            record
                .surfaces
                .iter()
                .any(|surface| !surface.collection.entities.is_empty())
        })
    }));
}

#[test]
fn task22o9_finish_retains_the_exact_boxed_o5_predecessor() {
    let chained = prepare_post_classic_chained_loops(ksr_project()).unwrap();
    let predecessor = std::ptr::from_ref(chained.predecessor.as_ref());
    let ordered = entity_collections::finish(chained);
    assert_eq!(
        std::ptr::from_ref(ordered.predecessor.as_ref()),
        predecessor
    );
}
