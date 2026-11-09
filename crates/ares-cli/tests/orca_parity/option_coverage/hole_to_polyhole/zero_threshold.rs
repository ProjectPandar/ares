use super::{BASE, LEGS, OrcaRunner, control_native, native, project, run_leg_with_native};

/// `PrintObject.cpp:280-283` tests each radial range independently.
/// Zero threshold does not compare vertex radii against edge midpoints.
#[test]
fn hole_to_polyhole_zero_threshold_converts() {
    let control = control_native(BASE);
    if control.is_empty() {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let root = tempfile::Builder::new()
        .prefix("hole-to-polyhole-enabled-witness-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let enabled = native(&runner, &root.join("enabled"), &project(&LEGS[1]));
    let enabled_motion = motion_commands(&enabled);
    assert!(!enabled_motion.is_empty());
    assert_ne!(
        enabled_motion,
        motion_commands(&control),
        "Native enabled motion must differ from the disabled fixture"
    );
    run_leg_with_native(&LEGS[3], |reference| {
        assert_eq!(
            motion_commands(reference),
            enabled_motion,
            "Native zero-threshold motion must match the enabled fixture"
        );
    });
}

fn motion_commands(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| {
            matches!(
                line.split_ascii_whitespace().next(),
                Some("G0" | "G1" | "G2" | "G3")
            )
        })
        .collect()
}
