mod inherited;

use crate::project_slice::{
    prepare_infill::{vertical_shell_filtering, vertical_shell_regularization},
    tests::support::KsrArchive,
};

#[test]
fn task22o23_every_outer_alignment_mismatch_rejects_before_geometry() {
    let mut objects = prepared();
    objects.objects.pop();
    rejects(objects);
    let mut caches = prepared();
    caches.caches.pop();
    rejects(caches);
    let mut projections = prepared();
    projections.projections.pop();
    rejects(projections);
    let mut trims = prepared();
    trims.trims.pop();
    rejects(trims);
    let mut regularizations = prepared();
    regularizations.regularizations.pop();
    rejects(regularizations);
    let mut traversal = prepared();
    traversal.predecessor.objects.pop();
    rejects(traversal);
}

#[test]
fn task22o23_every_new_record_alignment_mismatch_rejects_before_geometry() {
    let mut regularizations = prepared();
    regularizations.regularizations[0].records.pop();
    rejects(regularizations);
    let mut traversal = prepared();
    traversal.predecessor.objects[0].records.pop();
    rejects(traversal);

    let mut regularization_presence = prepared();
    regularization_presence.regularizations[0].records[1] = None;
    rejects(regularization_presence);
    let mut traversal_presence = prepared();
    traversal_presence.predecessor.objects[0].records[1] = None;
    rejects(traversal_presence);
}

#[test]
fn task22o23_inherited_count_and_identity_mismatches_still_precede_geometry() {
    let mut lslices = prepared();
    lslices.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .object
        .as_parts_mut()
        .1
        .pop();
    rejects(lslices);

    let mut identity = prepared();
    identity.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records[0]
        .as_mut()
        .unwrap()
        .planned_layer_index += 1;
    rejects(identity);
}

pub(super) fn prepared() -> vertical_shell_regularization::PreparedPostVerticalShellRegularization {
    super::super::fixture::prepare_o22(KsrArchive::new().bytes())
}

pub(super) fn rejects(
    prepared: vertical_shell_regularization::PreparedPostVerticalShellRegularization,
) {
    vertical_shell_filtering::reset_geometry_hooks();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = vertical_shell_filtering::prepare(prepared);
        }))
        .is_err()
    );
    assert!(vertical_shell_filtering::geometry_events().is_empty());
}
