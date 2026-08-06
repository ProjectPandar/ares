use crate::{
    SliceError,
    project_slice::{
        prepare_infill::horizontal_shell_propagation::{self, GeometryStep, PropagationEvent},
        tests::support::KsrArchive,
    },
};

const ERROR: &str = "horizontal-shell propagation geometry is outside the supported Clipper range";

fn archive(mode: &str) -> KsrArchive {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
        &format!("\"ensure_vertical_shell_thickness\": \"{mode}\""),
    );
    archive
}

fn rejects_at(step: GeometryStep, mode: &str, occurrence: usize) -> Vec<PropagationEvent> {
    let input = super::super::fixture::prepare_o25(archive(mode).bytes());
    horizontal_shell_propagation::reset_hooks();
    horizontal_shell_propagation::fail_geometry_at_occurrence(step, occurrence);
    let Err(error) = horizontal_shell_propagation::prepare(input) else {
        panic!("selected geometry occurrence must fail");
    };
    assert_eq!(error, SliceError::InvalidInput(ERROR.to_owned()));
    assert_eq!(horizontal_shell_propagation::commits(), 0);
    assert_eq!(horizontal_shell_propagation::disposals(), 1);
    let rollback = horizontal_shell_propagation::rollback_snapshots();
    assert_eq!(rollback.len(), 2);
    assert_eq!(rollback[1], rollback[0]);
    let events = horizontal_shell_propagation::events();
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, PropagationEvent::DirtyCommit { .. }))
    );
    events
}

#[test]
fn task22o26_every_geometry_site_fails_the_transaction_before_original_commit() {
    for step in [
        GeometryStep::SafetyIntersection,
        GeometryStep::NeighborExternalWidthScale,
        GeometryStep::FirstOpeningShrink,
        GeometryStep::FirstOpeningExpand,
        GeometryStep::FirstTooNarrowDifference,
        GeometryStep::FirstTrimDifference,
    ] {
        let _ = rejects_at(step, "none", 1);
    }
    for step in [
        GeometryStep::SourceSolidWidthScale,
        GeometryStep::SecondOpeningShrink,
        GeometryStep::SecondOpeningExpand,
        GeometryStep::SecondTooNarrowDifference,
        GeometryStep::RepairExpansion,
        GeometryStep::RepairIntersection,
        GeometryStep::SolidUnion,
        GeometryStep::InternalSafetyDifference,
        GeometryStep::ExternalGroupDifference,
    ] {
        let _ = rejects_at(step, "ensure_moderate", 1);
    }
}

#[test]
fn task22o26_actual_aligned_flow_overflows_roll_back_the_complete_original_graph() {
    for (mode, step, external) in [
        ("none", GeometryStep::NeighborExternalWidthScale, true),
        (
            "ensure_moderate",
            GeometryStep::SourceSolidWidthScale,
            false,
        ),
    ] {
        let mut input = super::super::fixture::prepare_o25(archive(mode).bytes());
        let records = &mut input.predecessor.objects[0]
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .records;
        for record in records.iter_mut().flatten() {
            if external {
                record.ext_perimeter_flow.width = f32::MAX;
            } else {
                record.solid_infill_flow.width = f32::MAX;
            }
        }
        horizontal_shell_propagation::reset_hooks();
        let Err(error) = horizontal_shell_propagation::prepare(input) else {
            panic!("invalid aligned flow must fail the full transaction");
        };
        assert_eq!(error, SliceError::InvalidInput(ERROR.to_owned()));
        assert_eq!(
            horizontal_shell_propagation::geometry_events().last(),
            Some(&step)
        );
        assert_eq!(horizontal_shell_propagation::commits(), 0);
        assert_eq!(horizontal_shell_propagation::disposals(), 1);
        let rollback = horizontal_shell_propagation::rollback_snapshots();
        assert_eq!(rollback.len(), 2);
        assert_eq!(rollback[1], rollback[0]);
    }
}

#[test]
fn task22o26_late_failure_discards_multiple_successful_working_rebuilds() {
    let events = rejects_at(GeometryStep::ExternalGroupDifference, "ensure_moderate", 20);
    assert!(
        events
            .iter()
            .filter(|event| matches!(event, PropagationEvent::Rebuild { .. }))
            .count()
            > 1
    );
}
