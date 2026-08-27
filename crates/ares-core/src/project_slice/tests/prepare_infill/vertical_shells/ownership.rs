use crate::project_slice::{
    perimeters,
    prepare_infill::{fill_surfaces, surface_type_detection, vertical_shells},
    tests::{prepare_infill::fill_surfaces::ownership::allocation_snapshot, support::KsrArchive},
};

#[test]
fn task22o19_moves_o18_allocations_and_allocates_distinct_cache_paths() {
    let detected = surface_type_detection::prepare(
        perimeters::prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap(),
    )
    .unwrap();
    let prepared = fill_surfaces::prepare(detected);
    let predecessor = std::ptr::from_ref(prepared.predecessor.as_ref());
    let before = allocation_snapshot(&prepared.objects);
    let source_points = prepared.objects[0].records[0].as_ref().unwrap().slices[0]
        .as_parts()
        .1
        .contour()
        .points()
        .as_ptr();
    let output = vertical_shells::prepare(prepared).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(allocation_snapshot(&output.objects), before);
    assert_ne!(
        output.caches[0].records[0]
            .as_ref()
            .unwrap()
            .bottom_surfaces[0]
            .points()
            .as_ptr(),
        source_points
    );
}

#[test]
fn task22o19_aligned_none_slot_stays_none_without_shifting_neighbors() {
    let detected = surface_type_detection::prepare(
        perimeters::prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap(),
    )
    .unwrap();
    let mut prepared = fill_surfaces::prepare(detected);
    prepared.objects[0].records[1] = None;
    let prelude = &mut prepared.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    prelude.object.records[1] = None;
    prelude.records[1] = None;

    let output = vertical_shells::prepare(prepared).unwrap();
    assert!(output.objects[0].records[0].is_some());
    assert!(output.objects[0].records[1].is_none());
    assert!(output.objects[0].records[2].is_some());
    assert!(output.caches[0].records[0].is_some());
    assert!(output.caches[0].records[1].is_none());
    assert!(output.caches[0].records[2].is_some());
}
