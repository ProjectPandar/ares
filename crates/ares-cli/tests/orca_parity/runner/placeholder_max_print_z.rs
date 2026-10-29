//! GCode.cpp:2968-2973: max_print_z is an integer ceiling.

#[test]
fn placeholder_max_print_z_control() {
    crate::runner::native_stream_case::check(
        ("placeholder-max-print-z", &["GCode.cpp:2968-2973"]),
        "control",
        include_bytes!("../../../../../tests/parity/placeholder-max-print-z/control.3mf"),
        "795d0ebc27cf301daa8e112e24c91468ddfd4c13b97af2527b67574d7e8bf903",
        (false, |_, _| {}),
    );
}

#[test]
fn placeholder_max_print_z_positive() {
    crate::runner::native_stream_case::check(
        ("placeholder-max-print-z", &["GCode.cpp:2968-2973"]),
        "positive",
        include_bytes!("../../../../../tests/parity/placeholder-max-print-z/positive.3mf"),
        "bb83ff8b780f7694ecce0e01536ed4a53fe58fb185c592278b5ce17fbee82ca3",
        (false, |_, _| {}),
    );
}
