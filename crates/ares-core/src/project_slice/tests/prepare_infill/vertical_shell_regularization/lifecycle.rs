use crate::project_slice::{
    prepare_infill::{
        surface_type_detection, vertical_shell_projection, vertical_shell_regularization,
        vertical_shell_trimming, vertical_shells,
    },
    tests::support::{KsrArchive, metadata},
};
use crate::{SliceError, slice_project};

#[test]
fn task22o22_inactive_typed_mode_produces_empty_sidecars_without_geometry() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
        "\"ensure_vertical_shell_thickness\": \"ensure_moderate\"",
    );
    vertical_shell_regularization::reset_geometry_hooks();
    let output = super::fixture::prepare(archive.bytes());
    assert!(output.regularizations.iter().all(|object| {
        object
            .records
            .iter()
            .flatten()
            .all(|record| record.regularized_shell.is_empty())
    }));
    assert!(vertical_shell_regularization::geometry_events().is_empty());
    vertical_shell_regularization::dispose(output);
    vertical_shell_regularization::reset_geometry_hooks();
}

#[tokio::test]
async fn task22o22_earlier_capability_errors_have_precedence() {
    for (from, to, expected) in [(
        "\"counterbore_hole_bridging\": \"none\"",
        "\"counterbore_hole_bridging\": \"partiallybridge\"",
        SliceError::UnsupportedProjectFeature("counterbore_hole_bridging".to_owned()),
    )] {
        let mut archive = KsrArchive::new();
        archive.replace_unique("Metadata/project_settings.config", from, to);
        vertical_shell_regularization::reset_invocations();
        assert_eq!(
            slice_project(archive.bytes(), metadata())
                .await
                .unwrap_err(),
            expected
        );
        assert_eq!(vertical_shell_regularization::invocations(), 0);
    }
    let (multi_region, _) = crate::project_slice::tests::region_fixture::modifier_projects();
    vertical_shell_regularization::reset_invocations();
    assert_eq!(
        slice_project(multi_region, metadata()).await.unwrap_err(),
        SliceError::UnsupportedProjectFeature("multi_region_layer_slices".to_owned())
    );
    assert_eq!(vertical_shell_regularization::invocations(), 0);
}

#[tokio::test]
async fn task22o22_o17_o19_o20_and_o21_failures_precede_regularization() {
    surface_type_detection::reset_geometry_hooks();
    surface_type_detection::fail_geometry_at(
        surface_type_detection::GeometryStep::TopSafetyDifference,
    );
    assert_precedes_o22("surface-type detection geometry is outside the supported Clipper range")
        .await;
    surface_type_detection::reset_geometry_hooks();

    for step in [
        vertical_shells::GeometryStep::Top,
        vertical_shells::GeometryStep::Bottom,
    ] {
        vertical_shells::reset_geometry_hooks();
        vertical_shells::fail_geometry_at(step);
        assert_precedes_o22("vertical-shell cache geometry is outside the supported Clipper range")
            .await;
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
        assert_precedes_o22(
            "vertical-shell projection geometry is outside the supported Clipper range",
        )
        .await;
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
        assert_precedes_o22(
            "vertical-shell internal trimming geometry is outside the supported Clipper range",
        )
        .await;
        vertical_shell_trimming::reset_geometry_hooks();
    }
}

#[tokio::test]
async fn task22o22_each_o20_anchor_failure_precedes_regularization() {
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
        vertical_shell_regularization::reset_invocations();
        assert!(slice_project(archive.bytes(), metadata()).await.is_err());
        assert_eq!(vertical_shell_regularization::invocations(), 0);
        vertical_shell_projection::reset_geometry_hooks();
    }
}

async fn assert_precedes_o22(expected: &str) {
    vertical_shell_regularization::reset_invocations();
    assert!(matches!(
        slice_project(KsrArchive::new().bytes(), metadata()).await,
        Err(SliceError::InvalidInput(message)) if message == expected
    ));
    assert_eq!(vertical_shell_regularization::invocations(), 0);
}
