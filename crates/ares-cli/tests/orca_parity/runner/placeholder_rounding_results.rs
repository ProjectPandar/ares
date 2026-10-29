//! PlaceholderParser.cpp:371-415: rounding functions return integers.

#[test]
fn placeholder_rounding_results_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-rounding-results",
            &[
                "PlaceholderParser.cpp:371-415",
                "PlaceholderParser.cpp:494-510",
            ],
        ),
        "control",
        include_bytes!("../../../../../tests/parity/placeholder-rounding-results/control.3mf"),
        "8b720023714827028a0570096810fa008697e78475cd682cae7fc3a32e1c5a59",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_rounding_results_round() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-rounding-results",
            &[
                "PlaceholderParser.cpp:371-415",
                "PlaceholderParser.cpp:494-510",
            ],
        ),
        "round",
        include_bytes!("../../../../../tests/parity/placeholder-rounding-results/round.3mf"),
        "192464dc3d58f39ea537297ecef3c43b85a5e692a0da42da6608c833e962ba84",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_rounding_results_floor() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-rounding-results",
            &[
                "PlaceholderParser.cpp:371-415",
                "PlaceholderParser.cpp:494-510",
            ],
        ),
        "floor",
        include_bytes!("../../../../../tests/parity/placeholder-rounding-results/floor.3mf"),
        "22c31b72086028648e22683aad1b85e11e2cc3a877a8fa1571d4f79a5e49f4a6",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_rounding_results_ceil() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-rounding-results",
            &[
                "PlaceholderParser.cpp:371-415",
                "PlaceholderParser.cpp:494-510",
            ],
        ),
        "ceil",
        include_bytes!("../../../../../tests/parity/placeholder-rounding-results/ceil.3mf"),
        "b82b943c50aecd227d941cb96989bb0c1104046c777be40b543840da01196f56",
        (false, |_, _| {}),
    );
}
