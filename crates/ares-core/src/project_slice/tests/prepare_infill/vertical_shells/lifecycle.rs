use crate::project_slice::{
    prepare_infill::{surface_type_detection, vertical_shells},
    tests::support::{KsrArchive, metadata},
};
use crate::{SliceError, slice_project};

#[tokio::test]
async fn task22o19_o17_geometry_failure_precedes_cache_population() {
    surface_type_detection::reset_geometry_hooks();
    surface_type_detection::fail_geometry_at(
        surface_type_detection::GeometryStep::TopSafetyDifference,
    );
    vertical_shells::reset_invocations();
    assert_eq!(
        slice_project(
            KsrArchive::new().bytes(),
            metadata(),
            crate::SlicerContext::new(0)
        )
        .await
        .unwrap_err(),
        SliceError::InvalidInput(
            "surface-type detection geometry is outside the supported Clipper range".to_owned()
        )
    );
    assert_eq!(vertical_shells::invocations(), 0);
    surface_type_detection::reset_geometry_hooks();
}
