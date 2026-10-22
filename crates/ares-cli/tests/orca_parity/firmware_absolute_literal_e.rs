//! Native G10/G11 emulation forwards literal E words to the active E mode.

#[test]
fn firmware_absolute_literal_e_explicit_retract_control() {
    crate::runner::native_stream_case::check(
        (
            "firmware-absolute-literal-e",
            &["GCode/GCodeProcessor.cpp:4873-4890"],
        ),
        "control",
        include_bytes!("../../../../tests/parity/firmware-absolute-literal-e/control.3mf"),
        "b5978da3842bd1734fcc36930d788c180b0ce0461605975ef9a4960c3dd79216",
        (false, |_, _| {}),
    );
}

#[test]
fn firmware_absolute_literal_e_firmware_retract_positive() {
    crate::runner::native_stream_case::check(
        (
            "firmware-absolute-literal-e",
            &["GCode/GCodeProcessor.cpp:4873-4890"],
        ),
        "positive",
        include_bytes!("../../../../tests/parity/firmware-absolute-literal-e/positive.3mf"),
        "dd75e19b01ae72b130de91421e4466ba9488e657dedd4d5fe96efba6cb712f14",
        (false, |_, _| {}),
    );
}
