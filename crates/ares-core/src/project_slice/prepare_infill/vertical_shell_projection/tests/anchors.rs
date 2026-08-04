use crate::{
    SliceError,
    geometry::{ExPolygon, JoinType, Point, Polygon, offset_paths},
    project_slice::prepare_infill::vertical_shell_projection::{
        GeometryStep, fail_geometry_at, gather, geometry_events, reset_geometry_hooks,
    },
};

use super::{cache, layer, lslice, options, projection_input, square};

#[test]
fn task22o20_count_one_zero_thickness_runs_top_then_bottom_anchors() {
    reset_geometry_hooks();
    let caches = vec![cache(0), cache(10), cache(20)];
    let layers = vec![layer(0, 0.2, 0.2), layer(1, 0.2, 0.4), layer(2, 0.2, 0.6)];
    let lslices = vec![lslice(-100, 100); 3];
    let mut options = options();
    options.top_shell_layers.0 = 1;
    options.top_shell_thickness.0 = 0.0;
    options.bottom_shell_layers.0 = 1;
    options.bottom_shell_thickness.0 = 0.0;
    let output =
        gather::project_record(1, projection_input(&caches, &layers, &lslices, &options, 5))
            .unwrap();
    assert_eq!(x_span(&output.shell[0]), 20);
    assert_eq!(
        geometry_events(),
        [
            GeometryStep::TopAnchorOffset,
            GeometryStep::TopAnchorIntersection,
            GeometryStep::BottomAnchorOffset,
            GeometryStep::BottomAnchorIntersection,
            GeometryStep::ShellUnion,
        ]
    );
}

#[test]
fn task22o20_anchor_uses_current_spacing_and_stopped_lslice_with_contour_then_hole() {
    reset_geometry_hooks();
    let mut current = cache(0).unwrap();
    current.top_surfaces = vec![
        square(0, 100_000_000),
        Polygon::new(vec![
            Point::new(30_000_000, 30_000_000),
            Point::new(30_000_000, 70_000_000),
            Point::new(70_000_000, 70_000_000),
            Point::new(70_000_000, 30_000_000),
        ]),
    ];
    let caches = vec![Some(current), cache(20)];
    let layers = vec![layer(0, 0.2, 0.2), layer(1, 0.2, 0.4)];
    let stopped = ExPolygon::new(
        square(-200_000_000, 200_000_000),
        vec![Polygon::new(vec![
            Point::new(49_000_000, 49_000_000),
            Point::new(49_000_000, 51_000_000),
            Point::new(51_000_000, 51_000_000),
            Point::new(51_000_000, 49_000_000),
        ])],
    );
    let lslices = vec![Vec::new(), vec![stopped]];
    let mut options = options();
    options.top_shell_layers.0 = 1;
    options.top_shell_thickness.0 = 0.0;
    options.bottom_shell_layers.0 = 0;
    let spacing = 16_777_217_i64;
    let output = gather::project_record(
        0,
        projection_input(&caches, &layers, &lslices, &options, spacing),
    )
    .unwrap();
    assert_eq!(spacing as f32, 16_777_216.0_f32);
    assert_eq!(
        output.shell,
        vec![
            Polygon::new(vec![
                Point::new(116_777_216, 116_777_216),
                Point::new(-16_777_216, 116_777_216),
                Point::new(-16_777_216, -16_777_216),
                Point::new(116_777_216, -16_777_216),
            ]),
            Polygon::new(vec![
                Point::new(46_777_216, 46_777_216),
                Point::new(46_777_216, 53_222_784),
                Point::new(53_222_784, 53_222_784),
                Point::new(53_222_784, 46_777_216),
            ]),
        ]
    );
    assert_eq!(
        geometry_events(),
        [
            GeometryStep::TopAnchorOffset,
            GeometryStep::TopAnchorIntersection
        ]
    );
}

#[test]
fn task22o20_each_anchor_geometry_site_has_an_independent_failure_hook() {
    for step in [
        GeometryStep::TopAnchorOffset,
        GeometryStep::TopAnchorIntersection,
        GeometryStep::BottomAnchorOffset,
        GeometryStep::BottomAnchorIntersection,
    ] {
        reset_geometry_hooks();
        fail_geometry_at(step);
        let caches = vec![cache(0), cache(0), cache(0)];
        let layers = vec![layer(0, 0.2, 0.2), layer(1, 0.2, 0.4), layer(2, 0.2, 0.6)];
        let lslices = vec![lslice(-100, 100); 3];
        let mut options = options();
        options.top_shell_layers.0 = 1;
        options.top_shell_thickness.0 = 0.0;
        options.bottom_shell_layers.0 = 1;
        options.bottom_shell_thickness.0 = 0.0;
        let error =
            gather::project_record(1, projection_input(&caches, &layers, &lslices, &options, 5))
                .unwrap_err();
        assert_eq!(
            error,
            SliceError::InvalidInput(
                "vertical-shell projection geometry is outside the supported Clipper range"
                    .to_owned()
            )
        );
    }
    reset_geometry_hooks();
}

#[test]
fn task22o20_empty_anchor_source_still_runs_offset_then_intersection() {
    reset_geometry_hooks();
    let mut current = cache(0).unwrap();
    current.top_surfaces.clear();
    let caches = vec![Some(current), cache(0)];
    let layers = vec![layer(0, 0.2, 0.2), layer(1, 0.2, 0.4)];
    let lslices = vec![lslice(-100, 100); 2];
    let mut options = options();
    options.top_shell_layers.0 = 1;
    options.top_shell_thickness.0 = 0.0;
    options.bottom_shell_layers.0 = 0;

    let output =
        gather::project_record(0, projection_input(&caches, &layers, &lslices, &options, 5))
            .unwrap();
    assert!(output.shell.is_empty());
    assert_eq!(
        geometry_events(),
        [
            GeometryStep::TopAnchorOffset,
            GeometryStep::TopAnchorIntersection,
        ]
    );
}

#[test]
fn task22o20_acute_anchor_freezes_miter_limit_three_not_two() {
    reset_geometry_hooks();
    let triangle = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(1_000_000, 0),
        Point::new(500_000, 500_000),
    ]);
    let mut current = cache(0).unwrap();
    current.top_surfaces = vec![triangle.clone()];
    let caches = vec![Some(current), cache(0)];
    let layers = vec![layer(0, 0.2, 0.2), layer(1, 0.2, 0.4)];
    let lslices = vec![lslice(-2_000_000, 2_000_000); 2];
    let mut options = options();
    options.top_shell_layers.0 = 1;
    options.top_shell_thickness.0 = 0.0;
    options.bottom_shell_layers.0 = 0;

    let expected = offset_paths(
        std::slice::from_ref(&triangle),
        100_000.0,
        JoinType::Miter,
        3.0,
    )
    .unwrap();
    assert_eq!(
        expected,
        [Polygon::new(vec![
            Point::new(500_000, 641_421),
            Point::new(-241_421, -100_000),
            Point::new(1_241_421, -100_000),
        ])]
    );
    let limit_two = offset_paths(&[triangle], 100_000.0, JoinType::Miter, 2.0).unwrap();
    assert_ne!(expected, limit_two);
    let output = gather::project_record(
        0,
        projection_input(&caches, &layers, &lslices, &options, 100_000),
    )
    .unwrap();
    assert_eq!(output.shell, expected);
}

fn x_span(path: &Polygon) -> i64 {
    let (minimum, maximum) = path
        .points()
        .iter()
        .map(|point| point.x())
        .fold((i64::MAX, i64::MIN), |(minimum, maximum), x| {
            (minimum.min(x), maximum.max(x))
        });
    maximum - minimum
}
