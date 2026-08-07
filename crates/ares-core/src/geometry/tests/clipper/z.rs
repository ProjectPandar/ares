mod input_fill;
mod lifecycle;
mod output;

use crate::geometry::clipper::{
    Clipper, ClipperOptions, PathRole,
    z::{KernelPoint, ZPath},
};

fn path(points: &[(i64, i64)], z: i64) -> ZPath {
    points
        .iter()
        .map(|&(x, y)| KernelPoint::new(x, y, z))
        .collect()
}

#[cfg(not(debug_assertions))]
mod release {
    use crate::geometry::clipper::z::z_fill_for_test;

    #[test]
    fn collector_preserves_first_two_sorted_labels() {
        assert_eq!(z_fill_for_test([8, 3, 5, 1]), (-1, vec![(1, 3)]));
        assert_eq!(z_fill_for_test([8, 3, 3, 1]), (-1, vec![(1, 3)]));
    }
}

fn crossing_clipper() -> Clipper {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper
        .add_z_closed_path(
            &path(&[(0, 0), (10, 0), (10, 10), (0, 10)], 1),
            PathRole::Clip,
        )
        .unwrap();
    clipper
        .add_z_open_path(&path(&[(-5, 5), (15, 5)], 2), PathRole::Subject)
        .unwrap();
    clipper
}
