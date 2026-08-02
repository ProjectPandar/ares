use super::support::{archive, project, summaries};

#[test]
fn task22o3_ksr_sparse_density_zero_suppresses_the_gap_only_pass() {
    let baseline = summaries(project());
    let mut zero = archive();
    zero.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_density\": \"15%\"",
        "\"sparse_infill_density\": \"0%\"",
    );
    let zero = summaries(zero.bytes());

    assert_eq!(
        baseline
            .iter()
            .map(|surface| &surface.depths)
            .collect::<Vec<_>>(),
        zero.iter()
            .map(|surface| &surface.depths)
            .collect::<Vec<_>>()
    );
    assert!(
        baseline.iter().map(|surface| surface.gaps).sum::<usize>()
            >= zero.iter().map(|surface| surface.gaps).sum::<usize>()
    );
    assert_ne!(baseline, zero);
}

#[test]
fn task22o3_ksr_gap_fill_speed_zero_disables_all_gap_masks() {
    let mut disabled = archive();
    disabled.replace_unique(
        "Metadata/project_settings.config",
        "\"gap_infill_speed\": \"250\"",
        "\"gap_infill_speed\": \"0\"",
    );
    let disabled = summaries(disabled.bytes());

    assert!(disabled.iter().all(|surface| surface.gaps == 0));
    assert_ne!(summaries(project()), disabled);
}

#[test]
fn task22o3_ksr_wall_loop_mutation_changes_raw_shell_depth() {
    let baseline = summaries(project());
    let mut deeper = archive();
    deeper.replace_unique(
        "Metadata/project_settings.config",
        "\"wall_loops\": \"2\"",
        "\"wall_loops\": \"3\"",
    );
    let deeper = summaries(deeper.bytes());

    assert_ne!(baseline, deeper);
    assert_ne!(
        baseline
            .iter()
            .map(|surface| &surface.depths)
            .collect::<Vec<_>>(),
        deeper
            .iter()
            .map(|surface| &surface.depths)
            .collect::<Vec<_>>()
    );
}
