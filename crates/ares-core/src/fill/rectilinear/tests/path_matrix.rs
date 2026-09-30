use crate::geometry::CoordinateScale;

use super::super::{MonotonicPathMatrix, prepare_rectilinear_slice};
use super::{rectangle, region};

#[test]
fn task22o86_pheromone_reset_preserves_cached_length_and_visibility() {
    let slice = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 2, 10, 80).unwrap();
    let regions = vec![region(0, 0, 0, 1), region(1, 1, 0, 1)];
    let mut matrix = MonotonicPathMatrix::new(&regions, &slice, CoordinateScale::LargeBed, 0.5);
    let before = *matrix.edge(0, false, 1, true);
    matrix.edge(0, false, 1, true).pheromone = 0.9;

    matrix.update_initial_pheromone(0.25);
    let after = *matrix.edge(0, false, 1, true);

    assert_eq!(after.length.to_bits(), before.length.to_bits());
    assert_eq!(after.visibility.to_bits(), before.visibility.to_bits());
    assert_eq!(after.pheromone.to_bits(), 0.25_f32.to_bits());
}
