use crate::project_slice::perimeters::classic::top_split::TopSplitOutcome;

use super::support::{geometry_summary, outcomes, project};

#[test]
fn task22o2_first_outer_and_split_geometry_are_repeatable() {
    let first = geometry_summary(project());
    assert!(!first.is_empty());
    assert_eq!(first, geometry_summary(project()));
    assert!(first.iter().any(|value| value.0 > 0));
    assert!(first.iter().any(|value| value.2 > 0));
}

#[test]
fn task22o2_topmost_and_single_loop_surfaces_are_skipped() {
    let outcomes = outcomes(project());
    assert!(outcomes.contains(&TopSplitOutcome::NoUpperLayer));
    assert!(outcomes.contains(&TopSplitOutcome::Applied));
}
