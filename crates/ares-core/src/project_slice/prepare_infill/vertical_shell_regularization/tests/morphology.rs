use crate::geometry::{ExPolygon, Point, Polygon};
use crate::project_slice::prepare_infill::{
    vertical_shell_regularization::{
        GeometryStep, geometry_events, regularize, reset_geometry_hooks,
    },
    vertical_shell_trimming::types::VerticalShellTrim,
};

#[test]
fn task22o22_o21_empty_gate_skips_all_regularization_geometry() {
    reset_geometry_hooks();
    let output =
        regularize::regularize_record(&VerticalShellTrim { shell: Vec::new() }, 400_000).unwrap();
    assert!(output.regularized_shell.is_empty());
    assert!(geometry_events().is_empty());
}

#[test]
fn task22o22_nonempty_shell_runs_exact_nested_source_order() {
    let shell = vec![super::rectangle(0, 0, 4_000_000, 4_000_000)];
    let input_points = shell[0].points().as_ptr();
    reset_geometry_hooks();
    let output = regularize::regularize_record(&VerticalShellTrim { shell }, 400_000).unwrap();
    assert_eq!(
        geometry_events(),
        vec![
            GeometryStep::Union,
            GeometryStep::Offset2First,
            GeometryStep::Offset2Second,
            GeometryStep::Shrink,
        ]
    );
    assert!(!output.regularized_shell.is_empty());
    assert_ne!(
        output.regularized_shell[0].contour().points().as_ptr(),
        input_points
    );
}

#[test]
fn task22o22_square_morphology_closes_gap_with_exact_order_and_fresh_storage() {
    let trim = VerticalShellTrim {
        shell: vec![
            super::rectangle(0, 0, 400, 400),
            super::rectangle(500, 0, 900, 400),
        ],
    };
    let input_outer = trim.shell.as_ptr() as usize;
    let input_points = trim
        .shell
        .iter()
        .map(|path| path.points().as_ptr() as usize)
        .collect::<Vec<_>>();
    let output = regularize::regularize_record(&trim, 100).unwrap();
    assert_eq!(
        output.regularized_shell,
        vec![ExPolygon::new(
            Polygon::new(vec![
                Point::new(433, 23),
                Point::new(467, 23),
                Point::new(512, -21),
                Point::new(888, -21),
                Point::new(921, 12),
                Point::new(921, 388),
                Point::new(888, 421),
                Point::new(512, 421),
                Point::new(467, 377),
                Point::new(433, 377),
                Point::new(388, 421),
                Point::new(12, 421),
                Point::new(-21, 388),
                Point::new(-21, 12),
                Point::new(12, -21),
                Point::new(388, -21),
            ]),
            Vec::new(),
        )]
    );
    assert_ne!(output.regularized_shell.as_ptr() as usize, input_outer);
    for expolygon in &output.regularized_shell {
        assert!(!input_points.contains(&(expolygon.contour().points().as_ptr() as usize)));
        for hole in expolygon.holes() {
            assert!(!input_points.contains(&(hole.points().as_ptr() as usize)));
        }
    }
}

#[test]
fn task22o22_touching_polygons_union_before_exact_square_morphology() {
    let output = regularize::regularize_record(
        &VerticalShellTrim {
            shell: vec![
                super::rectangle(0, 0, 400, 400),
                super::rectangle(400, 0, 800, 400),
            ],
        },
        100,
    )
    .unwrap();
    assert_eq!(
        output.regularized_shell,
        vec![ExPolygon::new(
            Polygon::new(vec![
                Point::new(821, 12),
                Point::new(821, 388),
                Point::new(788, 421),
                Point::new(12, 421),
                Point::new(-21, 388),
                Point::new(-21, 12),
                Point::new(12, -21),
                Point::new(788, -21),
            ]),
            Vec::new(),
        )]
    );
}

#[test]
fn task22o22_union_and_square_offsets_preserve_exact_hole_topology_order() {
    let mut hole = super::rectangle(300, 300, 700, 700);
    hole.reverse();
    let output = regularize::regularize_record(
        &VerticalShellTrim {
            shell: vec![super::rectangle(0, 0, 1_000, 1_000), hole],
        },
        100,
    )
    .unwrap();
    assert_eq!(
        output.regularized_shell,
        vec![ExPolygon::new(
            Polygon::new(vec![
                Point::new(1_021, 12),
                Point::new(1_021, 988),
                Point::new(988, 1_021),
                Point::new(12, 1_021),
                Point::new(-21, 988),
                Point::new(-21, 12),
                Point::new(12, -21),
                Point::new(988, -21),
            ]),
            vec![Polygon::new(vec![
                Point::new(346, 321),
                Point::new(321, 346),
                Point::new(321, 654),
                Point::new(346, 679),
                Point::new(654, 679),
                Point::new(679, 654),
                Point::new(679, 346),
                Point::new(654, 321),
            ])],
        )]
    );
}

#[test]
fn task22o22_opening_removes_material_narrower_than_the_ensure_diameter() {
    let output = regularize::regularize_record(
        &VerticalShellTrim {
            shell: vec![super::rectangle(0, 0, 60, 400)],
        },
        100,
    )
    .unwrap();
    assert!(output.regularized_shell.is_empty());
}

#[test]
fn task22o22_empty_union_still_reaches_both_offsets_and_shrink() {
    let contour = super::rectangle(0, 0, 4_000_000, 4_000_000);
    let mut reversed = contour.clone();
    reversed.reverse();
    reset_geometry_hooks();
    let output = regularize::regularize_record(
        &VerticalShellTrim {
            shell: vec![contour, reversed],
        },
        400_000,
    )
    .unwrap();
    assert!(output.regularized_shell.is_empty());
    assert_eq!(
        geometry_events(),
        vec![
            GeometryStep::Union,
            GeometryStep::Offset2First,
            GeometryStep::Offset2Second,
            GeometryStep::Shrink,
        ]
    );
}
