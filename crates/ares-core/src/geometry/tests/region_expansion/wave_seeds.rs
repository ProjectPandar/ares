mod aabb_order;
mod expanded;
mod oracle;
mod recovery;
mod splits;

use super::helpers::{expolygon, params, polygon};
use crate::geometry::region_expansion::wave_seeds;
use crate::geometry::{CoordinateScale, ExPolygon, Point, propagate_waves};

fn square(min: i64, max: i64) -> ExPolygon {
    expolygon(
        &[(min, min), (max, min), (max, max), (min, max)],
        Vec::new(),
    )
}

#[cfg(not(debug_assertions))]
mod release {
    use super::*;
    use crate::geometry::clipper::z::KernelPoint;
    use crate::geometry::region_expansion::recover_path_for_test;

    fn zpath(values: &[(i64, i64, i64)]) -> Vec<KernelPoint> {
        values
            .iter()
            .map(|&(x, y, z)| KernelPoint::new(x, y, z))
            .collect()
    }

    #[test]
    fn closed_fallback_emits_in_release() {
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
    fn failed_closed_containment_drops_in_release() {
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
