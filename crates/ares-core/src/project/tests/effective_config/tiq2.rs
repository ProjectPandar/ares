use crate::{
    OrcaBool, load_project,
    project::effective_config::phases::{
        NormalizeFdm2Pass, resolve_bounded_project_config_with_trace,
    },
};

// Failure modes: configured slots mistaken for used slots; final normalization
// uses stale counts; normalized enable is lost when resolving runtime views.
// Seeded probe 5v0gz2rkj... is whole-byte identical to oracle 1k0p5nlbs...:
// PrintApply.cpp:1620-1621 -> normalize_fdm_2(objects=1, used=1), enable 1 -> 0.
#[test]
fn tiq2_final_used_filament_normalization_matches_seeded_print_apply() {
    let project = load_project(include_bytes!(
        "../../../../../../tests/parity/validation/tiq2-type2-enabled.3mf"
    ))
    .unwrap();
    let (resolved, trace) = resolve_bounded_project_config_with_trace(&project).unwrap();
    let final_call = trace
        .normalize_fdm_2
        .iter()
        .find(|call| call.pass == NormalizeFdm2Pass::FirstLate)
        .unwrap();
    assert_eq!(
        (final_call.object_count, final_call.used_filament_count),
        (1, 1),
        "normalization calls: {:?}",
        trace
            .normalize_fdm_2
            .iter()
            .map(|call| (
                call.pass,
                call.object_count,
                call.used_filament_count,
                call.enable_prime_tower
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(trace.normalize_fdm_2.len(), 2);
    assert!(!final_call.enable_prime_tower);
    assert_eq!(
        resolved.views.runtime.process.print.enable_prime_tower,
        OrcaBool(false)
    );
}
