//! Print.hpp:1086 and Print.cpp:3444-3541: stock BBL Type1 tower parity.

const UPSTREAM: &[&str] = &[
    "Print.hpp:1086",
    "Print.cpp:3290-3301,3390-3541",
    "GCode/WipeTower.cpp:2449-2510,3516-3806,3864",
    "GCode.cpp:712-1117,2926-2939",
];

#[test]
fn bbl_type1_tower_enabled() {
    crate::runner::native_stream_case::check(
        ("bbl-type1-tower", UPSTREAM),
        "enabled",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/enabled.3mf"),
        "ab00b9040c3ccc1c10490a220880b87108aed7edd3a19926b8e7967f1287e506",
        (false, |_, _| {}),
    );
}

#[test]
fn bbl_type1_tower_disabled_control() {
    crate::runner::native_stream_case::check(
        ("bbl-type1-tower", UPSTREAM),
        "disabled",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/disabled.3mf"),
        "e78d0feba53df6318deb96929eb0620d5392f16a88832fb4ffd75969b853ed28",
        (false, |_, _| {}),
    );
}
