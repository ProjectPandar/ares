use crate::{ProjectVolumeType, Transform3d, slice_project};

use super::{
    raw_support::{
        bfs_restart_request, exact_dense_object, ordinal_gap_object, preflight_order_scenarios,
        ranged_later_request, request_wide_dense_objects, retained_facts,
    },
    support::{
        KsrArchive, identity_resolved, metadata, object, plan, project, project_volume,
        resolved_object, slot_limit, transform, unsupported,
    },
};
use crate::project_slice::raw_intersections::validate_dense_slot_counts;

#[test]
fn task22b_volume_ordinals_follow_nonempty_bfs_order_and_keep_filter_gaps() {
    use ProjectVolumeType::{ModelPart, NegativeVolume, ParameterModifier};

    let source = ordinal_gap_object();

    let projected = project(
        std::slice::from_ref(&source),
        &[identity_resolved(0)],
        vec![plan(0, 0, 2)],
    )
    .expect("supported projection");
    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].layers.len(), 2);
    assert_eq!(
        retained_facts(&projected[0], &source),
        [
            (2, 7, ModelPart),
            (3, 6, ParameterModifier),
            (5, 4, NegativeVolume),
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn task22b_volume_ordinals_distinguish_bfs_from_dfs_and_restart_per_object_request() {
    let (objects, resolved, plans) = bfs_restart_request();

    let assert_projection =
        |projected: &[crate::project_slice::raw_intersections::ProjectedPrintObject]| {
            assert_eq!(
                retained_facts(&projected[0], &objects[0]),
                [
                    (1, 3, ProjectVolumeType::ModelPart),
                    (2, 1, ProjectVolumeType::ModelPart),
                    (3, 2, ProjectVolumeType::ModelPart),
                ]
            );
            assert_eq!(
                retained_facts(&projected[1], &objects[1]),
                [(1, 99, ProjectVolumeType::NegativeVolume)]
            );
        };

    assert_projection(&project(&objects, &resolved, plans.clone()).unwrap());
    assert_projection(&project(&objects, &resolved, plans.clone()).unwrap());

    let left_plans = plans.clone();
    let left = async {
        tokio::task::yield_now().await;
        project(&objects, &resolved, left_plans).unwrap()
    };
    let right = async {
        tokio::task::yield_now().await;
        project(&objects, &resolved, plans).unwrap()
    };
    let (left, right) = tokio::join!(left, right);
    assert_projection(&left);
    assert_projection(&right);
}

#[test]
fn task22b_empty_occurrences_preserve_retained_volume_facts() {
    let accepted = object(
        "accepted.model",
        1,
        vec![
            project_volume("empty-a.model", 7, ProjectVolumeType::ModelPart, false),
            project_volume("empty-b.model", 7, ProjectVolumeType::NegativeVolume, false),
            project_volume("full.model", 8, ProjectVolumeType::ModelPart, true),
        ],
        &[Transform3d::IDENTITY],
    );
    let projected = project(
        std::slice::from_ref(&accepted),
        &[identity_resolved(0)],
        vec![plan(0, 0, 1)],
    )
    .unwrap();
    assert_eq!(
        retained_facts(&projected[0], &accepted),
        [(1, 8, ProjectVolumeType::ModelPart)]
    );
}

#[tokio::test]
async fn task22b_layer_range_preflight_runs_after_task22a_and_before_all_raw_geometry() {
    let mut benign = KsrArchive::new();
    benign.insert_text(
        "Metadata/layer_config_ranges.xml",
        r#"<objects><object id="1"><range min_z="0" max_z="1"><option opt_key="extruder">1</option></range></object></objects>"#,
    );
    assert_eq!(
        slice_project(benign.bytes(), metadata(), crate::SlicerContext::new(0))
            .await
            .unwrap_err(),
        unsupported("layer_config_ranges")
    );

    let (objects, resolved, plans) = ranged_later_request();
    assert_eq!(
        project(&objects, &resolved, plans).unwrap_err(),
        unsupported("layer_config_ranges")
    );
}

#[test]
fn task22b_print_object_centering_controls_accept_collapsed_xy_and_z_compensation() {
    let z = transform("1 0 0 0 1 0 0 0 1 0 0 3");
    let xyz = transform("1 0 0 0 1 0 0 0 1 21 -8 3");
    let source = object(
        "center.model",
        1,
        vec![project_volume(
            "center.model",
            1,
            ProjectVolumeType::ModelPart,
            true,
        )],
        &[z, xyz],
    );
    assert!(
        project(
            std::slice::from_ref(&source),
            &[resolved_object(0, &[z])],
            vec![plan(0, 0, 1)]
        )
        .is_ok()
    );

    // `fixed_xy_equal` ignores the Z row (filament Z-shrink compensation),
    // so a Z-only mismatch no longer rejects the gate.
    assert!(
        project(
            std::slice::from_ref(&source),
            &[identity_resolved(0)],
            vec![plan(0, 0, 1)],
        )
        .is_ok()
    );
}

#[test]
fn task22b_dense_slot_budget_counts_only_nonempty_sliceable_volumes_request_wide() {
    use ProjectVolumeType::{
        ModelPart, NegativeVolume, ParameterModifier, SupportBlocker, SupportEnforcer,
    };
    let first = object(
        "first.model",
        1,
        vec![
            project_volume("first.model", 1, ModelPart, true),
            project_volume("first.model", 2, NegativeVolume, true),
            project_volume("first.model", 3, ParameterModifier, true),
            project_volume("first.model", 4, ModelPart, false),
            project_volume("first.model", 5, SupportBlocker, true),
            project_volume("first.model", 6, SupportEnforcer, true),
        ],
        &[Transform3d::IDENTITY],
    );
    let second = object(
        "second.model",
        2,
        vec![project_volume("second.model", 7, ModelPart, true)],
        &[Transform3d::IDENTITY],
    );
    let projected = project(
        &[first, second],
        &[identity_resolved(0), identity_resolved(1)],
        vec![plan(0, 0, 2), plan(1, 0, 3)],
    )
    .unwrap();
    assert_eq!(projected[0].volumes().len(), 3);
    assert_eq!(projected[1].volumes().len(), 1);
    assert_eq!(validate_dense_slot_counts(&[(2, 3), (3, 1)]).unwrap(), 9);
    assert_eq!(
        validate_dense_slot_counts(&[(500_000, 1), (500_001, 1)]).unwrap_err(),
        slot_limit()
    );

    let exact = exact_dense_object();
    assert_eq!(exact.instances().len(), 2);
    assert!(project(&[exact], &[identity_resolved(0)], vec![plan(0, 0, 100_000)],).is_ok());

    let request_wide = request_wide_dense_objects();
    assert_eq!(
        project(
            &request_wide,
            &[identity_resolved(0), identity_resolved(1)],
            vec![plan(0, 0, 50_000), plan(1, 0, 50_000)],
        )
        .unwrap_err(),
        slot_limit()
    );
}

#[test]
fn task22b_dense_slot_budget_accepts_exact_limit_and_rejects_next_or_overflow() {
    assert_eq!(
        validate_dense_slot_counts(&[(40_000, 10), (60_000, 10)]).unwrap(),
        1_000_000
    );
    assert_eq!(
        validate_dense_slot_counts(&[(50_000, 10), (50_000, 11)]).unwrap_err(),
        slot_limit()
    );
    assert_eq!(
        validate_dense_slot_counts(&[(100_000, usize::MAX)]).unwrap_err(),
        slot_limit()
    );
    assert_eq!(
        validate_dense_slot_counts(&[(usize::MAX, 1), (1, 1)]).unwrap_err(),
        slot_limit()
    );
}

#[test]
fn task22b_raw_preflight_order_is_range_centering_shared_then_dense_slots() {
    for (objects, resolved, plans, expected) in preflight_order_scenarios() {
        assert_eq!(project(&objects, &resolved, plans).unwrap_err(), expected);
    }
}
