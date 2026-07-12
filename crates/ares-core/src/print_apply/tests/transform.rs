use super::super::transform_state::{
    StagedTransform3d, StagedTransform3f, staged_trafo_for_bbox,
    staged_trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only,
};

#[test]
fn trafo_for_bbox_identity_returns_f32_identity() {
    let result = staged_trafo_for_bbox(
        &StagedTransform3d::identity(),
        &StagedTransform3d::identity(),
    );

    assert_eq!(
        result.rows(),
        &[
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    );
}

#[test]
fn trafo_for_bbox_multiplies_object_then_volume_and_zeros_xy_translation() {
    let object = StagedTransform3d::from_rows([
        [2.0, 0.0, 0.0, 10.0],
        [0.0, 3.0, 0.0, 20.0],
        [0.0, 0.0, 4.0, 30.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let volume = StagedTransform3d::from_rows([
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 2.0],
        [0.0, 0.0, 1.0, 3.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    let result = staged_trafo_for_bbox(&object, &volume);

    assert_eq!(result.rows()[0][3], 0.0);
    assert_eq!(result.rows()[1][3], 0.0);
    assert_eq!(result.rows()[2][3], 42.0);
}

#[test]
fn trafo_for_bbox_uses_non_commutative_object_then_volume_order() {
    let object = StagedTransform3d::from_rows([
        [2.0, 1.0, 0.0, 0.0],
        [0.0, 3.0, 0.0, 0.0],
        [0.0, 0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let volume = StagedTransform3d::from_rows([
        [1.0, 0.0, 0.0, 5.0],
        [0.0, 1.0, 0.0, 7.0],
        [0.0, 0.0, 1.0, 11.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    let result = staged_trafo_for_bbox(&object, &volume);

    assert_eq!(result.rows()[2][3], 44.0);
}

#[test]
fn trafo_for_bbox_preserves_composed_linear_terms() {
    let object = StagedTransform3d::from_rows([
        [2.0, 1.0, 0.0, 0.0],
        [0.0, 3.0, 0.0, 0.0],
        [0.0, 0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let volume = StagedTransform3d::from_rows([
        [5.0, 0.0, 0.0, 0.0],
        [0.0, 7.0, 0.0, 0.0],
        [0.0, 0.0, 11.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    let result = staged_trafo_for_bbox(&object, &volume);

    assert_eq!(result.rows()[0][0], 10.0);
    assert_eq!(result.rows()[0][1], 7.0);
    assert_eq!(result.rows()[1][1], 21.0);
    assert_eq!(result.rows()[2][2], 44.0);
}

#[test]
fn trafo_for_bbox_returns_f32_rows() {
    let object = StagedTransform3d::from_rows([
        [1.0 / 3.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let volume = StagedTransform3d::identity();

    let result: StagedTransform3f = staged_trafo_for_bbox(&object, &volume);

    assert_eq!(result.rows()[0][0], (1.0_f64 / 3.0) as f32);
}

fn transform(rows: [[f64; 4]; 4]) -> StagedTransform3d {
    StagedTransform3d::from_rows(rows)
}

fn translate_z(z: f64) -> StagedTransform3d {
    transform([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, z],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

#[test]
fn transform_z_rotation_mirroring_predicate_accepts_identity() {
    let t = StagedTransform3d::identity();

    assert!(staged_trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(&t, &t));
}

#[test]
fn transform_z_rotation_mirroring_predicate_accepts_z_rotation() {
    let rotated = transform([
        [0.0, -1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    assert!(
        staged_trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(
            &rotated,
            &StagedTransform3d::identity(),
        )
    );
}

#[test]
fn transform_z_rotation_mirroring_predicate_accepts_xy_mirroring() {
    let mirrored = transform([
        [-1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    assert!(
        staged_trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(
            &mirrored,
            &StagedTransform3d::identity(),
        )
    );
}

#[test]
fn transform_z_rotation_mirroring_predicate_rejects_z_translation_mismatch() {
    assert!(
        !staged_trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(
            &translate_z(0.0),
            &translate_z(0.0002),
        )
    );
}

#[test]
fn transform_z_rotation_mirroring_predicate_rejects_tilted_z_column() {
    let tilted = transform([
        [1.0, 0.0, 0.01, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    assert!(
        !staged_trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(
            &tilted,
            &StagedTransform3d::identity(),
        )
    );
}

#[test]
fn transform_z_rotation_mirroring_predicate_rejects_xy_column_z_components() {
    let tilted_x = transform([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.01, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    assert!(
        !staged_trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(
            &tilted_x,
            &StagedTransform3d::identity(),
        )
    );
}

#[test]
fn transform_z_rotation_mirroring_predicate_rejects_non_unit_scale() {
    let scaled = transform([
        [2.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    assert!(
        !staged_trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(
            &scaled,
            &StagedTransform3d::identity(),
        )
    );
}

#[test]
fn transform_z_rotation_mirroring_predicate_rejects_non_perpendicular_xy_columns() {
    let skewed = transform([
        [1.0, 0.5, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    assert!(
        !staged_trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(
            &skewed,
            &StagedTransform3d::identity(),
        )
    );
}
