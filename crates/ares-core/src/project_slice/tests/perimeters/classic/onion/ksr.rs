use crate::{SliceError, slice_project};

use super::support::{metadata, project, summaries};

#[test]
fn task22o3_ksr_reaches_internal_raw_onion_state() {
    let surfaces = summaries(project());
    assert!(!surfaces.is_empty());
    assert!(surfaces.iter().any(|surface| {
        surface.initial >= surface.effective
            && surface.depths.contains(&0)
            && surface.depths.iter().any(|depth| *depth >= 1)
            && surface.depth_zero_normal + surface.depth_zero_smaller > 0
            && surface.last > 0
            && surface.effective >= 1
            && surface.geometry_checksum != 0
    }));
    assert!(surfaces.iter().any(|surface| surface.source_index == 0));
    assert!(surfaces.iter().any(|surface| surface.gaps > 0));
}

#[test]
fn task22o3_ksr_typed_geometry_is_deterministic() {
    assert_eq!(summaries(project()), summaries(project()));
}

#[tokio::test]
async fn task22o3_ksr_public_lifecycle_executes_stage_then_stays_incomplete() {
    assert_eq!(
        slice_project(project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}
