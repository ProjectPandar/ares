//! GCodeReader.cpp:29-84 numeric-word acceptance before processor motion input.
use crate::runner::native_stream_case::check;

const SOURCE: (&str, &[&str]) = (
    "processor-word-boundary",
    &[
        "GCodeReader.cpp:29-84",
        "GCodeReader.hpp:183-196",
        "GCode/GCodeProcessor.cpp:3813-3822",
    ],
);

#[test]
fn processor_word_boundary_spaced_control_matches_native_complete_output() {
    check(
        SOURCE,
        "control",
        include_bytes!("../../../../../tests/parity/processor-word-boundary/control.3mf"),
        "3a9c71fea578cca734cc03236cfbfc3cc1c664ef33b7b4d275a0d7ef76c3a7d9",
        (false, |_, _| {}),
    );
}

#[test]
fn processor_word_boundary_compact_target_matches_native_complete_output() {
    check(
        SOURCE,
        "target",
        include_bytes!("../../../../../tests/parity/processor-word-boundary/target.3mf"),
        "14a8c16229dad5db584b4b15573db49c37aadc2c0b804836047b50acddf6cb54",
        (false, |_, _| {}),
    );
}
