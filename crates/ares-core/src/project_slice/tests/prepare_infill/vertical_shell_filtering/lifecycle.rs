mod precedence;

use crate::{
    SliceError,
    project_slice::{
        prepare_infill::{vertical_shell_filtering, vertical_shell_regularization},
        tests::support::{KsrArchive, metadata},
    },
    slice_project,
};

#[tokio::test]
async fn task22o23_o22_failure_precedes_filtering() {
    vertical_shell_regularization::reset_geometry_hooks();
    vertical_shell_regularization::fail_geometry_at(
        vertical_shell_regularization::GeometryStep::Union,
    );
    vertical_shell_filtering::reset_invocations();
    assert!(matches!(
        slice_project(KsrArchive::new().bytes(), metadata()).await,
        Err(SliceError::InvalidInput(message))
            if message == "vertical-shell regularization geometry is outside the supported Clipper range"
    ));
    assert_eq!(vertical_shell_filtering::invocations(), 0);
    vertical_shell_regularization::reset_geometry_hooks();
}
