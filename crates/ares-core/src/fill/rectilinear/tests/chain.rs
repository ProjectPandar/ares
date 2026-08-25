use crate::geometry::CoordinateScale;

use super::super::chain::{MonotonicRegionLink, path_length};
use super::super::path_matrix::MonotonicPathMatrix;
use super::super::rng::Mt19937_64;
use super::super::{chain_monotonic_regions, prepare_rectilinear_slice};
use super::{rectangle, region};

#[test]
fn task22o87_default_mt19937_64_matches_standard_output() {
    let mut rng = Mt19937_64::default();
    assert_eq!(
        [rng.next(), rng.next(), rng.next()],
        [
            14_514_284_786_278_117_030,
            4_620_546_740_167_642_908,
            13_109_570_281_517_897_720,
        ]
    );
}

#[test]
fn singleton_distribution_consumes_one_engine_word() {
    let mut rng = Mt19937_64::default();

    assert_eq!(rng.index(1), 0);
    assert_eq!(rng.next(), 4_620_546_740_167_642_908);
}

#[test]
fn task22o87_empty_and_single_region_paths_are_complete_and_deterministic() {
    let slice = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 1, 10, 1).unwrap();
    assert!(chain_monotonic_regions(&[], &slice, CoordinateScale::Normal).is_empty());

    let regions = vec![region(0, 0, 0, 1)];
    let first = chain_monotonic_regions(&regions, &slice, CoordinateScale::Normal);
    let second = chain_monotonic_regions(&regions, &slice, CoordinateScale::Normal);
    assert_eq!(first, second);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].region, 0);
}

#[test]
fn branching_chain_preserves_precedence_and_repeatability() {
    let slice = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 2, 10, 80).unwrap();
    let mut regions = vec![region(0, 0, 0, 1), region(0, 0, 0, 1), region(1, 1, 0, 1)];
    regions[0].right_neighbors = vec![2];
    regions[1].right_neighbors = vec![2];
    regions[2].left_neighbors = vec![0, 1];
    let before = regions.clone();

    let first = chain_monotonic_regions(&regions, &slice, CoordinateScale::Normal);
    let second = chain_monotonic_regions(&regions, &slice, CoordinateScale::Normal);

    assert_eq!(first, second);
    let mut prerequisites = first[..2]
        .iter()
        .map(|link| link.region)
        .collect::<Vec<_>>();
    prerequisites.sort_unstable();
    assert_eq!(prerequisites, [0, 1]);
    assert_eq!(first[2].region, 2);
    assert_eq!(regions, before);
}

#[test]
fn task22o87_path_cost_reduces_in_source_accumulate_order() {
    let slice = prepare_rectilinear_slice(&rectangle(), 0.0, 0.0, 0.0, 1, 10, 1).unwrap();
    let mut regions = vec![region(0, 0, 0, 1), region(0, 0, 0, 1), region(0, 0, 0, 1)];
    regions[0].lengths = [16_777_216.0, 0.0];
    regions[1].lengths = [2.0, 0.0];
    regions[2].lengths = [1.0, 0.0];
    let path = [
        MonotonicRegionLink {
            region: 0,
            flipped: false,
        },
        MonotonicRegionLink {
            region: 1,
            flipped: false,
        },
        MonotonicRegionLink {
            region: 2,
            flipped: false,
        },
    ];
    let mut matrix = MonotonicPathMatrix::new(&regions, &slice, CoordinateScale::Normal, 0.5);

    assert_eq!(path_length(&path, &regions, &mut matrix), 16_777_218.0);
}
