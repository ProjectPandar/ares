use crate::{SliceError, project_slice::perimeters::classic::infill_boundary, slice_project};

use super::super::super::super::support::{ksr_project, metadata};

#[tokio::test]
async fn task22o15_public_lifecycle_invokes_boundary_once_and_stays_incomplete() {
    infill_boundary::reset_finish_invocations();
    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(infill_boundary::finish_invocations(), 1);
}
