use crate::project_slice::prepare_infill::{fill_surfaces, surface_type_detection};
use crate::{SliceError, slice_project};

use super::super::super::support::{KsrArchive, metadata};

#[tokio::test]
async fn task22o18_earlier_errors_leave_invocations_zero() {
    for (from, to, error) in [(
        "\"counterbore_hole_bridging\": \"none\"",
        "\"counterbore_hole_bridging\": \"partiallybridge\"",
        "counterbore_hole_bridging",
    )] {
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
