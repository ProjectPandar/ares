use crate::project_slice::perimeters::classic::top_split::{TopSplitOutcome, TopSplitUpperSource};

use super::support::{archive, geometry_summary, outcomes, project, upper_sources};

#[test]
fn task22o2_only_one_wall_top_disables_only_the_split() {
    let baseline = geometry_summary(project());
    let mut disabled = archive();
    disabled.replace_unique(
        "Metadata/project_settings.config",
        "\"only_one_wall_top\": \"1\"",
        "\"only_one_wall_top\": \"0\"",
    );
    let disabled = disabled.bytes();
    let disabled_outcomes = outcomes(&disabled);
    assert!(!disabled_outcomes.contains(&TopSplitOutcome::Applied));
    assert!(disabled_outcomes.contains(&TopSplitOutcome::Disabled));
    let disabled_geometry = geometry_summary(&disabled);
    assert_eq!(baseline.len(), disabled_geometry.len());
    assert!(disabled_geometry.iter().any(|value| value.0 > 0));
}

#[test]
fn task22o2_interface_shells_selects_the_typed_upper_source() {
    assert!(
        upper_sources(project())
            .iter()
            .all(|source| *source == TopSplitUpperSource::WholeLayer)
    );
    let mut same_region = archive();
    same_region.replace_unique(
        "Metadata/project_settings.config",
        "\"interface_shells\": \"0\"",
        "\"interface_shells\": \"1\"",
    );
    assert!(
        upper_sources(same_region.bytes())
            .iter()
            .all(|source| *source == TopSplitUpperSource::SameRegion)
    );
}

#[test]
fn task22o2_wall_loops_changes_the_exact_caller_gate() {
    let mut one_loop = archive();
    one_loop.replace_unique(
        "Metadata/project_settings.config",
        "\"wall_loops\": \"2\"",
        "\"wall_loops\": \"1\"",
    );
    let changed = outcomes(one_loop.bytes());
    assert!(!changed.contains(&TopSplitOutcome::Applied));
    assert!(changed.contains(&TopSplitOutcome::OneLoop));
}

#[test]
fn task22o2_gap_and_width_options_change_post_split_geometry() {
    let baseline = geometry_summary(project());
    let mut no_gap = archive();
    no_gap.replace_unique(
        "Metadata/project_settings.config",
        "\"gap_infill_speed\": \"250\"",
        "\"gap_infill_speed\": \"0\"",
    );
    assert_ne!(baseline, geometry_summary(no_gap.bytes()));

    let mut min_width = archive();
    min_width.replace_unique(
        "Metadata/project_settings.config",
        "\"min_width_top_surface\": \"300%\"",
        "\"min_width_top_surface\": \"0.2\"",
    );
    assert_ne!(baseline, geometry_summary(min_width.bytes()));

    let mut auto_sparse_width = archive();
    auto_sparse_width.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_line_width\": \"0.45\"",
        "\"sparse_infill_line_width\": \"0\"",
    );
    assert_ne!(baseline, geometry_summary(auto_sparse_width.bytes()));

    let mut wider_sparse_width = archive();
    wider_sparse_width.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_line_width\": \"0.45\"",
        "\"sparse_infill_line_width\": \"0.6\"",
    );
    assert_ne!(baseline, geometry_summary(wider_sparse_width.bytes()));
}
