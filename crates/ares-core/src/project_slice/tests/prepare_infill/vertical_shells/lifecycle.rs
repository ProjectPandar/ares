use crate::project_slice::{
    prepare_infill::{surface_type_detection, vertical_shells},
    tests::support::{KsrArchive, metadata},
};
use crate::{SliceError, slice_project};

#[tokio::test]
async fn task22o19_public_lifecycle_runs_once_and_remains_incomplete() {
    vertical_shells::reset_invocations();
    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(vertical_shells::invocations(), 1);
}

#[tokio::test]
async fn task22o19_earlier_capability_errors_have_precedence() {
    for (from, to, feature) in [
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
        vertical_shells::reset_invocations();
        assert_eq!(
            slice_project(archive.bytes(), metadata())
                .await
                .unwrap_err(),
            SliceError::UnsupportedProjectFeature(feature.to_owned())
        );
        assert_eq!(vertical_shells::invocations(), 0);
    }
}

#[tokio::test]
async fn task22o19_o17_geometry_failure_precedes_cache_population() {
    surface_type_detection::reset_geometry_hooks();
    surface_type_detection::fail_geometry_at(
        surface_type_detection::GeometryStep::TopSafetyDifference,
    );
    vertical_shells::reset_invocations();
    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::InvalidInput(
            "surface-type detection geometry is outside the supported Clipper range".to_owned()
        )
    );
    assert_eq!(vertical_shells::invocations(), 0);
    surface_type_detection::reset_geometry_hooks();
}

#[tokio::test]
async fn task22o19_multi_region_fails_before_cache_population() {
    let (multi_region, _) = crate::project_slice::tests::region_fixture::modifier_projects();
    vertical_shells::reset_invocations();
    assert_eq!(
        slice_project(multi_region, metadata()).await.unwrap_err(),
        SliceError::UnsupportedProjectFeature("multi_region_layer_slices".to_owned())
    );
    assert_eq!(vertical_shells::invocations(), 0);
}
