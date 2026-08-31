use super::{
    FamilyRequest, MultilineFillParams, Sweep, fill_surface, generate_family, source_base_angle,
};
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
        overlap: 0.0,
        angle: 0.0,
        density: 0.2,
        multiline: 1,
        anchor_length: 0.0,
        anchor_length_max: 0.01,
        dont_sort: false,
    }
}

#[test]
fn source_multiline_base_direction_adds_the_fixed_quarter_turn() {
    assert_eq!(source_base_angle(0.0), std::f32::consts::FRAC_PI_2);
    assert_eq!(
        source_base_angle(std::f32::consts::FRAC_PI_4),
        3.0 * std::f32::consts::FRAC_PI_4
    );
}

#[test]
fn zero_angle_family_uses_vertical_scanlines() {
    let surface = square();
    let paths = generate_family(FamilyRequest {
        component: &surface,
        source: &surface,
        reference: Point::new(0, 0),
        params: params(),
        density: 0.2,
        sweep: Sweep {
            angle: 0.0,
            shift: 0.0,
        },
        x_margin: 400_100,
        scale: CoordinateScale::Normal,
    })
    .unwrap();

    assert!(paths.iter().all(|path| {
        let points = path.points();
        points.first().unwrap().x() == points.last().unwrap().x()
    }));
}

#[test]
fn cubic_transition_keeps_one_connected_path_without_tangent_fragments() {
    let surface = ExPolygon::new(
        Polygon::new(vec![
            Point::new(-3_846_111, -3_846_111),
            Point::new(3_846_118, -3_846_111),
            Point::new(3_846_118, 3_846_118),
            Point::new(-3_846_111, 3_846_118),
        ]),
        Vec::new(),
    );
    let shift = (std::f64::consts::FRAC_1_SQRT_2 * 5.4) as f32;
    let paths = fill_surface(
        &surface,
        MultilineFillParams {
            spacing: 0.407_079_637_050_628_66,
            overlap: 0.0,
            angle: std::f32::consts::FRAC_PI_4,
            density: 0.15,
            multiline: 1,
            anchor_length: 1.628_318_5,
            anchor_length_max: 40.0,
            dont_sort: false,
        },
        &[
            Sweep { angle: 0.0, shift },
            Sweep {
                angle: std::f32::consts::FRAC_PI_3,
                shift: -shift,
            },
            Sweep {
                angle: 2.0 * std::f32::consts::FRAC_PI_3,
                shift,
            },
        ],
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].points().len(), 10);
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
