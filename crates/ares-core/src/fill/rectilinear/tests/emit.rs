use crate::geometry::{CoordinateScale, Point};

use super::super::{
    chain_monotonic_regions, compute_region_costs, connect_contours, connect_region_neighbors,
    emit_monotonic_polylines, generate_monotonic_regions, insert_phony_outer_pairs,
    prepare_rectilinear_slice,
};
use super::rectangle;

#[test]
fn task22o88_empty_chain_emits_no_polylines() {
    let slice = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 1, 10, 1).unwrap();
    assert!(emit_monotonic_polylines(&[], &[], &slice, CoordinateScale::Normal).is_empty());
}

#[test]
fn task22o88_rectangular_chain_emits_exact_repeatable_zigzag_points() {
    let mut slice = prepare_rectilinear_slice(&rectangle(), 0.0, -5.0, -10.0, 3, 10, 20).unwrap();
    connect_contours(&mut slice, false, 0.0);
    insert_phony_outer_pairs(&mut slice.lines);
    let mut regions = generate_monotonic_regions(&slice.lines);
    connect_region_neighbors(&mut regions, &slice.lines);
    compute_region_costs(&mut regions, &slice, CoordinateScale::Normal);
    let path = chain_monotonic_regions(&regions, &slice, CoordinateScale::Normal);
    let before = (regions.clone(), slice.clone(), path.clone());

    let first = emit_monotonic_polylines(&path, &regions, &slice, CoordinateScale::Normal);
    let second = emit_monotonic_polylines(&path, &regions, &slice, CoordinateScale::Normal);

    assert_eq!(first, second);
    assert_eq!(
        first
            .iter()
            .map(|polyline| polyline.points().to_vec())
            .collect::<Vec<_>>(),
        vec![vec![
            Point::new(10, 5),
            Point::new(10, 70),
            Point::new(30, 70),
            Point::new(30, 10),
            Point::new(50, 10),
            Point::new(50, 75),
        ]]
    );
    assert_eq!((regions, slice, path), before);
}
