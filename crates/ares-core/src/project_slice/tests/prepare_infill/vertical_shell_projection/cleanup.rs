use crate::{
    SliceError,
    project_slice::{
        consume_post_vertical_shell_projection,
        prepare_infill::vertical_shell_projection::{self, GeometryStep},
        tests::{
            deep_cleanup_support::{deepen_both_tree_families, run_on_constrained_stack},
            support::{KsrArchive, metadata},
        },
    },
};

use super::fixture;

#[test]
fn task22o20_direct_success_disposal_fits_constrained_stack_with_both_deep_trees() {
    let mut prepared = fixture::prepare_o19(KsrArchive::new().bytes());
    deepen_both_tree_families(&mut prepared.predecessor);
    let (probe, dropped) = prepared.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        let output = vertical_shell_projection::prepare(prepared).unwrap();
        vertical_shell_projection::dispose(output);
    });
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn task22o20_public_incomplete_consumption_fits_constrained_stack_with_both_deep_trees() {
    let mut prepared = fixture::prepare_o19(KsrArchive::new().bytes());
    deepen_both_tree_families(&mut prepared.predecessor);
    let (probe, dropped) = prepared.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        let output = vertical_shell_projection::prepare(prepared).unwrap();
        assert_eq!(
            consume_post_vertical_shell_projection(output, metadata()).unwrap_err(),
            SliceError::ProjectSlicingIncomplete
        );
    });
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn task22o20_failure_disposal_fits_constrained_stack_with_both_deep_trees() {
    for step in [GeometryStep::HoleIntersection, GeometryStep::ShellUnion] {
        let mut prepared = fixture::prepare_o19(KsrArchive::new().bytes());
        deepen_both_tree_families(&mut prepared.predecessor);
        let (probe, dropped) = prepared.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            vertical_shell_projection::reset_geometry_hooks();
            vertical_shell_projection::fail_geometry_at(step);
            assert!(matches!(
                vertical_shell_projection::prepare(prepared),
                Err(SliceError::InvalidInput(message))
                    if message == "vertical-shell projection geometry is outside the supported Clipper range"
            ));
            vertical_shell_projection::reset_geometry_hooks();
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}

#[test]
fn task22o20_each_anchor_failure_disposes_both_deep_trees() {
    for (step, top) in [
        (GeometryStep::TopAnchorOffset, true),
        (GeometryStep::TopAnchorIntersection, true),
        (GeometryStep::BottomAnchorOffset, false),
        (GeometryStep::BottomAnchorIntersection, false),
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
        let mut prepared = fixture::prepare_o19(archive.bytes());
        deepen_both_tree_families(&mut prepared.predecessor);
        let (probe, dropped) = prepared.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            vertical_shell_projection::reset_geometry_hooks();
            vertical_shell_projection::fail_geometry_at(step);
            assert!(vertical_shell_projection::prepare(prepared).is_err());
            vertical_shell_projection::reset_geometry_hooks();
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}
