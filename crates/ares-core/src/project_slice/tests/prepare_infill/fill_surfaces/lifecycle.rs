use crate::project_slice::{
    perimeters,
    prepare_infill::{fill_surfaces, surface_type_detection},
};
use crate::{SliceError, slice_project};

use super::super::super::support::{KsrArchive, metadata};

#[tokio::test]
async fn task22o18_public_lifecycle_runs_once_and_remains_incomplete() {
    fill_surfaces::reset_invocations();
    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(fill_surfaces::invocations(), 1);
}

#[tokio::test]
async fn task22o18_earlier_errors_leave_invocations_zero() {
    for (from, to, error) in [
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
        fill_surfaces::reset_invocations();
        assert_eq!(
            slice_project(archive.bytes(), metadata())
                .await
                .unwrap_err(),
            SliceError::UnsupportedProjectFeature(error.to_owned())
        );
        assert_eq!(fill_surfaces::invocations(), 0);
    }

    fill_surfaces::reset_invocations();
    surface_type_detection::reset_geometry_hooks();
    surface_type_detection::fail_geometry_at(
        surface_type_detection::GeometryStep::TopSafetyDifference,
    );
    assert!(matches!(
        slice_project(KsrArchive::new().bytes(), metadata()).await,
        Err(SliceError::InvalidInput(_))
    ));
    assert_eq!(fill_surfaces::invocations(), 0);
    surface_type_detection::reset_geometry_hooks();
}

#[tokio::test]
async fn task22o18_global_spiral_is_rejected_even_when_record_flags_would_be_false() {
    let mut thresholds = KsrArchive::new();
    thresholds.replace_unique(
        "Metadata/project_settings.config",
        "\"bottom_shell_layers\": \"3\"",
        "\"bottom_shell_layers\": \"1000\"",
    );
    thresholds.replace_unique(
        "Metadata/project_settings.config",
        "\"bottom_shell_thickness\": \"0\"",
        "\"bottom_shell_thickness\": \"1000\"",
    );
    let detected = surface_type_detection::prepare(
        perimeters::prepare_post_layer_region_perimeters(thresholds.clone().bytes()).unwrap(),
    )
    .unwrap();
    assert!(detected.predecessor.objects.iter().all(|object| {
        let input = &object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        input
            .records
            .iter()
            .flatten()
            .all(|record| !record.spiral_mode)
    }));

    thresholds.replace_unique(
        "Metadata/project_settings.config",
        "\"spiral_mode\": \"0\"",
        "\"spiral_mode\": \"1\"",
    );
    fill_surfaces::reset_invocations();
    surface_type_detection::reset_invocations();
    assert_eq!(
        slice_project(thresholds.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("spiral_mode".to_owned())
    );
    assert_eq!(surface_type_detection::invocations(), 0);
    assert_eq!(fill_surfaces::invocations(), 0);
}
