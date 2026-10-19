use super::{Case, Mesh, OrcaRunner, evaluate, inputs, observation, require_equal};
use std::fs;

#[test]
fn nearest_slotted_rectilinear_inner_seam_matches_native() {
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let active = inputs::reference(
        &runner,
        "nearest-slotted-inner-seam",
        Case {
            pattern: "alignedrectilinear",
            count: "1",
            angle: "22.5",
            density: "30%",
            mesh: Mesh::RotatedSlot,
            combined: false,
            nearest: true,
        },
    );
    observation::require_hole_clipping(&observation::observe(&active.reference));
    let root = evaluate(&active);
    assert_eq!(
        fs::read(root.join("ares.gcode")).unwrap(),
        fs::read(root.join("ares.cli.gcode")).unwrap(),
    );
    require_equal(&root);
}
