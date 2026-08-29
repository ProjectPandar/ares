use super::{MultilineFillParams, Sweep, fill_surface, generate_family};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon};

fn square() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(-10_000_000, -10_000_000),
            Point::new(10_000_000, -10_000_000),
            Point::new(10_000_000, 10_000_000),
            Point::new(-10_000_000, 10_000_000),
        ]),
        Vec::new(),
    )
}

fn params() -> MultilineFillParams {
    MultilineFillParams {
        spacing: 0.4,
        angle: 0.0,
        density: 0.2,
        multiline: 1,
        anchor_length: 0.0,
        anchor_length_max: 0.01,
        dont_sort: false,
    }
}

#[test]
fn zero_angle_family_uses_vertical_scanlines() {
    let paths = generate_family(
        &square(),
        params(),
        0.2,
        Sweep {
            angle: 0.0,
            shift: 0.0,
        },
        CoordinateScale::Normal,
    )
    .unwrap();

    assert!(paths.iter().all(|path| {
        let points = path.points();
        points.first().unwrap().x() == points.last().unwrap().x()
    }));
}

#[test]
fn two_families_split_total_density_once() {
    let paths = fill_surface(
        &square(),
        params(),
        &[
            Sweep {
                angle: 0.0,
                shift: 0.0,
            },
            Sweep {
                angle: std::f32::consts::FRAC_PI_2,
                shift: 0.0,
            },
        ],
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(paths.len(), 10);
    assert_eq!(
        paths.iter().map(|path| path.points().len()).sum::<usize>(),
        20
    );
}

#[test]
fn pattern_shift_moves_the_scanline_grid() {
    let unshifted = fill_surface(
        &square(),
        params(),
        &[Sweep {
            angle: 0.0,
            shift: 0.0,
        }],
        CoordinateScale::Normal,
    )
    .unwrap();
    let shifted = fill_surface(
        &square(),
        params(),
        &[Sweep {
            angle: 0.0,
            shift: 0.5,
        }],
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_ne!(unshifted, shifted);
    assert!(unshifted.len().abs_diff(shifted.len()) <= 1);
}
