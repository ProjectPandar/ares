//! PlaceholderParser.cpp:617-644: min/max preserve integer result typing.

#[test]
fn placeholder_minmax_results_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-minmax-results",
            &["PlaceholderParser.cpp:617-644"],
        ),
        "control",
        include_bytes!("../../../../../tests/parity/placeholder-minmax-results/control.3mf"),
        "7386cb2553829028b6a740743f49b6af833adbe3fbb2090d283f558601e6bfba",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_minmax_results_min() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-minmax-results",
            &["PlaceholderParser.cpp:617-644"],
        ),
        "min",
        include_bytes!("../../../../../tests/parity/placeholder-minmax-results/min.3mf"),
        "9bb1d7ff26621d2c43c9eb6cf92a9910e237db422580874a7b39267269ba46b1",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_minmax_results_max() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-minmax-results",
            &["PlaceholderParser.cpp:617-644"],
        ),
        "max",
        include_bytes!("../../../../../tests/parity/placeholder-minmax-results/max.3mf"),
        "27e8385fbbdc816166e98c1dd10a94397f77c0d74ad7016ea9728a24f2c17b97",
        (false, |_, _| {}),
    );
}
