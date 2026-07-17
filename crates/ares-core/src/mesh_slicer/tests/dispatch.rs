use crate::{SliceError, geometry::Coord};

use super::super::{
    IntersectionLine, IntersectionPoint, MeshPlaneInput, RawIntersectionBudget,
    dispatch_mesh_on_planes_with, eligible_plane_range, slice_mesh_on_planes,
};

const RAW_INTERSECTION_LIMIT: usize = 1_000_000;
const RAW_INTERSECTION_LIMIT_ERROR: &str =
    "project raw intersection count exceeds supported limit of 1000000";

fn endpoint_coordinates(endpoint: IntersectionPoint) -> (Coord, Coord) {
    let point = endpoint.point();
    (point.x(), point.y())
}

fn line_coordinates(line: IntersectionLine) -> ((Coord, Coord), (Coord, Coord)) {
    (
        endpoint_coordinates(line.a()),
        endpoint_coordinates(line.b()),
    )
}

fn one_line_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>, Vec<f32>) {
    (
        vec![[0.0, 0.0, 0.0], [4.0, 0.0, 2.0], [0.0, 4.0, 2.0]],
        vec![[0, 1, 2]],
        vec![1.0],
    )
}

fn limit_error() -> SliceError {
    SliceError::InvalidInput(RAW_INTERSECTION_LIMIT_ERROR.to_owned())
}

#[test]
fn task22b_multi_plane_dispatch_preserves_boundaries_duplicates_and_empty_slots() {
    let vertices = [[0.0, 0.0, -1.0], [4.0, 0.0, 1.0], [0.0, 4.0, 1.0]];
    let triangles = [[0, 1, 2]];
    let planes = [-2.0, -1.0, 0.0, 0.0, 1.0, 1.0, 2.0];
    let mut trace = Vec::new();
    let mut traced_lines = vec![Vec::new(); planes.len()];
    let mut trace_budget = RawIntersectionBudget::new();

    dispatch_mesh_on_planes_with(
        MeshPlaneInput::new(&vertices, &triangles, &planes),
        &mut trace_budget,
        |face_index, plane_index| trace.push((face_index, plane_index)),
        |plane_index, line| traced_lines[plane_index].push(line),
    )
    .unwrap();

    assert_eq!(eligible_plane_range(&planes, -1.0, 1.0), 1..6);
    assert_eq!(trace, vec![(0, 1), (0, 2), (0, 3), (0, 4), (0, 5)]);
    assert_eq!(
        traced_lines.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![0, 0, 1, 1, 1, 1, 0]
    );
    assert_eq!(traced_lines[2], traced_lines[3]);
    assert_eq!(traced_lines[4], traced_lines[5]);
    assert_eq!(trace_budget.retained_lines, 4);

    let mut wrapper_budget = RawIntersectionBudget { retained_lines: 7 };
    assert_eq!(
        slice_mesh_on_planes(&vertices, &triangles, &planes, &mut wrapper_budget).unwrap(),
        traced_lines
    );
    assert_eq!(wrapper_budget.retained_lines, 11);

    let horizontal_vertices = [[0.0, 0.0, 1.0], [2.0, 0.0, 1.0], [0.0, 2.0, 1.0]];
    let horizontal_triangles = [[0, 1, 2]];
    let horizontal_planes = [1.0];
    let mut horizontal_budget = RawIntersectionBudget::new();
    let mut horizontal_trace = Vec::new();
    let mut horizontal_lines = vec![Vec::new()];
    dispatch_mesh_on_planes_with(
        MeshPlaneInput::new(
            &horizontal_vertices,
            &horizontal_triangles,
            &horizontal_planes,
        ),
        &mut horizontal_budget,
        |face_index, plane_index| horizontal_trace.push((face_index, plane_index)),
        |plane_index, line| horizontal_lines[plane_index].push(line),
    )
    .unwrap();
    assert!(horizontal_trace.is_empty());
    assert_eq!(horizontal_budget.retained_lines, 0);
    assert_eq!(horizontal_lines, vec![Vec::new()]);
}

#[test]
fn task22b_multi_plane_dispatch_is_face_major_then_eligible_plane_major() {
    let vertices = [
        [100.0, 0.0, 1.0],
        [120.0, 0.0, 2.0],
        [100.0, 20.0, 2.0],
        [0.0, 0.0, 0.0],
        [20.0, 0.0, 3.0],
        [0.0, 20.0, 3.0],
    ];
    let triangles = [[0, 1, 2], [3, 4, 5]];
    let planes = [0.0, 1.0, 1.5, 2.0, 3.0];
    let mut trace = Vec::new();
    let mut lines = vec![Vec::new(); planes.len()];
    let mut budget = RawIntersectionBudget::new();

    dispatch_mesh_on_planes_with(
        MeshPlaneInput::new(&vertices, &triangles, &planes),
        &mut budget,
        |face_index, plane_index| trace.push((face_index, plane_index)),
        |plane_index, line| lines[plane_index].push(line),
    )
    .unwrap();

    assert_eq!(
        trace,
        vec![
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0),
            (1, 1),
            (1, 2),
            (1, 3),
            (1, 4),
        ]
    );
    assert_eq!(
        lines[2]
            .iter()
            .copied()
            .map(line_coordinates)
            .collect::<Vec<_>>(),
        vec![((100, 10), (110, 0)), ((0, 10), (10, 0))]
    );

    let mut wrapper_budget = RawIntersectionBudget::new();
    assert_eq!(
        slice_mesh_on_planes(&vertices, &triangles, &planes, &mut wrapper_budget).unwrap(),
        lines
    );
}

#[test]
fn task22b_raw_line_budget_claims_before_append_and_checks_limit_or_overflow() {
    let mut exact_limit = RawIntersectionBudget {
        retained_lines: RAW_INTERSECTION_LIMIT - 1,
    };
    assert_eq!(exact_limit.claim(1), Ok(()));
    assert_eq!(exact_limit.retained_lines, RAW_INTERSECTION_LIMIT);
    assert_eq!(exact_limit.claim(1), Err(limit_error()));
    assert_eq!(exact_limit.retained_lines, RAW_INTERSECTION_LIMIT);

    let mut summed_limit = RawIntersectionBudget {
        retained_lines: RAW_INTERSECTION_LIMIT - 1,
    };
    assert_eq!(summed_limit.claim(2), Err(limit_error()));
    assert_eq!(summed_limit.retained_lines, RAW_INTERSECTION_LIMIT - 1);

    let mut overflow = RawIntersectionBudget { retained_lines: 1 };
    assert_eq!(overflow.claim(usize::MAX), Err(limit_error()));
    assert_eq!(overflow.retained_lines, 1);

    let (vertices, triangles, planes) = one_line_mesh();
    let mut shared = RawIntersectionBudget {
        retained_lines: RAW_INTERSECTION_LIMIT - 1,
    };
    let mut appended = Vec::new();
    dispatch_mesh_on_planes_with(
        MeshPlaneInput::new(&vertices, &triangles, &planes),
        &mut shared,
        |_, _| {},
        |plane_index, line| appended.push((plane_index, line)),
    )
    .unwrap();
    assert_eq!(shared.retained_lines, RAW_INTERSECTION_LIMIT);
    assert_eq!(appended.len(), 1);

    assert_eq!(
        dispatch_mesh_on_planes_with(
            MeshPlaneInput::new(&vertices, &triangles, &planes),
            &mut shared,
            |_, _| {},
            |plane_index, line| appended.push((plane_index, line)),
        ),
        Err(limit_error())
    );
    assert_eq!(shared.retained_lines, RAW_INTERSECTION_LIMIT);
    assert_eq!(appended.len(), 1);
}

#[test]
fn task22b_multi_plane_slicing_is_repeatably_deterministic() {
    let vertices = [
        [8.0, 0.0, 0.0],
        [12.0, 0.0, 2.0],
        [8.0, 4.0, 2.0],
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 2.0],
        [0.0, 4.0, 2.0],
    ];
    let triangles = [[0, 1, 2], [3, 4, 5]];
    let planes = [0.0, 0.5, 1.0, 1.0, 2.0];
    let mut first_budget = RawIntersectionBudget::new();
    let mut second_budget = RawIntersectionBudget::new();

    let first = slice_mesh_on_planes(&vertices, &triangles, &planes, &mut first_budget).unwrap();
    let second = slice_mesh_on_planes(&vertices, &triangles, &planes, &mut second_budget).unwrap();

    assert_eq!(first, second);
    assert_eq!(first_budget.retained_lines, 8);
    assert_eq!(second_budget.retained_lines, 8);
    assert_eq!(
        first.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![0, 2, 2, 2, 2]
    );

    let nonmanifold_vertices = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 2.0],
        [0.0, 4.0, 2.0],
        [10.0, 0.0, 0.0],
        [14.0, 0.0, 2.0],
        [10.0, 4.0, 2.0],
        [10.0, 6.0, 2.0],
        [10.0, 8.0, 2.0],
    ];
    let nonmanifold_triangles = [[0, 1, 2], [3, 4, 5], [3, 4, 6], [3, 4, 7]];
    let nonmanifold_planes = [1.0];
    let mut untouched_budget = RawIntersectionBudget::new();
    let mut attempts = Vec::new();
    let mut output = Vec::new();
    assert_eq!(
        dispatch_mesh_on_planes_with(
            MeshPlaneInput::new(
                &nonmanifold_vertices,
                &nonmanifold_triangles,
                &nonmanifold_planes,
            ),
            &mut untouched_budget,
            |face_index, plane_index| attempts.push((face_index, plane_index)),
            |plane_index, line| output.push((plane_index, line)),
        ),
        Err(SliceError::UnsupportedProjectFeature(
            "mesh_topology".to_owned()
        ))
    );
    assert_eq!(untouched_budget.retained_lines, 0);
    assert!(attempts.is_empty());
    assert!(output.is_empty());
}
