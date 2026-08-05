use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::{
            vertical_shell_filtering::{self, filter},
            vertical_shell_regularization::types::VerticalShellRegularization,
            vertical_shell_trimming::types::VerticalShellTrim,
        },
        region_slices::RegionSurface,
    },
};

use super::{super::empty_record, coordinates};

#[test]
fn task22o23_flat_miter_three_closing_freezes_scale_order_holes_and_counts() {
    let captured = vec![
        closing(two_components(150, 1), CoordinateScale::Normal),
        closing(two_components(250, 1), CoordinateScale::Normal),
        closing(holed(1), CoordinateScale::Normal),
        closing(mixed_winding(1), CoordinateScale::Normal),
        closing(two_components(150, 10), CoordinateScale::LargeBed),
        closing(two_components(250, 10), CoordinateScale::LargeBed),
        closing(holed(10), CoordinateScale::LargeBed),
        closing(mixed_winding(10), CoordinateScale::LargeBed),
    ];
    assert_eq!(captured, expected_closing_outputs());
}

fn expected_closing_outputs() -> Vec<Vec<Vec<(i64, i64)>>> {
    vec![
        vec![vec![(2150, 1000), (0, 1000), (0, 0), (2150, 0)]],
        vec![
            vec![(1000, 1000), (0, 1000), (0, 0), (1000, 0)],
            vec![(2250, 1000), (1250, 1000), (1250, 0), (2250, 0)],
        ],
        vec![
            vec![(3000, 3000), (0, 3000), (0, 0), (3000, 0)],
            vec![(1000, 1000), (1000, 2000), (2000, 2000), (2000, 1000)],
        ],
        vec![
            vec![(3000, 1000), (2000, 1000), (2000, 0), (3000, 0)],
            vec![(800, 800), (200, 800), (200, 200), (800, 200)],
        ],
        vec![vec![(215, 100), (0, 100), (0, 0), (215, 0)]],
        vec![
            vec![(100, 100), (0, 100), (0, 0), (100, 0)],
            vec![(225, 100), (125, 100), (125, 0), (225, 0)],
        ],
        vec![
            vec![(300, 300), (0, 300), (0, 0), (300, 0)],
            vec![(100, 100), (100, 200), (200, 200), (200, 100)],
        ],
        vec![
            vec![(300, 100), (200, 100), (200, 0), (300, 0)],
            vec![(80, 80), (20, 80), (20, 20), (80, 20)],
        ],
    ]
}

fn closing(internal: Vec<ExPolygon>, scale: CoordinateScale) -> Vec<Vec<(i64, i64)>> {
    let mut current = empty_record();
    current
        .fill_surfaces
        .extend(internal.into_iter().map(RegionSurface::internal));
    vertical_shell_filtering::reset_geometry_hooks();
    filter::filter_record(
        filter::RecordOperands {
            trim: &VerticalShellTrim {
                shell: vec![polygon(&[(0, 0), (1, 0), (1, 1)])],
            },
            regularization: &VerticalShellRegularization {
                regularized_shell: Vec::new(),
            },
            current: &current,
            previous_lslices: None,
            next_lslices: None,
        },
        20,
        scale,
    )
    .unwrap();
    coordinates(&filter::volume_snapshots().pop().unwrap().1)
}

fn two_components(gap: i64, divisor: i64) -> Vec<ExPolygon> {
    let width = 1_000 / divisor;
    let height = 1_000 / divisor;
    let gap = gap / divisor;
    vec![
        ExPolygon::new(rectangle(0, 0, width, height), Vec::new()),
        ExPolygon::new(
            rectangle(width + gap, 0, width * 2 + gap, height),
            Vec::new(),
        ),
    ]
}

fn holed(divisor: i64) -> Vec<ExPolygon> {
    vec![ExPolygon::new(
        rectangle(0, 0, 3_000 / divisor, 3_000 / divisor),
        vec![clockwise_rectangle(
            1_000 / divisor,
            1_000 / divisor,
            2_000 / divisor,
            2_000 / divisor,
        )],
    )]
}

fn mixed_winding(divisor: i64) -> Vec<ExPolygon> {
    vec![
        ExPolygon::new(
            clockwise_rectangle(0, 0, 1_000 / divisor, 1_000 / divisor),
            Vec::new(),
        ),
        ExPolygon::new(
            rectangle(2_000 / divisor, 0, 3_000 / divisor, 1_000 / divisor),
            Vec::new(),
        ),
    ]
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    polygon(&[
        (min_x, min_y),
        (max_x, min_y),
        (max_x, max_y),
        (min_x, max_y),
    ])
}

fn clockwise_rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    polygon(&[
        (min_x, min_y),
        (min_x, max_y),
        (max_x, max_y),
        (max_x, min_y),
    ])
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}
