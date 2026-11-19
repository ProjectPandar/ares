use crate::runner::native_stream_case;

const PROJECT: &[u8] = include_bytes!(
    "../../../../tests/parity/unclear-travel-lift-cancellation/unclear-after-lift.3mf"
);
const SHA: &str = "4e1e66e707189ab66cfab2b4d0a827c7576069d273d5df49f4f351caa6fc46c7";

#[test]
fn approach_keeps_z_after_cancelling_a_lift_at_the_rounded_layer_height() {
    native_stream_case::check(
        (
            "unclear-travel-lift-cancellation",
            &[
                "GCodeWriter.cpp::travel_to_xyz:1096–1139",
                "GCodeWriter.cpp::will_move_z:1279–1292",
                "GCode.cpp::_extrude_piece:9361–9383",
            ],
        ),
        "rounded-toolchange-z",
        PROJECT,
        SHA,
        (true, |reference, replay| {
            for native in [&reference.native, &replay.native] {
                let moves: Vec<_> = native
                    .lines()
                    .filter(|line| line.starts_with("G1 "))
                    .collect();
                assert!(moves.windows(2).any(|pair| pair
                    == [
                        "G1 Z.6 ; move to first perimeter point",
                        "G1 Z.6 ; ensure Z matches planned layer height",
                    ]));
            }
        }),
    );
}
