use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn arachne_near_self_intersection_short_step_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project =
        include_bytes!("../../../../tests/parity/arachne-near-self-intersections/short-step.3mf");
    assert_eq!(
        sha(project),
        "ebc96e851366d5328a05226929dacde3f5b9577dcffce5e51046f71da36a44fe"
    );
    let case = NativeCase::slice(&runner, project, "arachne-near-self-intersection-");
    for (key, value) in [
        ("wall_generator", "arachne"),
        ("wall_maximum_resolution", "0.5"),
        ("wall_maximum_deviation", "0.05"),
        ("resolution", "0.001"),
    ] {
        assert!(case.native.contains(&format!("; {key} = {value}\n")));
    }
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 11, 13, 0, 0, 0,
    ));
}
