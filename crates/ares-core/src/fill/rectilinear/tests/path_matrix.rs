use crate::geometry::CoordinateScale;

use super::super::{MonotonicPathMatrix, prepare_rectilinear_slice};
use super::{rectangle, region};

#[test]
fn task22o86_matrix_lazily_caches_all_orientation_addresses() {
    let slice = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 2, 10, 80).unwrap();
    let regions = vec![region(0, 0, 0, 1), region(1, 1, 0, 1)];
    let before = regions.clone();
    let mut matrix = MonotonicPathMatrix::new(&regions, &slice, CoordinateScale::Normal, 0.5);

    let costs = [false, true]
        .into_iter()
        .flat_map(|from| {
            [false, true].map(|to| {
                let edge = *matrix.edge(0, from, 1, to);
                (edge.length.to_bits(), edge.visibility.to_bits())
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(
        costs,
        vec![
            (955_073_539, 1_167_236_743),
            (950_519_212, 1_169_005_682),
            (950_519_212, 1_169_005_682),
            (955_073_539, 1_167_236_743),
        ]
    );
    assert_eq!(regions, before);
}

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
