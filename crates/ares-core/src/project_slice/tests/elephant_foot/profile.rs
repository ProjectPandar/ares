use crate::geometry::Point;

use super::super::super::elephant_foot::{
    distance::resample_polygon,
    profile::{map_distances_to_compensation, smooth_compensation_banded},
};

fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

#[test]
fn task22m_elephant_foot_resampling_freezes_points_and_parameters() {
    let contour = [
        Point::new(0, 0),
        Point::new(10, 0),
        Point::new(10, 10),
        Point::new(0, 10),
    ];
    let (points, parameters) = resample_polygon(&contour, 4.0).unwrap();

    assert_eq!(
        points,
        [
            Point::new(0, 6),
            Point::new(0, 3),
            Point::new(0, 0),
            Point::new(3, 0),
            Point::new(6, 0),
            Point::new(10, 0),
            Point::new(10, 3),
            Point::new(10, 6),
            Point::new(10, 10),
            Point::new(6, 10),
            Point::new(3, 10),
            Point::new(0, 10),
        ]
    );
    assert_eq!(
        parameters
            .iter()
            .map(|parameter| parameter.source_index)
            .collect::<Vec<_>>(),
        [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3]
    );
    assert_eq!(
        parameters
            .iter()
            .map(|parameter| parameter.interpolated)
            .collect::<Vec<_>>(),
        [
            true, true, false, true, true, false, true, true, false, true, true, false,
        ]
    );
    assert!(
        parameters
            .iter()
            .all(|parameter| parameter.step_length.to_bits() == 0x400a_aaaa_aaaa_aaab)
    );
    assert_eq!(
        parameters
            .iter()
            .map(|parameter| parameter.curve_parameter.to_bits())
            .collect::<Vec<_>>(),
        [
            0x400a_aaaa_aaaa_aaab,
            0x401a_aaaa_aaaa_aaab,
            0x4024_0000_0000_0000,
            0x402a_aaaa_aaaa_aaab,
            0x4030_aaaa_aaaa_aaab,
            0x4034_0000_0000_0000,
            0x4037_5555_5555_5555,
            0x403a_aaaa_aaaa_aaaa,
            0x403d_ffff_ffff_ffff,
            0x4040_aaaa_aaaa_aaaa,
            0x4042_5555_5555_5555,
            0x4044_0000_0000_0000,
        ]
    );
}

#[test]
fn task22m_elephant_foot_resampling_truncates_negative_interpolation_and_skips_short_inputs() {
    let contour = [
        Point::new(-10, -10),
        Point::new(0, -10),
        Point::new(0, 0),
        Point::new(-10, 0),
    ];
    let (points, _) = resample_polygon(&contour, 4.0).unwrap();
    assert_eq!(points[0], Point::new(-10, -3));

    for short in [
        &[][..],
        &[Point::new(0, 0)][..],
        &[Point::new(0, 0), Point::new(1, 1)][..],
    ] {
        let (points, parameters) = resample_polygon(short, 4.0).unwrap();
        assert!(points.is_empty());
        assert!(parameters.is_empty());
    }
}

#[test]
fn task22m_elephant_foot_distance_mapping_freezes_strict_threshold_bits() {
    let mut values = [
        f32::from_bits(0x407f_ffff),
        4.0,
        6.0,
        8.0,
        f32::from_bits(0x4100_0001),
    ];
    map_distances_to_compensation(&mut values, 4.0, 2.0);
    assert_eq!(
        bits(&values),
        [
            0x0000_0000,
            0x8000_0000,
            0xbf80_0000,
            0xc000_0000,
            0xc000_0000
        ]
    );

    let mut inexact_upper_bound = [8.0];
    map_distances_to_compensation(&mut inexact_upper_bound, 4.2, 1.9);
    assert_eq!(bits(&inexact_upper_bound), [0xbff3_3334]);
}

#[test]
fn task22m_elephant_foot_banded_smoothing_freezes_each_jacobi_pass() {
    let contour = [
        Point::new(0, 0),
        Point::new(3, 0),
        Point::new(6, 0),
        Point::new(6, 3),
        Point::new(6, 6),
        Point::new(3, 6),
        Point::new(0, 6),
        Point::new(0, 3),
    ];
    let input = [0.0, -1.0, -2.0, -3.0, -4.0, -3.0, -2.0, -1.0];
    let snapshots = [
        [
            0x0000_0000,
            0xbf80_0000,
            0xc000_0000,
            0xc039_9999,
            0xc066_6666,
            0xc039_9999,
            0xc000_0000,
            0xbf80_0000,
        ],
        [
            0x0000_0000,
            0xbf80_0000,
            0xbffc_28f6,
            0xc032_3d71,
            0xc053_3333,
            0xc032_3d71,
            0xbffc_28f6,
            0xbf80_0000,
        ],
        [
            0x0000_0000,
            0xbf80_0000,
            0xbff6_147a,
            0xc02a_9ba6,
            0xc044_1893,
            0xc02a_9ba6,
            0xbff6_147a,
            0xbf80_0000,
        ],
    ];

    for (passes, expected) in (1..=3).zip(snapshots) {
        let mut values = input;
        smooth_compensation_banded(&contour, &mut values, 4.0, 0.3, passes);
        assert_eq!(bits(&values), expected);
        assert!(
            values
                .iter()
                .zip(input)
                .all(|(&value, source)| value >= source)
        );
    }
}

#[test]
fn task22m_elephant_foot_banded_smoothing_preserves_strict_equality_branches() {
    let immediate_contour = [
        Point::new(0, 0),
        Point::new(1, 0),
        Point::new(2, 1),
        Point::new(-1, 2),
        Point::new(-1, 0),
    ];
    let mut immediate_values = [-1.0, -0.0, 0.0, 0.0, -0.0];
    smooth_compensation_banded(&immediate_contour, &mut immediate_values, 1.0, 1.0, 1);
    assert_eq!(
        bits(&immediate_values),
        [0x8000_0000, 0x8000_0000, 0, 0, 0x8000_0000]
    );

    let accumulated_contour = [
        Point::new(0, 0),
        Point::new(1, 0),
        Point::new(2, 0),
        Point::new(3, 1),
        Point::new(2, 3),
        Point::new(-2, 3),
        Point::new(-3, 1),
        Point::new(-2, 0),
        Point::new(-1, 0),
    ];
    let mut accumulated_values = [-1.0, -0.0, -0.0, 0.0, 0.0, 0.0, 0.0, -0.0, -0.0];
    smooth_compensation_banded(&accumulated_contour, &mut accumulated_values, 2.0, 1.0, 1);
    assert_eq!(
        bits(&accumulated_values),
        [0, 0, 0x8000_0000, 0, 0, 0, 0, 0x8000_0000, 0,]
    );
}
