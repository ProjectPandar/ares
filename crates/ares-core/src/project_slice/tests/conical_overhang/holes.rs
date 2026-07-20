use crate::geometry::{CoordinateScale, ExPolygon};

use super::{
    apply_objects, donut, layer_geometry, object_options, output_rectangle, polygon, post_region,
    print_object, rectangle, region_options,
};

const OUTER: (i64, i64, i64, i64) = (0, 0, 4_000_000, 4_000_000);
const SMALL_HOLE: (i64, i64, i64, i64) = (1_000_000, 1_000_000, 2_000_200, 2_000_200);

#[test]
fn task22l_stage_hole_positive_and_negative_zero_both_skip_protection() {
    for hole_size in [0.0, -0.0] {
        let output = run(
            donut(OUTER, &[SMALL_HOLE]),
            vec![rectangle(900_000, 900_000, 2_100_200, 2_100_200)],
            hole_size,
        );
        assert_eq!(output, vec![output_rectangle(0, 0, 4_000_000, 4_000_000)]);
    }
}

#[test]
fn task22l_stage_hole_threshold_is_strict_and_full_coverage_is_protected() {
    let equal_hole = (1_000_000, 1_000_000, 2_048_776, 2_048_776);
    assert_eq!(
        run(
            donut(OUTER, &[equal_hole]),
            vec![rectangle(900_000, 900_000, 2_148_776, 2_148_776)],
            1.099_511_627_776,
        ),
        vec![output_rectangle(0, 0, 4_000_000, 4_000_000)]
    );

    assert_eq!(
        run(
            donut(OUTER, &[SMALL_HOLE]),
            vec![rectangle(900_000, 900_000, 2_100_200, 2_100_200)],
            1.1,
        ),
        vec![expected_donut(SMALL_HOLE)]
    );
}

#[test]
fn task22l_stage_hole_partial_and_uncovered_regions_are_not_protected() {
    assert_eq!(
        run(
            donut(OUTER, &[SMALL_HOLE]),
            vec![rectangle(900_000, 900_000, 1_500_100, 2_100_200)],
            1.1,
        ),
        vec![ExPolygon::new(
            polygon(&[
                (4_000_000, 4_000_000),
                (0, 4_000_000),
                (0, 0),
                (4_000_000, 0),
            ]),
            vec![polygon(&[
                (1_500_100, 1_000_000),
                (1_500_100, 2_000_200),
                (2_000_200, 2_000_200),
                (2_000_200, 1_000_000),
            ])],
        )]
    );

    assert_eq!(
        run(
            donut(OUTER, &[SMALL_HOLE]),
            vec![rectangle(2_500_000, 2_500_000, 3_500_000, 3_500_000)],
            1.1,
        ),
        vec![expected_donut(SMALL_HOLE)]
    );
}

#[test]
fn task22l_stage_multiple_nonrectangular_holes_keep_fixed_ordered_result() {
    let lower = ExPolygon::new(
        polygon(&[
            (0, 0),
            (7_000_000, 0),
            (7_000_000, 4_000_000),
            (0, 4_000_000),
        ]),
        vec![
            polygon(&[
                (1_000_000, 1_000_000),
                (1_000_000, 2_000_000),
                (2_000_000, 1_000_000),
            ]),
            polygon(&[
                (4_000_000, 800_000),
                (3_000_000, 2_000_000),
                (4_000_000, 3_200_000),
                (5_000_000, 2_000_000),
            ]),
        ],
    );
    let output = run(
        lower,
        vec![
            rectangle(800_000, 800_000, 2_200_000, 2_200_000),
            rectangle(3_500_000, 500_000, 4_500_000, 2_000_000),
        ],
        3.0,
    );

    assert_eq!(
        output,
        vec![ExPolygon::new(
            polygon(&[
                (7_000_000, 4_000_000),
                (0, 4_000_000),
                (0, 0),
                (7_000_000, 0)
            ]),
            vec![
                polygon(&[
                    (4_500_000, 2_000_000),
                    (3_500_000, 2_000_000),
                    (3_500_000, 1_400_000),
                    (3_000_000, 2_000_000),
                    (4_000_000, 3_200_000),
                    (5_000_000, 2_000_000),
                    (4_500_000, 1_400_000)
                ]),
                polygon(&[
                    (1_000_000, 1_000_000),
                    (1_000_000, 2_000_000),
                    (2_000_000, 1_000_000)
                ]),
            ],
        )]
    );
}

#[test]
fn task22l_stage_high_coordinate_hole_uses_existing_cross_accumulation() {
    const BASE: i64 = 9_007_199_252_740_000;
    let hole = (
        BASE + 2_000_000,
        BASE + 2_000_000,
        BASE + 3_000_200,
        BASE + 3_000_200,
    );
    let output = run(
        donut((BASE, BASE, BASE + 5_000_000, BASE + 5_000_000), &[hole]),
        vec![rectangle(
            BASE + 1_900_000,
            BASE + 1_900_000,
            BASE + 3_100_200,
            BASE + 3_100_200,
        )],
        0.5,
    );

    assert_eq!(
        output,
        vec![ExPolygon::new(
            polygon(&[
                (BASE + 5_000_000, BASE + 5_000_000),
                (BASE, BASE + 5_000_000),
                (BASE, BASE),
                (BASE + 5_000_000, BASE),
            ]),
            vec![polygon(&[
                (BASE + 2_000_000, BASE + 2_000_000),
                (BASE + 2_000_000, BASE + 3_000_200),
                (BASE + 3_000_200, BASE + 3_000_200),
                (BASE + 3_000_200, BASE + 2_000_000),
            ])],
        )]
    );
}

fn run(lower: ExPolygon, upper: Vec<ExPolygon>, hole_size: f64) -> Vec<ExPolygon> {
    let region = post_region(
        0,
        region_options(true, 1, 0, 0.0, 0),
        vec![vec![lower], upper],
    );
    let mut object = print_object(0, 0, &[0.2, 0.2], vec![region]);
    apply_objects(
        std::slice::from_mut(&mut object),
        vec![object_options(0.0, hole_size, 0.2)],
        CoordinateScale::Normal,
    )
    .unwrap();
    layer_geometry(&object, 0, 0)
}

fn expected_donut(hole: (i64, i64, i64, i64)) -> ExPolygon {
    let (min_x, min_y, max_x, max_y) = hole;
    ExPolygon::new(
        polygon(&[
            (4_000_000, 4_000_000),
            (0, 4_000_000),
            (0, 0),
            (4_000_000, 0),
        ]),
        vec![polygon(&[
            (min_x, min_y),
            (min_x, max_y),
            (max_x, max_y),
            (max_x, min_y),
        ])],
    )
}
