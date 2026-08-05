use crate::{
    SliceError,
    project_slice::{
        consume_post_vertical_shell_regularization,
        prepare_infill::vertical_shell_regularization::{self, GeometryStep},
        tests::{
            prepare_infill::surface_type_detection::cleanup::{
                deepen_both_tree_families, run_on_constrained_stack,
            },
            support::{KsrArchive, metadata},
        },
    },
};

use super::fixture;

#[test]
fn task22o22_direct_success_and_public_incomplete_disposal_fit_deep_trees() {
    for public in [false, true] {
        let mut prepared = fixture::prepare_o21(KsrArchive::new().bytes());
        deepen_both_tree_families(&mut prepared.predecessor);
        let (probe, dropped) = prepared.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            let output = vertical_shell_regularization::prepare(prepared).unwrap();
            if public {
                assert_eq!(
                    consume_post_vertical_shell_regularization(output, metadata()).unwrap_err(),
                    SliceError::ProjectSlicingIncomplete
                );
            } else {
                vertical_shell_regularization::dispose(output);
            }
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}

#[test]
fn task22o22_each_geometry_failure_disposes_both_deep_trees() {
    for step in [
        GeometryStep::Union,
        GeometryStep::Offset2First,
        GeometryStep::Offset2Second,
        GeometryStep::Shrink,
    ] {
        let mut prepared = fixture::prepare_o21(KsrArchive::new().bytes());
        deepen_both_tree_families(&mut prepared.predecessor);
        let (probe, dropped) = prepared.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            vertical_shell_regularization::reset_geometry_hooks();
            vertical_shell_regularization::fail_geometry_at(step);
            assert!(matches!(
                vertical_shell_regularization::prepare(prepared),
                Err(SliceError::InvalidInput(message))
                    if message == "vertical-shell regularization geometry is outside the supported Clipper range"
            ));
            vertical_shell_regularization::reset_geometry_hooks();
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}
