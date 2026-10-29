//! PlaceholderParser.cpp:275,670-684: digits without decimals rounds to integer.

#[test]
fn placeholder_digits_rounding_control() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-digits-rounding",
            &["PlaceholderParser.cpp:275", "PlaceholderParser.cpp:670-684"],
        ),
        "control",
        include_bytes!("../../../../../tests/parity/placeholder-digits-rounding/control.3mf"),
        "8a27b9e0b7b846de267945f932509309cb602bdbe0b25d965714eb13b75b2895",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_digits_rounding_positive() {
    crate::runner::native_stream_case::check(
        (
            "placeholder-digits-rounding",
            &["PlaceholderParser.cpp:275", "PlaceholderParser.cpp:670-684"],
        ),
        "positive",
        include_bytes!("../../../../../tests/parity/placeholder-digits-rounding/positive.3mf"),
        "c1d592fb03d511568fae3d214eb443fabc02a1aa311ca4102a7e88fc0333bb90",
        (false, |_, _| {}),
    );
}
