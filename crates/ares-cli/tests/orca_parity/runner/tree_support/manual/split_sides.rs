use super::split::support_moves;
use super::{
    OrcaRunner, has_support_extrusion, overhang_case, paint_canopy, parity, slice_painted,
};

#[test]
fn manual_slim_one_edge_side_zero_preserves_complete_output() {
    compare_split_side("tree-support/manual/slim/one-edge/side-zero", "401");
}

#[test]
fn manual_slim_one_edge_side_one_preserves_complete_output() {
    compare_split_side("tree-support/manual/slim/one-edge/side-one", "405");
}

fn compare_split_side(label: &str, code: &str) {
    let Some(baseline) = overhang_case(label, "tree_slim", &[("support_type", "tree(manual)")])
    else {
        return;
    };
    assert!(!has_support_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let leaf = slice_painted(
        &runner,
        paint_canopy(&baseline.project, &["4", "4"]),
        "split-side-leaf-control",
    );
    let divided = slice_painted(
        &runner,
        paint_canopy(&baseline.project, &[code, "4"]),
        "split-side-painted",
    );
    let leaf_gcode = std::str::from_utf8(&leaf.reference).unwrap();
    let divided_gcode = std::str::from_utf8(&divided.reference).unwrap();
    assert!(has_support_extrusion(leaf_gcode));
    assert!(has_support_extrusion(divided_gcode));
    assert_ne!(support_moves(leaf_gcode), support_moves(divided_gcode));
    let outcome = parity::compare_case(&divided);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
