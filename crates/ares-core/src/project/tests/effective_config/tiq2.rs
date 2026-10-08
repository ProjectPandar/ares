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

// Qualified cache history: support builds at spacing400000, then the object
// reuses that cache; raw waypoint4399900,4301461 plus origin165,165.
// GCode.cpp:5343-5429; AvoidCrossingPerimeters.cpp:480-507,1252-1264.
#[tokio::test]
async fn tiq2_support_cache_waypoint_matches_seeded_travel() {
    let output = crate::slice_project(
        include_bytes!("../../../../../../tests/parity/validation/tiq2-type2-enabled.3mf"),
        crate::GenerationMetadata::deterministic(2026, 10, 7, 0, 0, 0),
        crate::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let text = String::from_utf8(output).unwrap();
    let lines = text.lines().collect::<Vec<_>>();
    let travel = lines
        .windows(3)
        .find(|lines| lines[0] == "G1 Z.6 F21000" && lines[2] == "G1 X169.301 Y169.301")
        .unwrap();
    assert_eq!(travel[1], "G1 X169.4 Y169.301 Z.6");
}
