use super::support::{direct_raw_checksums, project, summaries};

#[test]
fn task22o4_ksr_reaches_ordered_loop_hierarchy() {
    let summaries = summaries(project());
    assert!(!summaries.is_empty());
    assert!(summaries.iter().any(|summary| summary.roots > 0));
    assert!(summaries.iter().any(|summary| summary.root_checksum != 0));
}

#[test]
fn task22o4_ksr_hierarchy_is_deterministic_and_preserves_o3_raw_shells() {
    let first = summaries(project());
    assert_eq!(first, summaries(project()));
    assert_eq!(
        first
            .iter()
            .map(|summary| (summary.source_index, summary.raw_checksum))
            .collect::<Vec<_>>(),
        direct_raw_checksums(project())
    );
}
