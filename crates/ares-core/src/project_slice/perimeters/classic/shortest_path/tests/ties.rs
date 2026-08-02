use super::super::chain_extrusion_paths;
use super::chain::path;

#[test]
fn task22o8_equal_distance_ties_preserve_kd_visit_and_heap_order() {
    let paths = vec![
        path(&[(0, 0, 0), (1, 0, 0)], 1.0, 1.0, 1.0),
        path(&[(2, 1, 0), (3, 1, 0)], 1.0, 1.0, 1.0),
        path(&[(2, -1, 0), (3, -1, 0)], 1.0, 1.0, 1.0),
    ];
    assert_eq!(
        chain_extrusion_paths(&paths, Some([0, 0])),
        vec![(0, false), (1, false), (2, true)]
    );
}

#[test]
fn task22o8_duplicate_endpoints_keep_literal_source_tie_result() {
    let paths = vec![
        path(&[(0, 0, 0), (10, 0, 0)], 1.0, 1.0, 1.0),
        path(&[(10, 0, 0), (20, 0, 0)], 1.0, 1.0, 1.0),
        path(&[(10, 0, 0), (30, 0, 0)], 1.0, 1.0, 1.0),
    ];
    assert_eq!(
        chain_extrusion_paths(&paths, Some([0, 0])),
        vec![(0, false), (1, false), (2, false)]
    );
}
