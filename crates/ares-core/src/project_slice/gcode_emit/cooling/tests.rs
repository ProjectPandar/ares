mod equalizer_drain;
mod role_end;

#[test]
fn redundant_feed_only_move_is_dropped() {
    let mut output = b"G1 F9000\nG1 F9000\n".to_vec();
    let mut state = super::feedrate::State::new(
        vec![(
            0,
            super::feedrate::Config {
                enabled: false,
                target_time: 0.0,
                minimum_speed: 0.0,
                keep_outer_wall_speed: false,
                relative_e: true,
                keep_markers: false,
            },
        )],
        150.0,
        "T",
        0,
    );

    super::feedrate::rewrite_layer(&mut output, 0, &mut state);

    // Upstream skips the complete zero-length F-collision line including
    // its newline (`CoolingBuffer.cpp:910-913`).
    assert_eq!(output, b"G1 F9000\n");
}
