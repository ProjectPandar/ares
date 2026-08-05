use crate::{
    SliceError,
    project_slice::{
        prepare_infill::{
            surface_type_detection, vertical_shell_filtering, vertical_shell_projection,
            vertical_shell_regularization, vertical_shell_trimming, vertical_shells,
        },
        tests::support::{KsrArchive, metadata},
    },
    slice_project,
};

#[tokio::test]
async fn task22o23_all_earlier_capability_errors_keep_filtering_uninvoked() {
    for (from, to, expected) in [
        (
            "\"spiral_mode\": \"0\"",
            "\"spiral_mode\": \"1\"",
            "spiral_mode",
        ),
        (
            "\"counterbore_hole_bridging\": \"none\"",
            "\"counterbore_hole_bridging\": \"partiallybridge\"",
            "counterbore_hole_bridging",
        ),
        (
            "\"interface_shells\": \"0\"",
            "\"interface_shells\": \"1\"",
            "interface_shells",
        ),
        (
            "\"enable_extra_bridge_layer\": \"disabled\"",
            "\"enable_extra_bridge_layer\": \"apply_to_all\"",
            "enable_extra_bridge_layer",
        ),
    ] {
        let mut archive = KsrArchive::new();
        archive.replace_unique("Metadata/project_settings.config", from, to);
        vertical_shell_filtering::reset_invocations();
        assert_eq!(
            slice_project(archive.bytes(), metadata())
                .await
                .unwrap_err(),
            SliceError::UnsupportedProjectFeature(expected.to_owned())
        );
        assert_eq!(vertical_shell_filtering::invocations(), 0);
    }
    let (multi_region, _) = crate::project_slice::tests::region_fixture::modifier_projects();
    vertical_shell_filtering::reset_invocations();
    assert_eq!(
        slice_project(multi_region, metadata()).await.unwrap_err(),
        SliceError::UnsupportedProjectFeature("multi_region_layer_slices".to_owned())
    );
    assert_eq!(vertical_shell_filtering::invocations(), 0);
}

#[tokio::test]
async fn task22o23_every_earlier_geometry_stage_precedes_filtering() {
    surface_type_detection::reset_geometry_hooks();
    surface_type_detection::fail_geometry_at(
        surface_type_detection::GeometryStep::TopSafetyDifference,
    );
    assert_precedes().await;
    surface_type_detection::reset_geometry_hooks();

    for step in [
        vertical_shells::GeometryStep::Top,
        vertical_shells::GeometryStep::Bottom,
    ] {
        vertical_shells::reset_geometry_hooks();
        vertical_shells::fail_geometry_at(step);
        assert_precedes().await;
        vertical_shells::reset_geometry_hooks();
    }
    for step in [
        vertical_shell_projection::GeometryStep::TopVisit,
        vertical_shell_projection::GeometryStep::BottomVisit,
        vertical_shell_projection::GeometryStep::HoleIntersection,
        vertical_shell_projection::GeometryStep::ShellUnion,
    ] {
        vertical_shell_projection::reset_geometry_hooks();
        vertical_shell_projection::fail_geometry_at(step);
        assert_precedes().await;
        vertical_shell_projection::reset_geometry_hooks();
    }
    for step in [
        vertical_shell_trimming::GeometryStep::SafetyOffset,
        vertical_shell_trimming::GeometryStep::SafetyIntersection,
        vertical_shell_trimming::GeometryStep::Difference,
        vertical_shell_trimming::GeometryStep::EmptyGate,
        vertical_shell_trimming::GeometryStep::SolidAppend,
    ] {
        vertical_shell_trimming::reset_geometry_hooks();
        vertical_shell_trimming::fail_geometry_at(step);
        assert_precedes().await;
        vertical_shell_trimming::reset_geometry_hooks();
    }
    for step in [
        vertical_shell_regularization::GeometryStep::Union,
        vertical_shell_regularization::GeometryStep::Offset2First,
        vertical_shell_regularization::GeometryStep::Offset2Second,
        vertical_shell_regularization::GeometryStep::Shrink,
    ] {
        vertical_shell_regularization::reset_geometry_hooks();
        vertical_shell_regularization::fail_geometry_at(step);
        assert_precedes().await;
        vertical_shell_regularization::reset_geometry_hooks();
    }
}

#[tokio::test]
async fn task22o23_each_projection_anchor_failure_precedes_filtering() {
    for (step, top) in [
        (
            vertical_shell_projection::GeometryStep::TopAnchorOffset,
            true,
        ),
        (
            vertical_shell_projection::GeometryStep::TopAnchorIntersection,
            true,
        ),
        (
            vertical_shell_projection::GeometryStep::BottomAnchorOffset,
            false,
        ),
        (
            vertical_shell_projection::GeometryStep::BottomAnchorIntersection,
            false,
        ),
    ] {
        let mut archive = KsrArchive::new();
        if top {
            archive.replace_unique(
                "Metadata/project_settings.config",
                "\"top_shell_layers\": \"5\"",
                "\"top_shell_layers\": \"1\"",
            );
            archive.replace_unique(
                "Metadata/project_settings.config",
                "\"top_shell_thickness\": \"1\"",
                "\"top_shell_thickness\": \"0\"",
            );
        } else {
            archive.replace_unique(
                "Metadata/project_settings.config",
                "\"bottom_shell_layers\": \"3\"",
                "\"bottom_shell_layers\": \"1\"",
            );
        }
        vertical_shell_projection::reset_geometry_hooks();
        vertical_shell_projection::fail_geometry_at(step);
        vertical_shell_filtering::reset_invocations();
        assert!(slice_project(archive.bytes(), metadata()).await.is_err());
        assert_eq!(vertical_shell_filtering::invocations(), 0);
        vertical_shell_projection::reset_geometry_hooks();
    }
}

async fn assert_precedes() {
    vertical_shell_filtering::reset_invocations();
    assert!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .is_err()
    );
    assert_eq!(vertical_shell_filtering::invocations(), 0);
}
