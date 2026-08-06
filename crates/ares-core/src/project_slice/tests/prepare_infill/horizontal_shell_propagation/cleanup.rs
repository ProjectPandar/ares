use crate::{
    SliceError,
    project_slice::{
        consume_post_horizontal_shell_propagation,
        prepare_infill::horizontal_shell_propagation::{self, GeometryStep},
        tests::{
            prepare_infill::surface_type_detection::cleanup::{
                deepen_both_tree_families, run_on_constrained_stack,
            },
            support::{KsrArchive, metadata},
        },
    },
};

#[test]
fn task22o26_direct_and_public_success_dispose_both_deep_predecessor_families() {
    for public in [false, true] {
        let mut input = super::fixture::prepare_o25(KsrArchive::new().bytes());
        deepen_both_tree_families(&mut input.predecessor);
        let (probe, dropped) = input.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            let output = horizontal_shell_propagation::prepare(input).unwrap();
            if public {
                assert_eq!(
                    consume_post_horizontal_shell_propagation(output, metadata()).unwrap_err(),
                    SliceError::ProjectSlicingIncomplete
                );
            } else {
                horizontal_shell_propagation::dispose(output);
            }
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}

#[test]
fn task22o26_late_geometry_failure_disposes_both_deep_predecessor_families() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"ensure_vertical_shell_thickness\": \"ensure_all\",",
        "\t\"ensure_vertical_shell_thickness\": \"ensure_moderate\",",
    );
    let mut input = super::fixture::prepare_o25(archive.bytes());
    deepen_both_tree_families(&mut input.predecessor);
    let (probe, dropped) = input.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        horizontal_shell_propagation::reset_hooks();
        horizontal_shell_propagation::fail_geometry_at_occurrence(
            GeometryStep::ExternalGroupDifference,
            2,
        );
        assert!(matches!(
            horizontal_shell_propagation::prepare(input),
            Err(SliceError::InvalidInput(message))
                if message == "horizontal-shell propagation geometry is outside the supported Clipper range"
        ));
        assert_eq!(horizontal_shell_propagation::commits(), 0);
        assert_eq!(horizontal_shell_propagation::disposals(), 1);
    });
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}
