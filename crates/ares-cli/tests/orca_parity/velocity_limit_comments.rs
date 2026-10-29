//! `GCodeWriter.cpp:341-346`: optional suffix on nonempty merged Klipper commands.

#[test]
fn merged_velocity_verbose_comment_matches_native() {
    crate::runner::native_stream_case::check(
        (
            "velocity-limit-comments",
            &[
                "GCodeWriter.cpp:311-350::set_accel_and_jerk",
                "GCodeWriter.cpp:341-346",
            ],
        ),
        "comments",
        include_bytes!(
            "../../../../tests/parity/velocity-limit-comments/klipper-acceleration-comments.3mf"
        ),
        "ce5dc71dc0a8e39933614cf0d7f752071a5fd3728cdc88a666bfb673560b938b",
        (false, |reference, _| {
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| {
                    line.starts_with("SET_VELOCITY_LIMIT ") && line.contains(" ACCEL_TO_DECEL=")
                })
                .collect();
            assert!(
                commands
                    .iter()
                    .any(|line| line.ends_with(" ; adjust VELOCITY_LIMIT(accel/jerk)"))
            );
            assert!(commands.iter().all(|line| line.contains(" ACCEL=")));
        }),
    );
}

#[test]
fn merged_velocity_no_comment_control_stays_byte_equal() {
    crate::runner::native_stream_case::check(
        (
            "velocity-limit-comments",
            &[
                "GCodeWriter.cpp:311-350::set_accel_and_jerk",
                "GCodeWriter.cpp:341-346",
            ],
        ),
        "control",
        include_bytes!(
            "../../../../tests/parity/velocity-limit-comments/klipper-acceleration-control.3mf"
        ),
        "644dbe2a3e522c3f835ff854c5388ef546820a02cf31eeab442deb8fee3553e9",
        (false, |reference, _| {
            let commands: Vec<_> = reference
                .native
                .lines()
                .filter(|line| {
                    line.starts_with("SET_VELOCITY_LIMIT ") && line.contains(" ACCEL_TO_DECEL=")
                })
                .collect();
            assert!(!commands.is_empty());
            assert!(
                commands
                    .iter()
                    .all(|line| !line.contains(';') && line.contains(" ACCEL="))
            );
        }),
    );
}
