use super::super::generate_fuzzy_painted_region_sort_state::{
    StagedGenerateFuzzySortParent, staged_sort_generate_fuzzy_painted_regions,
};
use super::super::generate_fuzzy_volume_region_state::{
    StagedGenerateFuzzyConfig, StagedGenerateFuzzyParentType, StagedGenerateFuzzyRegion,
    StagedGenerateFuzzySkinType,
};

fn parent(print_object_region_id: u64) -> StagedGenerateFuzzySortParent {
    StagedGenerateFuzzySortParent::new(print_object_region_id)
}

fn fuzzy(
    parent_type: StagedGenerateFuzzyParentType,
    parent: usize,
    region_id: u64,
    marker: u64,
    fuzzy_skin: StagedGenerateFuzzySkinType,
) -> StagedGenerateFuzzyRegion {
    StagedGenerateFuzzyRegion::new(
        parent_type,
        parent,
        region_id,
        StagedGenerateFuzzyConfig::from_parent(marker, fuzzy_skin),
    )
}

#[test]
fn generate_fuzzy_painted_region_sort_orders_volume_parents_by_print_region_id() {
    let sorted = staged_sort_generate_fuzzy_painted_regions(
        &[parent(30), parent(10), parent(20)],
        &[],
        &[
            fuzzy(
                StagedGenerateFuzzyParentType::VolumeRegion,
                0,
                100,
                10,
                StagedGenerateFuzzySkinType::External,
            ),
            fuzzy(
                StagedGenerateFuzzyParentType::VolumeRegion,
                1,
                101,
                20,
                StagedGenerateFuzzySkinType::AllWalls,
            ),
            fuzzy(
                StagedGenerateFuzzyParentType::VolumeRegion,
                2,
                102,
                30,
                StagedGenerateFuzzySkinType::Hole,
            ),
        ],
    );

    let parents: Vec<usize> = sorted.iter().map(|region| region.parent()).collect();
    assert_eq!(parents, vec![1, 2, 0]);
}

#[test]
fn generate_fuzzy_painted_region_sort_orders_painted_parents_by_print_region_id() {
    let sorted = staged_sort_generate_fuzzy_painted_regions(
        &[],
        &[parent(40), parent(20), parent(30)],
        &[
            fuzzy(
                StagedGenerateFuzzyParentType::PaintedRegion,
                0,
                100,
                10,
                StagedGenerateFuzzySkinType::External,
            ),
            fuzzy(
                StagedGenerateFuzzyParentType::PaintedRegion,
                1,
                101,
                20,
                StagedGenerateFuzzySkinType::AllWalls,
            ),
            fuzzy(
                StagedGenerateFuzzyParentType::PaintedRegion,
                2,
                102,
                30,
                StagedGenerateFuzzySkinType::Hole,
            ),
        ],
    );

    let parents: Vec<usize> = sorted.iter().map(|region| region.parent()).collect();
    assert_eq!(parents, vec![1, 2, 0]);
}

#[test]
fn generate_fuzzy_painted_region_sort_orders_mixed_parent_types_by_resolved_id() {
    let sorted = staged_sort_generate_fuzzy_painted_regions(
        &[parent(40), parent(10)],
        &[parent(30), parent(20)],
        &[
            fuzzy(
                StagedGenerateFuzzyParentType::VolumeRegion,
                0,
                100,
                10,
                StagedGenerateFuzzySkinType::External,
            ),
            fuzzy(
                StagedGenerateFuzzyParentType::PaintedRegion,
                0,
                101,
                20,
                StagedGenerateFuzzySkinType::AllWalls,
            ),
            fuzzy(
                StagedGenerateFuzzyParentType::PaintedRegion,
                1,
                102,
                30,
                StagedGenerateFuzzySkinType::Hole,
            ),
            fuzzy(
                StagedGenerateFuzzyParentType::VolumeRegion,
                1,
                103,
                40,
                StagedGenerateFuzzySkinType::Contour,
            ),
        ],
    );

    let order: Vec<(StagedGenerateFuzzyParentType, usize)> = sorted
        .iter()
        .map(|region| (region.parent_type(), region.parent()))
        .collect();
    assert_eq!(
        order,
        vec![
            (StagedGenerateFuzzyParentType::VolumeRegion, 1),
            (StagedGenerateFuzzyParentType::PaintedRegion, 1),
            (StagedGenerateFuzzyParentType::PaintedRegion, 0),
            (StagedGenerateFuzzyParentType::VolumeRegion, 0),
        ]
    );
}

#[test]
fn generate_fuzzy_painted_region_sort_preserves_region_fields() {
    let sorted = staged_sort_generate_fuzzy_painted_regions(
        &[parent(2)],
        &[parent(1)],
        &[
            fuzzy(
                StagedGenerateFuzzyParentType::VolumeRegion,
                0,
                700,
                77,
                StagedGenerateFuzzySkinType::External,
            ),
            fuzzy(
                StagedGenerateFuzzyParentType::PaintedRegion,
                0,
                800,
                88,
                StagedGenerateFuzzySkinType::DisabledFuzzy,
            ),
        ],
    );

    assert_eq!(
        sorted[1].parent_type(),
        StagedGenerateFuzzyParentType::VolumeRegion
    );
    assert_eq!(sorted[1].parent(), 0);
    assert_eq!(sorted[1].region_id(), 700);
    assert_eq!(sorted[1].derived_config().marker(), 77);
    assert_eq!(
        sorted[1].derived_config().fuzzy_skin(),
        StagedGenerateFuzzySkinType::All
    );
}

#[test]
fn generate_fuzzy_painted_region_sort_empty_is_noop() {
    let sorted = staged_sort_generate_fuzzy_painted_regions(&[parent(1)], &[parent(2)], &[]);

    assert!(sorted.is_empty());
}

#[test]
fn generate_fuzzy_painted_region_sort_single_entry_is_noop() {
    let region = fuzzy(
        StagedGenerateFuzzyParentType::VolumeRegion,
        0,
        700,
        77,
        StagedGenerateFuzzySkinType::External,
    );
    let sorted = staged_sort_generate_fuzzy_painted_regions(&[parent(1)], &[], &[region]);

    assert_eq!(sorted, vec![region]);
}
