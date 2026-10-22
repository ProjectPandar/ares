//! Print::brim_flow is shared across objects, including GUI width overrides.

#[test]
fn second_object_inner_width_target() {
    check(
        "target",
        include_bytes!("../../../../tests/parity/print-global-brim-flow/target.3mf"),
        "b0580d24169a925fc8de7241600dd615767a41ef209a2efc3617690980fcfcc5",
    );
}

#[test]
fn uniform_inner_width_control() {
    check(
        "control",
        include_bytes!("../../../../tests/parity/print-global-brim-flow/control.3mf"),
        "7e019a264fb7a912276bf16a3559e62c62a6f521c1397181fd01ae3a3a963e8a",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "print-global-brim-flow",
            &[
                "Print.cpp:2007-2025::Print::brim_flow",
                "Brim.cpp:445-463::outer_inner_brim_area",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
