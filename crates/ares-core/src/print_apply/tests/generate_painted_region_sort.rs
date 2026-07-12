use super::super::generate_painted_region_sort_state::{
    StagedGeneratePaintedSortParent, StagedGeneratePaintedSortableRegion,
    staged_sort_generate_painted_regions,
};

fn parent(print_object_region_id: u64) -> StagedGeneratePaintedSortParent {
    StagedGeneratePaintedSortParent::new(print_object_region_id)
}

fn painted(
    extruder_id: u32,
    parent: usize,
    region_id: u64,
    marker: u64,
) -> StagedGeneratePaintedSortableRegion {
    StagedGeneratePaintedSortableRegion::new(extruder_id, parent, region_id, marker)
}

#[test]
fn generate_painted_region_sort_orders_by_parent_print_region_id() {
    let sorted = staged_sort_generate_painted_regions(
        &[parent(30), parent(10), parent(20)],
        &[
            painted(5, 0, 100, 1),
            painted(5, 1, 101, 2),
            painted(5, 2, 102, 3),
        ],
    );

    let parents: Vec<usize> = sorted.iter().map(|region| region.parent()).collect();
    assert_eq!(parents, vec![1, 2, 0]);
}

#[test]
fn generate_painted_region_sort_breaks_ties_by_extruder_id() {
    let sorted = staged_sort_generate_painted_regions(
        &[parent(10), parent(10)],
        &[
            painted(9, 0, 100, 1),
            painted(3, 1, 101, 2),
            painted(7, 0, 102, 3),
        ],
    );

    let order: Vec<(u32, usize)> = sorted
        .iter()
        .map(|region| (region.extruder_id(), region.parent()))
        .collect();
    assert_eq!(order, vec![(3, 1), (7, 0), (9, 0)]);
}

#[test]
fn generate_painted_region_sort_preserves_region_fields() {
    let sorted = staged_sort_generate_painted_regions(
        &[parent(2), parent(1)],
        &[painted(4, 0, 700, 77), painted(2, 1, 800, 88)],
    );

    assert_eq!(sorted[1].extruder_id(), 4);
    assert_eq!(sorted[1].parent(), 0);
    assert_eq!(sorted[1].region_id(), 700);
    assert_eq!(sorted[1].marker(), 77);
}

#[test]
fn generate_painted_region_sort_empty_is_noop() {
    let sorted = staged_sort_generate_painted_regions(&[parent(1)], &[]);

    assert!(sorted.is_empty());
}

#[test]
fn generate_painted_region_sort_single_entry_is_noop() {
    let sorted = staged_sort_generate_painted_regions(&[parent(1)], &[painted(4, 0, 700, 77)]);

    assert_eq!(sorted, vec![painted(4, 0, 700, 77)]);
}
