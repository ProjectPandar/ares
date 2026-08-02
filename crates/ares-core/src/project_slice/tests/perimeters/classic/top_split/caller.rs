use crate::project_slice::perimeters::{
    classic::top_split::TopSplitOutcome, prepare_post_classic_top_split,
};

use super::support::project;

#[test]
fn task22o2_top_split_records_remain_aligned_with_owned_prelude() {
    let prepared = prepare_post_classic_top_split(project()).unwrap();
    assert!(!prepared.objects.is_empty());
    for object in prepared.objects {
        let (prelude, split_records) = object.into_parts();
        let (_, prelude_records) = prelude.into_parts();
        assert_eq!(prelude_records.len(), split_records.len());
        for (before, after) in prelude_records.iter().zip(&split_records) {
            assert_eq!(before.is_some(), after.is_some());
            if let (Some(before), Some(after)) = (before, after) {
                assert_eq!(before.surfaces.len(), after.surfaces.len());
            }
        }
        let surface_pairs = prelude_records
            .iter()
            .zip(&split_records)
            .filter_map(|(before, after)| Some((before.as_ref()?, after.as_ref()?)))
            .flat_map(|(before, after)| before.surfaces.iter().zip(&after.surfaces));
        for (surface, split) in surface_pairs {
            assert_eq!(surface.source_index, split.source_index);
            assert_eq!(surface.loop_number, split.initial_loop_number);
        }
    }
}

#[test]
fn task22o2_top_split_applies_after_assigning_normal_offsets_to_last() {
    let prepared = prepare_post_classic_top_split(project()).unwrap();
    let applied = prepared
        .objects
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flatten()
        .flat_map(|record| record.surfaces)
        .filter(|surface| surface.outcome == TopSplitOutcome::Applied)
        .collect::<Vec<_>>();
    assert!(!applied.is_empty());
    assert!(
        applied
            .iter()
            .all(|surface| surface.initial_loop_number > 0)
    );
    assert!(
        applied
            .iter()
            .filter(|surface| surface.normal_first_offset.is_empty())
            .all(|surface| {
                surface.remaining.is_empty()
                    && surface.top_fills.is_empty()
                    && surface.fill_clip.is_empty()
            })
    );
}
