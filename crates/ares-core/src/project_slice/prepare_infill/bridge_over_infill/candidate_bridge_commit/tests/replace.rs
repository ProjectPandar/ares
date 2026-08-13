use super::*;

#[test]
fn task22o64_replaces_original_layer_with_completed_allocation_and_order() {
    let mut current = vec![
        candidate(source(12, 0, 0), -100, 0.0),
        candidate(source(12, 0, 1), -80, 0.1),
    ];
    let completed = vec![
        candidate(source(12, 4, 9), 70, 0.9),
        candidate(source(12, 2, 3), -20, -0.25),
        candidate(source(12, 8, 5), 40, 1.5),
    ];
    let completed_ptr = completed.as_ptr();
    let polygon_ptrs = completed
        .iter()
        .map(|candidate| candidate.new_polygons.as_ptr())
        .collect::<Vec<_>>();
    let expected = snapshot(&completed);

    replace_candidate_layer(&mut current, completed);

    assert_eq!(current.as_ptr(), completed_ptr);
    assert_eq!(snapshot(&current), expected);
    assert_eq!(
        current
            .iter()
            .map(|candidate| candidate.new_polygons.as_ptr())
            .collect::<Vec<_>>(),
        polygon_ptrs
    );
    assert_eq!(
        current
            .iter()
            .map(|candidate| candidate.source.surface_index)
            .collect::<Vec<_>>(),
        vec![9, 3, 5]
    );
}

#[test]
fn task22o64_empty_completed_vector_clears_original_layer_and_keeps_allocation() {
    let mut current = vec![candidate(source(2, 1, 8), 10, 0.25)];
    let completed = Vec::with_capacity(5);
    let completed_ptr = completed.as_ptr();
    let completed_capacity = completed.capacity();

    replace_candidate_layer(&mut current, completed);

    assert!(current.is_empty());
    assert_eq!(current.as_ptr(), completed_ptr);
    assert_eq!(current.capacity(), completed_capacity);
}

#[test]
fn task22o64_replacement_touches_only_the_supplied_vector() {
    let mut current = vec![candidate(source(4, 0, 1), 0, 0.1)];
    let untouched = vec![candidate(source(5, 0, 2), 100, 0.2)];
    let untouched_before = snapshot(&untouched);
    let completed = vec![candidate(source(4, 3, 7), 50, 0.7)];

    replace_candidate_layer(&mut current, completed);

    assert_eq!(snapshot(&untouched), untouched_before);
    assert_eq!(current[0].source, source(4, 3, 7));
}
