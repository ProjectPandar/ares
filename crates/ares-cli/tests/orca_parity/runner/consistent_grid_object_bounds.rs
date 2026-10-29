//! PrintObject.cpp:95-112 -> Fill.cpp -> FillRectilinear.cpp:2741-2807.
//! One shared frame boundary for ZigZag, CrossZag and LockedZag.

use super::{Case, Expectations, object_bounds::check};

const ZIGZAG: Expectations = Expectations {
    input_dir: "tests/parity/consistent-grid-object-bounds",
    label: "consistent-grid-object-bounds",
    pattern: ares_core::ProcessInfillPattern::ZigZag,
    multiline: 1,
    footer: ["; sparse_infill_pattern = zigzag", "; fill_multiline = 1"],
};
const CROSSZAG: Expectations = Expectations {
    pattern: ares_core::ProcessInfillPattern::CrossZag,
    footer: ["; sparse_infill_pattern = crosszag", "; fill_multiline = 1"],
    ..ZIGZAG
};
const LOCKEDZAG: Expectations = Expectations {
    pattern: ares_core::ProcessInfillPattern::LockedZag,
    footer: [
        "; sparse_infill_pattern = lockedzag",
        "; fill_multiline = 1",
    ],
    ..ZIGZAG
};
const RECTILINEAR: Expectations = Expectations {
    pattern: ares_core::ProcessInfillPattern::Rectilinear,
    footer: [
        "; sparse_infill_pattern = rectilinear",
        "; fill_multiline = 1",
    ],
    ..ZIGZAG
};

const ZIGZAG08: Case = Case {
    name: "zigzag-compensation08",
    sha: "2764c00429118e2285aeac74e36242c93fb0f6e3b2841d6576527a11abc3f8c2",
    top: 6,
    enabled: false,
    flag: false,
};
const ZIGZAG0: Case = Case {
    name: "zigzag-compensation0",
    sha: "e413f331e008147446c4ef63ddda32d5c8c1b8694eedace21948c2df1829bb76",
    ..ZIGZAG08
};
const ZIGZAG01: Case = Case {
    name: "zigzag-compensation01",
    sha: "b9bba1156611846a0e1aef3efc56b8520403cf4307e2129fb1c0cc9b23bfd7fa",
    ..ZIGZAG08
};
const CROSSZAG08: Case = Case {
    name: "crosszag-compensation08",
    sha: "2f1e3004772244b83ed0aa8df94858f9e2e7d95ec225bfdca471e1a503ccd077",
    ..ZIGZAG08
};
const CROSSZAG0: Case = Case {
    name: "crosszag-compensation0",
    sha: "1531d7401eb1f9dfa7d99beda680da61ddd37ed124baef6350ebf418fde0ffd8",
    ..ZIGZAG08
};
const LOCKEDZAG08: Case = Case {
    name: "lockedzag-compensation08",
    sha: "e4d5d41399bdd49f59dee8942b84d0d72802f05651158eb9ec3268a1c76b7d6e",
    ..ZIGZAG08
};
const LOCKEDZAG0: Case = Case {
    name: "lockedzag-compensation0",
    sha: "1ff705f79f4b97bdc6e8e8d80bb56d7412e7bbc3c4ad2293a2d0af3b4988cf03",
    ..ZIGZAG08
};
const RECTILINEAR08: Case = Case {
    name: "rectilinear-compensation08",
    sha: "851a673b2a5db02a5f184fcf6c6f5053eb0ca262b78bee68a31f801b34715a03",
    ..ZIGZAG08
};

#[test]
fn zigzag_compensated_object_bounds_match_native_complete_output() {
    check(&ZIGZAG, ZIGZAG08, 0.8);
}

#[test]
fn crosszag_compensated_object_bounds_match_native_complete_output() {
    check(&CROSSZAG, CROSSZAG08, 0.8);
}

#[test]
fn lockedzag_compensated_object_bounds_match_native_complete_output() {
    check(&LOCKEDZAG, LOCKEDZAG08, 0.8);
}

#[test]
fn zigzag_uncompensated_control_matches_native_complete_output() {
    check(&ZIGZAG, ZIGZAG0, 0.0);
}

#[test]
fn zigzag_below_grid_boundary_control_matches_native_complete_output() {
    check(&ZIGZAG, ZIGZAG01, 0.1);
}

#[test]
fn crosszag_uncompensated_control_matches_native_complete_output() {
    check(&CROSSZAG, CROSSZAG0, 0.0);
}

#[test]
fn lockedzag_uncompensated_control_matches_native_complete_output() {
    check(&LOCKEDZAG, LOCKEDZAG0, 0.0);
}

#[test]
fn rectilinear_compensated_control_matches_native_complete_output() {
    check(&RECTILINEAR, RECTILINEAR08, 0.8);
}
