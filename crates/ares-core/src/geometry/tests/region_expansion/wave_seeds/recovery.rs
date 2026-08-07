use super::*;
use crate::geometry::clipper::z::{KernelPoint, ZPath};
#[cfg(debug_assertions)]
use crate::geometry::region_expansion::assert_source_topology_for_test;
use crate::geometry::region_expansion::recover_path_for_test;

fn zpath(values: &[(i64, i64, i64)]) -> ZPath {
    values
        .iter()
        .map(|&(x, y, z)| KernelPoint::new(x, y, z))
        .collect()
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn source_endpoint_topology_rejects_an_open_path_without_intersection_metadata() {
    assert_source_topology_for_test(&zpath(&[(0, 0, 2), (10, 0, 2)]), (2, 3));
}

#[test]
fn closed_source_fallback_recovers_source_and_boundary_ids() {
    let seeds = wave_seeds(
        &[square(20, 30)],
        &[square(0, 100)],
        1.0,
        false,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(seeds.len(), 1);
    assert_eq!((seeds[0].src, seeds[0].boundary), (0, 0));
    assert_eq!(
        seeds[0].path.points().first(),
        seeds[0].path.points().last()
    );
}

#[test]
fn crossing_path_recovers_negative_intersection_pair() {
    let source = expolygon(&[(-20, 40), (120, 40), (120, 60), (-20, 60)], Vec::new());
    let seeds = wave_seeds(
        &[source],
        &[square(0, 100)],
        1.0,
        false,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert!(!seeds.is_empty());
    assert!(seeds.iter().all(|seed| seed.src == 0 && seed.boundary == 0));
}

#[test]
fn direct_positive_ids_precede_lazy_fallback_and_preserve_xy() {
    let path = zpath(&[(20, 20, 2), (30, 20, 1), (30, 30, 2)]);
    let (seeds, built) = recover_path_for_test(
        path,
        &[],
        &[square(0, 100)],
        (1, 2, 3),
        CoordinateScale::Normal,
    );
    assert!(!built);
    assert_eq!((seeds[0].src, seeds[0].boundary), (0, 0));
    assert_eq!(
        seeds[0]
            .path
            .points()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect::<Vec<_>>(),
        vec![(20, 20), (30, 20), (30, 30)]
    );
}

#[test]
fn source_only_uses_lazy_containment_and_missing_source_drops() {
    let (inside, built) = recover_path_for_test(
        zpath(&[(20, 20, 2), (30, 20, 2)]),
        &[],
        &[square(0, 100)],
        (1, 2, 3),
        CoordinateScale::Normal,
    );
    assert!(built);
    assert_eq!((inside[0].src, inside[0].boundary), (0, 0));

    let (outside, built) = recover_path_for_test(
        zpath(&[(200, 200, 2), (210, 200, 2)]),
        &[],
        &[square(0, 100)],
        (1, 2, 3),
        CoordinateScale::Normal,
    );
    assert!(built);
    assert!(outside.is_empty());

    let (missing, built) = recover_path_for_test(
        zpath(&[(20, 20, 1), (30, 20, 1)]),
        &[],
        &[square(0, 100)],
        (1, 2, 3),
        CoordinateScale::Normal,
    );
    assert!(!built);
    assert!(missing.is_empty());
}

#[test]
fn negative_front_has_precedence_over_negative_back() {
    let (seeds, built) = recover_path_for_test(
        zpath(&[(0, 0, -2), (5, 0, 2), (10, 0, -1)]),
        &[(1, 2), (1, 3)],
        &[square(-20, 20)],
        (1, 2, 4),
        CoordinateScale::Normal,
    );
    assert!(!built);
    assert_eq!((seeds[0].src, seeds[0].boundary), (1, 0));
    assert_eq!(seeds[0].path.points()[0], Point::new(0, 0));
}

#[test]
fn rare_repair_uses_the_last_source_label() {
    let (seeds, built) = recover_path_for_test(
        zpath(&[(10, 10, 1), (20, 20, 2), (30, 30, 3), (10, 10, 1)]),
        &[],
        &[square(0, 100)],
        (1, 2, 4),
        CoordinateScale::Normal,
    );
    assert!(built);
    assert_eq!((seeds[0].src, seeds[0].boundary), (1, 0));
}

#[test]
fn closed_last_source_label_repairs_front_and_samples_outer_or_hole_boundary() {
    let holed = expolygon(
        &[(0, 0), (100, 0), (100, 100), (0, 100)],
        vec![polygon(&[(40, 40), (60, 40), (60, 60), (40, 60)])],
    );
    for point in [(20, 20), (40, 50)] {
        let (seeds, built) = recover_path_for_test(
            zpath(&[(10, 10, 1), (point.0, point.1, 2), (10, 10, 1)]),
            &[],
            std::slice::from_ref(&holed),
            (1, 2, 3),
            CoordinateScale::Normal,
        );
        assert!(built);
        assert_eq!((seeds[0].src, seeds[0].boundary), (0, 0));
    }
}

#[cfg(not(debug_assertions))]
pub(crate) mod release {
    use super::*;

    #[test]
    fn valid_closed_fallback_remains_available_in_release() {
        let seeds = wave_seeds(
            &[square(20, 30)],
            &[square(0, 100)],
            1.0,
            false,
            CoordinateScale::Normal,
        )
        .unwrap();
        assert_eq!((seeds[0].src, seeds[0].boundary), (0, 0));
    }

    #[test]
    fn invalid_front_pair_continues_to_valid_back_pair() {
        let (seeds, built) = recover_path_for_test(
            zpath(&[(0, 0, -1), (5, 0, 2), (10, 0, -2)]),
            &[(9, 9), (1, 2)],
            &[square(-20, 20)],
            (1, 2, 3),
            CoordinateScale::Normal,
        );
        assert!(!built);
        assert_eq!((seeds[0].src, seeds[0].boundary), (0, 0));
    }

    #[test]
    fn failed_hole_interior_containment_drops_in_release() {
        let holed = expolygon(
            &[(0, 0), (100, 0), (100, 100), (0, 100)],
            vec![polygon(&[(40, 40), (60, 40), (60, 60), (40, 60)])],
        );
        let (seeds, built) = recover_path_for_test(
            zpath(&[(50, 50, 2), (50, 55, 2), (50, 50, 2)]),
            &[],
            &[holed],
            (1, 2, 3),
            CoordinateScale::Normal,
        );
        assert!(built);
        assert!(seeds.is_empty());
    }
}
