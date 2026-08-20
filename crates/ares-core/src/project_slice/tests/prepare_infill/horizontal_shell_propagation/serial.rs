use crate::project_slice::prepare_infill::horizontal_shell_propagation::{self, SourceKind};

#[test]
fn task22o26_later_source_gather_observes_earlier_working_rebuild() {
    horizontal_shell_propagation::reset_hooks();
    let output = horizontal_shell_propagation::prepare(super::fixture::controlled(true)).unwrap();
    let observation = horizontal_shell_propagation::gather_observations()
        .into_iter()
        .find(|observation| {
            observation.layer == 1
                && observation.dirty_before_gather
                && observation.path_count > 0
                && observation.path_digest != observation.original_path_digest
        })
        .expect("layer 1 must gather the Top fragment rebuilt by layer 0 Bottom propagation");
    assert_eq!((observation.object, observation.kind), (0, SourceKind::Top));
    horizontal_shell_propagation::dispose(output);
}
