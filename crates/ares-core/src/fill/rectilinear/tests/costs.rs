use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};

use super::super::{
    compute_region_costs, connect_contours, generate_monotonic_regions, insert_phony_outer_pairs,
    prepare_rectilinear_slice,
};
use super::rectangle;

#[test]
fn task22o85_symmetric_region_costs_normalize_to_zero_for_both_orientations() {
    let mut slice = prepare_rectilinear_slice(&rectangle(), 0.0, -5.0, -10.0, 3, 10, 20).unwrap();
    connect_contours(&mut slice, false, 0.0);
    insert_phony_outer_pairs(&mut slice.lines);
    let mut regions = generate_monotonic_regions(&slice.lines);
    let before = slice.clone();

    compute_region_costs(&mut regions, &slice, CoordinateScale::Normal);

    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].lengths, [0.0, 0.0]);
    assert_eq!(slice, before);
}

#[test]
fn task22o85_region_costs_are_repeatable_and_preserve_scale_rounding() {
    let asymmetric = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(120, 0),
            Point::new(100, 80),
            Point::new(0, 60),
        ]),
        Vec::new(),
    );
    let mut slice = prepare_rectilinear_slice(&asymmetric, 0.0, -5.0, -10.0, 2, 10, 60).unwrap();
    connect_contours(&mut slice, false, 0.0);
    insert_phony_outer_pairs(&mut slice.lines);
    let source = generate_monotonic_regions(&slice.lines);
    let mut normal = source.clone();
    let mut repeat = source.clone();
    let mut large = source;

    compute_region_costs(&mut normal, &slice, CoordinateScale::Normal);
    compute_region_costs(&mut repeat, &slice, CoordinateScale::Normal);
    compute_region_costs(&mut large, &slice, CoordinateScale::LargeBed);

    assert_eq!(normal, repeat);
    assert_eq!(
        normal
            .iter()
            .chain(&large)
            .flat_map(|region| region.lengths.map(f32::to_bits))
            .collect::<Vec<_>>(),
        vec![891_255_680, 0, 919_034_432, 0]
    );
}
