use super::PendingPathBranch;

#[test]
fn task22o5_pending_branch_retains_exact_predicate_operands() {
    assert!(matches!(
        PendingPathBranch::from_operands(true, 3, 2),
        PendingPathBranch::OverhangClipping {
            detect_overhang_wall: true,
            layer_id: 3,
            raft_layers: 2
        }
    ));
    for branch in [
        PendingPathBranch::from_operands(false, 3, 2),
        PendingPathBranch::from_operands(true, 2, 2),
        PendingPathBranch::from_operands(true, 1, 2),
    ] {
        assert!(matches!(branch, PendingPathBranch::OrdinaryUnsplit { .. }));
    }
}
