use super::split::support_moves;
use super::{
    OrcaRunner, has_support_extrusion, overhang_case, paint_canopy, parity, slice_painted,
};

#[test]
fn auto_slim_painted_blocker_suppresses_support_and_preserves_complete_output() {
    let label = "tree-support/auto/slim/painted-blocker";
    let Some(baseline) = overhang_case(label, "tree_slim", &[]) else {
        return;
    };
    assert!(has_support_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let blocked = slice_painted(
        &runner,
        paint_canopy(&baseline.project, &["8", "8"]),
        "painted-blocker",
    );
    assert!(!has_support_extrusion(
        std::str::from_utf8(&blocked.reference).unwrap()
    ));
    let outcome = parity::compare_case(&blocked);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn auto_slim_split_blocker_changes_support_paths_and_preserves_complete_output() {
    let label = "tree-support/auto/slim/split-painted-blocker";
    let Some(baseline) = overhang_case(label, "tree_slim", &[]) else {
        return;
    };
    let baseline_gcode = std::str::from_utf8(&baseline.reference).unwrap();
    assert!(has_support_extrusion(baseline_gcode));
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let blocked = slice_painted(
        &runner,
        paint_canopy(&baseline.project, &["80803", "0"]),
        "split-painted-blocker",
    );
    let blocked_gcode = std::str::from_utf8(&blocked.reference).unwrap();
    assert!(has_support_extrusion(blocked_gcode));
    assert_ne!(support_moves(baseline_gcode), support_moves(blocked_gcode));
    let outcome = parity::compare_case(&blocked);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
