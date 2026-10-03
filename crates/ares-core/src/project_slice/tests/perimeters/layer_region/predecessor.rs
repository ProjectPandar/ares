//! `PostPerimeterPredecessor` envelope behavior at the common classic/arachne
//! post-perimeter boundary (`PerimeterGenerator.cpp:2093` dispatch seam).
use crate::project_slice::{
    perimeters::{layer_region::PostPerimeterPredecessor, prepare_post_layer_region_perimeters},
    prepare_infill::surface_type_detection,
};

use super::super::super::support::KsrArchive;

#[test]
fn post_perimeter_predecessor_envelopes_the_classic_traversal_without_copying() {
    let prepared = prepare_post_layer_region_perimeters(
        &KsrArchive::new().bytes(),
        &mut crate::SlicerContext::new(0),
    )
    .unwrap();
    let boxed = std::ptr::from_ref(prepared.predecessor.as_classic());
    assert!(matches!(
        prepared.predecessor,
        PostPerimeterPredecessor::Classic(_)
    ));
    let drilled = prepared.predecessor.into_classic();
    assert_eq!(std::ptr::from_ref(drilled.as_ref()), boxed);
}

#[test]
fn surface_type_detection_forwards_the_drilled_classic_allocation() {
    let prepared = prepare_post_layer_region_perimeters(
        &KsrArchive::new().bytes(),
        &mut crate::SlicerContext::new(0),
    )
    .unwrap();
    let boxed = std::ptr::from_ref(prepared.predecessor.as_classic());
    let output = surface_type_detection::prepare(prepared).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), boxed);
}
