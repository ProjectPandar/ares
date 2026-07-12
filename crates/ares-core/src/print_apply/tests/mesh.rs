use super::super::mesh_state::{
    StagedIndexedTriangleSet, StagedLayerHeightRange, staged_transformed_its_bbox2d,
    staged_transformed_its_bboxes_in_z_ranges,
};
use super::super::transform_state::StagedTransform3f;

fn identity_transform() -> StagedTransform3f {
    StagedTransform3f::from_rows([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

#[test]
fn transformed_its_bbox2d_expands_single_triangle_by_offset_and_epsilon() {
    let its = StagedIndexedTriangleSet::new(
        vec![[1.0, 2.0, 3.0], [4.0, -1.0, 5.0], [-2.0, 6.0, -1.0]],
        vec![[0, 1, 2]],
    );

    let bbox = staged_transformed_its_bbox2d(&its, &identity_transform(), 0.5);

    assert_eq!(bbox.min(), [-2.5, -1.5, -1.0001]);
    assert_eq!(bbox.max(), [4.5, 6.5, 5.0001]);
}

#[test]
fn transformed_its_bbox2d_extends_across_multiple_triangles() {
    let its = StagedIndexedTriangleSet::new(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [-3.0, 4.0, -5.0],
        ],
        vec![[0, 1, 2], [1, 2, 3]],
    );

    let bbox = staged_transformed_its_bbox2d(&its, &identity_transform(), 0.0);

    assert_eq!(bbox.min(), [-3.0, 0.0, -5.0001]);
    assert_eq!(bbox.max(), [2.0, 4.0, 2.0001]);
}

#[test]
fn transformed_its_bbox2d_applies_transform_before_extending() {
    let its = StagedIndexedTriangleSet::new(
        vec![[1.0, 2.0, 3.0], [2.0, 3.0, 4.0], [3.0, 4.0, 5.0]],
        vec![[0, 1, 2]],
    );
    let transform = StagedTransform3f::from_rows([
        [2.0, 0.0, 0.0, 10.0],
        [0.0, 3.0, 0.0, 20.0],
        [0.0, 0.0, 4.0, 30.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    let bbox = staged_transformed_its_bbox2d(&its, &transform, 0.0);

    assert_eq!(bbox.min(), [12.0, 26.0, 42.0 - 0.0001]);
    assert_eq!(bbox.max(), [16.0, 32.0, 50.0 + 0.0001]);
}

#[test]
#[should_panic]
fn transformed_its_bbox2d_panics_for_empty_indices() {
    let its = StagedIndexedTriangleSet::new(vec![[0.0, 0.0, 0.0]], Vec::new());

    staged_transformed_its_bbox2d(&its, &identity_transform(), 0.0);
}

fn range(start: f64, end: f64) -> StagedLayerHeightRange {
    StagedLayerHeightRange::new(start, end)
}

fn vertical_triangle() -> StagedIndexedTriangleSet {
    StagedIndexedTriangleSet::new(
        vec![[0.0, 0.0, 0.0], [10.0, 0.0, 10.0], [0.0, 10.0, 10.0]],
        vec![[0, 1, 2]],
    )
}

#[test]
fn transformed_its_bboxes_in_z_ranges_returns_empty_for_empty_ranges() {
    let result = staged_transformed_its_bboxes_in_z_ranges(
        &vertical_triangle(),
        &identity_transform(),
        &[],
        0.0,
    );

    assert!(result.is_empty());
}

#[test]
fn transformed_its_bboxes_in_z_ranges_leaves_missed_range_unpopulated() {
    let result = staged_transformed_its_bboxes_in_z_ranges(
        &vertical_triangle(),
        &identity_transform(),
        &[range(11.0, 12.0)],
        0.0,
    );

    assert_eq!(result.len(), 1);
    assert!(!result[0].is_populated());
}

#[test]
fn transformed_its_bboxes_in_z_ranges_extends_edges_fully_inside_range() {
    let its = StagedIndexedTriangleSet::new(
        vec![[1.0, 2.0, 2.0], [3.0, 5.0, 4.0], [-2.0, 7.0, 6.0]],
        vec![[0, 1, 2]],
    );

    let result = staged_transformed_its_bboxes_in_z_ranges(
        &its,
        &identity_transform(),
        &[range(1.0, 7.0)],
        0.0,
    );

    assert!(result[0].is_populated());
    assert_eq!(result[0].min(), [-2.0, 2.0, 2.0 - 0.0001]);
    assert_eq!(result[0].max(), [3.0, 7.0, 6.0 + 0.0001]);
}

#[test]
fn transformed_its_bboxes_in_z_ranges_adds_lower_bound_crossing_and_upper_endpoint() {
    let its = StagedIndexedTriangleSet::new(
        vec![[0.0, 0.0, 0.0], [10.0, 0.0, 10.0], [10.0, 0.0, 10.0]],
        vec![[0, 1, 2]],
    );

    let result = staged_transformed_its_bboxes_in_z_ranges(
        &its,
        &identity_transform(),
        &[range(5.0, 12.0)],
        0.0,
    );

    assert!(result[0].is_populated());
    assert_eq!(result[0].min(), [5.0, 0.0, 5.0 - 0.0001]);
    assert_eq!(result[0].max(), [10.0, 0.0, 10.0 + 0.0001]);
}

#[test]
fn transformed_its_bboxes_in_z_ranges_adds_upper_bound_crossing_and_lower_endpoint() {
    let its = StagedIndexedTriangleSet::new(
        vec![[0.0, 0.0, 0.0], [10.0, 0.0, 10.0], [0.0, 0.0, 0.0]],
        vec![[0, 1, 2]],
    );

    let result = staged_transformed_its_bboxes_in_z_ranges(
        &its,
        &identity_transform(),
        &[range(-1.0, 5.0)],
        0.0,
    );

    assert!(result[0].is_populated());
    assert_eq!(result[0].min(), [0.0, 0.0, 0.0 - 0.0001]);
    assert_eq!(result[0].max(), [5.0, 0.0, 5.0 + 0.0001]);
}

#[test]
fn transformed_its_bboxes_in_z_ranges_adds_two_intersections_for_spanning_edge() {
    let its = StagedIndexedTriangleSet::new(
        vec![[0.0, 0.0, 0.0], [10.0, 0.0, 10.0], [0.0, 0.0, 0.0]],
        vec![[0, 1, 2]],
    );

    let result = staged_transformed_its_bboxes_in_z_ranges(
        &its,
        &identity_transform(),
        &[range(2.0, 8.0)],
        0.0,
    );

    assert!(result[0].is_populated());
    assert_eq!(result[0].min(), [2.0, 0.0, 2.0 - 0.0001]);
    assert_eq!(result[0].max(), [8.0, 0.0, 8.0 + 0.0001]);
}

#[test]
fn transformed_its_bboxes_in_z_ranges_preserves_independent_range_results() {
    let result = staged_transformed_its_bboxes_in_z_ranges(
        &vertical_triangle(),
        &identity_transform(),
        &[range(-1.0, 1.0), range(4.0, 6.0), range(11.0, 12.0)],
        0.0,
    );

    assert_eq!(result.len(), 3);
    assert!(result[0].is_populated());
    assert!(result[1].is_populated());
    assert!(!result[2].is_populated());
}

#[test]
fn transformed_its_bboxes_in_z_ranges_applies_transform_before_clipping() {
    let its = StagedIndexedTriangleSet::new(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]],
        vec![[0, 1, 2]],
    );
    let transform = StagedTransform3f::from_rows([
        [10.0, 0.0, 0.0, 1.0],
        [0.0, 10.0, 0.0, 2.0],
        [0.0, 0.0, 10.0, 3.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    let result =
        staged_transformed_its_bboxes_in_z_ranges(&its, &transform, &[range(8.0, 12.0)], 0.0);

    assert!(result[0].is_populated());
    assert_eq!(result[0].min(), [1.0, 2.0, 8.0 - 0.0001]);
    assert_eq!(result[0].max(), [10.0, 11.0, 12.0 + 0.0001]);
}

#[test]
fn transformed_its_bboxes_in_z_ranges_expands_populated_bbox_by_offset_and_epsilon() {
    let result = staged_transformed_its_bboxes_in_z_ranges(
        &vertical_triangle(),
        &identity_transform(),
        &[range(0.0, 10.0)],
        0.5,
    );

    assert!(result[0].is_populated());
    assert_eq!(result[0].min(), [-0.5, -0.5, -0.0001]);
    assert_eq!(result[0].max(), [10.5, 10.5, 10.0001]);
}
