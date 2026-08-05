use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::{
            vertical_shell_filtering::{
                self, GeometryStep,
                filter::{RecordOperands, filter_record},
            },
            vertical_shell_regularization::types::VerticalShellRegularization,
            vertical_shell_trimming::types::VerticalShellTrim,
        },
        region_slices::RegionSurface,
    },
};

use super::{empty_record, expolygon, rectangle};

#[test]
fn task22o23_strict_thresholds_and_short_circuits_follow_source_order() {
    let cases = [
        (
            31_499_999,
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
                GeometryStep::ClosingShrink,
                GeometryStep::CandidateScan,
                GeometryStep::CandidateExpansion,
                GeometryStep::ProtectionDifference,
                GeometryStep::EmptyGate,
            ],
            0,
        ),
        (
            31_500_000,
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
                GeometryStep::ClosingShrink,
                GeometryStep::CandidateScan,
                GeometryStep::VisibilityDifference,
                GeometryStep::EmptyGate,
            ],
            1,
        ),
        (
            168_000_000,
            vec![
                GeometryStep::NeighborIntersection,
                GeometryStep::ClosingGrow,
                GeometryStep::ClosingShrink,
                GeometryStep::CandidateScan,
                GeometryStep::EmptyGate,
            ],
            1,
        ),
    ];
    for (width, expected_events, survivors) in cases {
        vertical_shell_filtering::reset_geometry_hooks();
        let output = run(vec![expolygon(0, 0, width, 1)], None, None).unwrap();
        assert_eq!(output.filtered_shell.len(), survivors);
        assert_eq!(vertical_shell_filtering::geometry_events(), expected_events);
    }
}

#[test]
fn task22o23_spacing_bits_change_the_threshold_branch_and_survivor() {
    let candidate = expolygon(0, 0, 31_500_000, 1);
    let run_spacing = |spacing| {
        filter_record(
            RecordOperands {
                trim: &VerticalShellTrim {
                    shell: vec![rectangle(0, 0, 1, 1)],
                },
                regularization: &VerticalShellRegularization {
                    regularized_shell: vec![candidate.clone()],
                },
                current: &empty_record(),
                previous_lslices: None,
                next_lslices: None,
            },
            spacing,
            CoordinateScale::Normal,
        )
        .unwrap()
        .filtered_shell
        .len()
    };
    assert_eq!(run_spacing(20), 1);
    assert_eq!(run_spacing(21), 0);
}

#[test]
fn task22o23_visibility_difference_distinguishes_wrapped_partial_disjoint_and_holed() {
    let candidate = expolygon(0, 0, 40_000_000, 1);
    let full = vec![expolygon(-10, -10, 40_000_010, 10)];
    let partial = vec![expolygon(-10, -10, 20_000_000, 10)];
    let disjoint = vec![expolygon(50_000_000, 0, 60_000_000, 10)];
    let hole = Polygon::new(vec![
        Point::new(-1, -1),
        Point::new(-1, 2),
        Point::new(40_000_001, 2),
        Point::new(40_000_001, -1),
    ]);
    let holed = vec![ExPolygon::new(
        rectangle(-10, -10, 40_000_010, 10),
        vec![hole],
    )];
    for (neighbors, survivors) in [(full, 0), (partial, 1), (disjoint, 1), (holed, 1)] {
        let output = run(vec![candidate.clone()], Some(&neighbors), Some(&neighbors)).unwrap();
        assert_eq!(output.filtered_shell.len(), survivors);
    }
}

#[test]
fn task22o23_signed_contour_plus_hole_area_drives_the_tiny_branch() {
    let contour = rectangle(0, 0, 40_000_000, 1);
    let hole = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(0, 1),
        Point::new(10_000_001, 1),
        Point::new(10_000_001, 0),
    ]);
    vertical_shell_filtering::reset_geometry_hooks();
    let output = run(vec![ExPolygon::new(contour, vec![hole])], None, None).unwrap();
    assert!(output.filtered_shell.is_empty());
    assert!(
        !vertical_shell_filtering::geometry_events().contains(&GeometryStep::VisibilityDifference)
    );
}

#[test]
fn task22o23_internal_protection_uses_literal_flat_path_count() {
    let covering = expolygon(-10, -10, 110, 110);
    let disjoint = expolygon(200, 0, 210, 10);
    let splitter = expolygon(45, -10, 55, 110);
    let mut current = empty_record();
    current
        .fill_surfaces
        .push(RegionSurface::internal(expolygon(0, 0, 100, 100)));
    let output = filter_record(
        RecordOperands {
            trim: &VerticalShellTrim {
                shell: vec![rectangle(0, 0, 1, 1)],
            },
            regularization: &VerticalShellRegularization {
                regularized_shell: vec![covering.clone(), disjoint, splitter],
            },
            current: &current,
            previous_lslices: None,
            next_lslices: None,
        },
        20,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(output.filtered_shell, vec![covering]);
}

#[test]
fn task22o23_interleaved_survivors_are_stable_and_deep_cloned() {
    let removed = expolygon(0, 0, 1, 1);
    let first = expolygon(10, 0, 200_000_010, 1);
    let second = expolygon(20, 2, 200_000_020, 3);
    let first_points = first.contour().points().as_ptr();
    let second_points = second.contour().points().as_ptr();
    let output = run(
        vec![
            removed,
            first.clone(),
            expolygon(0, 4, 2, 5),
            second.clone(),
        ],
        None,
        None,
    )
    .unwrap();
    assert_eq!(output.filtered_shell, vec![first, second]);
    assert_ne!(
        output.filtered_shell[0].contour().points().as_ptr(),
        first_points
    );
    assert_ne!(
        output.filtered_shell[1].contour().points().as_ptr(),
        second_points
    );
}

fn run(
    candidates: Vec<ExPolygon>,
    previous: Option<&[ExPolygon]>,
    next: Option<&[ExPolygon]>,
) -> Result<
    crate::project_slice::prepare_infill::vertical_shell_filtering::types::VerticalShellTinyFilter,
    crate::SliceError,
> {
    filter_record(
        RecordOperands {
            trim: &VerticalShellTrim {
                shell: vec![rectangle(0, 0, 1, 1)],
            },
            regularization: &VerticalShellRegularization {
                regularized_shell: candidates,
            },
            current: &empty_record(),
            previous_lslices: previous,
            next_lslices: next,
        },
        20,
        CoordinateScale::Normal,
    )
}
