use crate::{
    geometry::{JoinType, Point, Polygon, SAFETY_OFFSET, opening_paths},
    project_slice::prepare_infill::horizontal_shell_propagation::geometry,
};

#[test]
fn propagation_opening_is_asymmetric_miter5_with_exact_safety_expansion() {
    let paths = vec![Polygon::new(vec![
        Point::new(0, 0),
        Point::new(26_800, 100_000),
        Point::new(-26_800, 100_000),
    ])];
    let margin = 1_000.0_f32;
    let actual = geometry::opening_for_test(&paths, margin).unwrap();
    let asymmetric =
        opening_paths(&paths, margin, margin + SAFETY_OFFSET, JoinType::Miter, 5.0).unwrap();
    let symmetric = opening_paths(&paths, margin, margin, JoinType::Miter, 5.0).unwrap();
    let miter3 =
        opening_paths(&paths, margin, margin + SAFETY_OFFSET, JoinType::Miter, 3.0).unwrap();
    assert_eq!(actual, asymmetric);
    assert_ne!(actual, symmetric);
    assert_ne!(actual, miter3);
}
