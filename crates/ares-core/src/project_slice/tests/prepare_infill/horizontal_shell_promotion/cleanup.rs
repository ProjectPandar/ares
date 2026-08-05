use crate::{
    SliceError,
    project_slice::{
        consume_post_horizontal_shell_promotion,
        prepare_infill::horizontal_shell_promotion,
        tests::{
            prepare_infill::surface_type_detection::cleanup::{
                deepen_both_tree_families, run_on_constrained_stack,
            },
            support::{KsrArchive, metadata},
        },
    },
};

#[test]
fn task22o25_direct_and_public_success_dispose_both_deep_predecessor_families() {
    for public in [false, true] {
        let mut input = super::fixture::prepare_o24(KsrArchive::new().bytes());
        deepen_both_tree_families(&mut input.predecessor);
        let (probe, dropped) = input.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            let output = horizontal_shell_promotion::prepare(input).unwrap();
            if public {
                assert_eq!(
                    consume_post_horizontal_shell_promotion(output, metadata()).unwrap_err(),
                    SliceError::ProjectSlicingIncomplete
                );
            } else {
                horizontal_shell_promotion::dispose(output);
            }
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}

#[test]
fn task22o25_parse_failure_disposes_both_deep_predecessor_families() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"extra_solid_infills\": \"\"",
        "\"extra_solid_infills\": \"2147483648\"",
    );
    let mut input = super::fixture::prepare_o24(archive.bytes());
    deepen_both_tree_families(&mut input.predecessor);
    let (probe, dropped) = input.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        assert!(matches!(
            horizontal_shell_promotion::prepare(input),
            Err(SliceError::InvalidInput(message))
                if message == "invalid extra_solid_infills pattern"
        ));
    });
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}
