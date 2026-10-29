//! Whole-output Native RED for the forced BBL eager caller's can_lift handoff.
use crate::runner::native_stream_case;

const SOURCE: (&str, &[&str]) = (
    "bbl-eager-enforce",
    &[
        "GCode.cpp:5205-5210",
        "GCode.cpp:7676-7704",
        "GCodeWriter.cpp:652-681",
    ],
);

#[test]
fn orca_parity_bbl_eager_enforce_all_surfaces_complete_output() {
    native_stream_case::check(
        SOURCE,
        "all-surfaces",
        include_bytes!("../../../../../tests/parity/bbl-eager-enforce/all-surfaces.3mf"),
        "438bf4c61895a7a8d51e3c61a70fa8df120a9f3a607d63a4291d359984154a7c",
        (false, |_, _| {}),
    );
}

#[test]
fn orca_parity_bbl_eager_enforce_bottom_only_complete_output() {
    native_stream_case::check(
        SOURCE,
        "bottom-only",
        include_bytes!("../../../../../tests/parity/bbl-eager-enforce/bottom-only.3mf"),
        "f735864a4dc6d992440a7e9d9a8b5f620775cdda7ea09d53dc09e65849664bcf",
        (false, |_, _| {}),
    );
}

#[test]
fn orca_parity_bbl_eager_enforce_top_only_complete_output() {
    native_stream_case::check(
        SOURCE,
        "top-only",
        include_bytes!("../../../../../tests/parity/bbl-eager-enforce/top-only.3mf"),
        "bd42ee78a736e7916ae8b4c549c8c52f45c6522b89214d1029d2701b9039bc77",
        (false, |_, _| {}),
    );
}

#[test]
fn orca_parity_bbl_eager_enforce_top_and_bottom_complete_output() {
    native_stream_case::check(
        SOURCE,
        "top-and-bottom",
        include_bytes!("../../../../../tests/parity/bbl-eager-enforce/top-and-bottom.3mf"),
        "792645d56b67be089d137bc95ce7b76827fbef882782774c235ddc3974ec63c3",
        (false, |_, _| {}),
    );
}
