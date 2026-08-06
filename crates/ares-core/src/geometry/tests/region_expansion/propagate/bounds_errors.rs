use super::{seed, square_boundary};
use crate::geometry::tests::region_expansion::helpers::{expolygon, params, polygon};
use crate::geometry::{
    ClipperError, propagate_waves, wavefront_counter_clockwise, wavefront_step_for_test,
};

#[test]
fn task22o27_bbox_inflation_truncates_and_prefilters_distant_ranges() {
    let edge = [expolygon(
        &[(10, 0), (100, 0), (100, 100), (10, 100)],
        vec![],
    )];
    let wave = [seed(1, 0, &[(0, 30), (0, 70)])];
    assert!(
        propagate_waves(&wave, &edge, &params((20.0, 20.0, 0, 9.9, 25.0, 0.1)))
            .unwrap()
            .is_empty()
    );
    assert!(
        !propagate_waves(&wave, &edge, &params((20.0, 20.0, 0, 10.0, 25.0, 0.1)))
            .unwrap()
            .is_empty()
    );

    const OUTSIDE: i64 = 0x4000_0000_0000_0000;
    let distant = [expolygon(
        &[
            (OUTSIDE, 0),
            (OUTSIDE + 100, 0),
            (OUTSIDE + 100, 100),
            (OUTSIDE, 100),
        ],
        vec![],
    )];
    assert_eq!(
        propagate_waves(&wave, &distant, &params((20.0, 20.0, 0, 100.0, 25.0, 0.1))),
        Ok(Vec::new())
    );
}

#[test]
fn task22o27_wave_step_orientation_uses_clipper_operation_order() {
    const HIGH: i64 = 0x3fff_ffff_ffff_ffff;
    let path = polygon(&[
        (HIGH - 6786, 2714),
        (HIGH - 175, -3338),
        (HIGH - 914, -93),
        (HIGH - 4192, -430),
    ]);
    assert!(path.area() >= 0.0);
    assert!(!wavefront_counter_clockwise(&path));
}

#[test]
fn task22o27_clockwise_wave_step_preserves_source_sign_and_reversal() {
    let clockwise = polygon(&[(0, 0), (0, 1000), (1000, 1000), (1000, 0)]);
    let stepped = wavefront_step_for_test(&[clockwise], 100.0, 25.0, 0.5).unwrap();
    assert_eq!(
        stepped
            .iter()
            .map(|path| {
                path.points()
                    .iter()
                    .map(|point| (point.x(), point.y()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![vec![(900, 100), (100, 100), (100, 900), (900, 900)]]
    );
}

#[test]
fn task22o27_propagation_returns_the_first_range_error() {
    const HIGH: i64 = 0x3fff_ffff_ffff_ffff;
    let valid_boundary = [expolygon(
        &[
            (HIGH - 5000, -1000),
            (HIGH, -1000),
            (HIGH, 1000),
            (HIGH - 5000, 1000),
        ],
        vec![],
    )];
    assert_eq!(
        propagate_waves(
            &[seed(1, 0, &[(HIGH - 10, -100), (HIGH - 10, 100)])],
            &valid_boundary,
            &params((100.0, 100.0, 0, 500.0, 25.0, 0.1)),
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
    let ordered_error = std::panic::catch_unwind(|| {
        propagate_waves(
            &[
                seed(1, 0, &[(HIGH - 10, -100), (HIGH - 10, 100)]),
                seed(2, 1, &[(0, 400), (0, 600)]),
            ],
            &valid_boundary,
            &params((100.0, 100.0, 0, 500.0, 25.0, 0.1)),
        )
    })
    .expect("the first group error must precede later boundary access");
    assert_eq!(ordered_error, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(
        propagate_waves(
            &[seed(1, 0, &[(0, 400), (0, 600)])],
            &[square_boundary()],
            &params((
                100.0,
                0x4000_0000_0000_0000_i64 as f32,
                1,
                500.0,
                f64::MAX,
                0.1,
            )),
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );

    let invalid_clip = [expolygon(
        &[
            (HIGH - 4000, -1000),
            (HIGH + 1, -1000),
            (HIGH + 1, 1000),
            (HIGH - 4000, 1000),
        ],
        vec![],
    )];
    assert_eq!(
        propagate_waves(
            &[seed(1, 0, &[(HIGH - 1000, -100), (HIGH - 1000, 100)])],
            &invalid_clip,
            &params((100.0, 100.0, 0, 5000.0, 25.0, 0.1)),
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
