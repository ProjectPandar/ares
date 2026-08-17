use crate::project_slice::perimeters::classic::traversal::PendingPathBranch;

use super::support::{assert_record_alignment, project, summaries};

#[test]
fn task22o5_ksr_reaches_semantic_seed_trees_and_preserves_o4() {
    assert_record_alignment(project());
    let summaries = summaries(project());
    assert!(!summaries.is_empty());
    assert!(summaries.iter().any(|summary| summary.roots > 0));
    assert!(summaries.iter().any(|summary| summary.seeds > 0));
    assert!(summaries.iter().any(|summary| summary.checksum != 0));
    assert!(summaries.iter().any(|summary| summary.source_index == 0));
    let _preserved_diagnostics = summaries
        .iter()
        .map(|summary| summary.diagnostics)
        .collect::<Vec<_>>();
}

#[test]
fn task22o5_ksr_is_deterministic_and_retains_pending_branch_provenance() {
    let first = summaries(project());
    assert_eq!(first, summaries(project()));
    assert!(first.iter().any(|summary| matches!(
        summary.branch,
        PendingPathBranch::OverhangClipping {
            detect_overhang_wall: true,
            ..
        }
    )));
    assert!(
        first
            .iter()
            .any(|summary| matches!(summary.branch, PendingPathBranch::OrdinaryUnsplit { .. }))
    );
}
