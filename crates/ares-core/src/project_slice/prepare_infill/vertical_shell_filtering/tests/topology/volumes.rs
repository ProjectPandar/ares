use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::prepare_infill::{
        vertical_shell_filtering::{self, filter},
        vertical_shell_regularization::types::VerticalShellRegularization,
        vertical_shell_trimming::types::VerticalShellTrim,
    },
};

use super::{super::empty_record, coordinates};

#[test]
fn task22o23_neighbor_intersection_freezes_first_middle_last_and_topology_order() {
    let lower = vec![rectangle(0, 0, 20, 10)];
    let upper_partial = vec![rectangle(5, -5, 25, 15)];
    let disjoint = vec![rectangle(30, 0, 40, 10)];
    let components = vec![rectangle(0, 0, 10, 10), rectangle(20, 0, 30, 10)];
    let covering = vec![rectangle(-5, -5, 35, 15)];
    let holed_lower = vec![ExPolygon::new(
        rectangle(0, 0, 30, 30).contour().clone(),
        vec![clockwise_rectangle(10, 10, 20, 20)],
    )];
    let holed_upper = vec![rectangle(5, 5, 25, 25)];

    let captured = vec![
        volume(None, Some(&upper_partial)),
        volume(Some(&lower), None),
        volume(Some(&lower), Some(&disjoint)),
        volume(Some(&lower), Some(&upper_partial)),
        volume(Some(&lower), Some(&covering)),
        volume(Some(&components), Some(&covering)),
        volume(Some(&holed_lower), Some(&holed_upper)),
        volume(Some(&upper_partial), Some(&lower)),
    ];
    assert_eq!(captured, expected_neighbor_outputs());
}

fn expected_neighbor_outputs() -> Vec<Vec<Vec<(i64, i64)>>> {
    vec![
        vec![],
        vec![],
        vec![],
        vec![vec![(20, 10), (5, 10), (5, 0), (20, 0)]],
        vec![vec![(20, 10), (0, 10), (0, 0), (20, 0)]],
        vec![
            vec![(30, 10), (20, 10), (20, 0), (30, 0)],
            vec![(10, 10), (0, 10), (0, 0), (10, 0)],
        ],
        vec![
            vec![(25, 25), (5, 25), (5, 5), (25, 5)],
            vec![(10, 10), (10, 20), (20, 20), (20, 10)],
        ],
        vec![vec![(20, 10), (5, 10), (5, 0), (20, 0)]],
    ]
}

fn volume(previous: Option<&[ExPolygon]>, next: Option<&[ExPolygon]>) -> Vec<Vec<(i64, i64)>> {
    vertical_shell_filtering::reset_geometry_hooks();
    filter::filter_record(
        filter::RecordOperands {
            trim: &VerticalShellTrim {
                shell: vec![Polygon::new(vec![
                    Point::new(0, 0),
                    Point::new(1, 0),
                    Point::new(1, 1),
                ])],
            },
            regularization: &VerticalShellRegularization {
                regularized_shell: Vec::new(),
            },
            current: &empty_record(),
            previous_lslices: previous,
            next_lslices: next,
        },
        20,
        CoordinateScale::Normal,
    )
    .unwrap();
    coordinates(&filter::volume_snapshots().pop().unwrap().0)
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(min_x, min_y),
            Point::new(max_x, min_y),
            Point::new(max_x, max_y),
            Point::new(min_x, max_y),
        ]),
        Vec::new(),
    )
}

fn clockwise_rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(min_x, max_y),
        Point::new(max_x, max_y),
        Point::new(max_x, min_y),
    ])
}
