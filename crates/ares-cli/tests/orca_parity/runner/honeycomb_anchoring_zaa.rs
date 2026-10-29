//! Fill.cpp:1440-1447 -> FillBase.cpp:311-313 -> FillHoneycomb.hpp:53.
//! Public failures: altered input/flags, absent bridge activity, unstable Native,
//! ordinary Sparse drift, API/CLI drift, or strict complete-output drift.
//! Progress/M73 remains in the unchanged whole-file acceptance comparator.

use super::{Case, Expectations, verify};

const EXPECTED: Expectations = Expectations {
    input_dir: "tests/parity/honeycomb-anchoring-zaa",
    label: "honeycomb-anchoring-zaa",
    pattern: ares_core::ProcessInfillPattern::Honeycomb,
    multiline: 2,
    footer: [
        "; sparse_infill_pattern = honeycomb",
        "; fill_multiline = 2",
    ],
};

const TARGET: Case = Case {
    name: "top6-enabled-flag1",
    sha: "078df2b3f6ffa92167b8c044408b02beaf5534a7bcebdbe6bebc606b074a9812",
    top: 6,
    enabled: true,
    flag: true,
};
const FLAG_FALSE: Case = Case {
    name: "top6-enabled-flag0",
    sha: "c1a68d0452738de551592b6bebdce8db631379320c3d25ea6f74c4c4a54f66f5",
    top: 6,
    enabled: true,
    flag: false,
};
const DISABLED_ZAA: Case = Case {
    name: "top6-disabled-flag1",
    sha: "662a314cc90f2a83c3a79de9e4c742b7bc29cd6eea30975bded1e1037079fbd7",
    top: 6,
    enabled: false,
    flag: true,
};
const PHASE_ZERO: Case = Case {
    name: "top7-enabled-flag1",
    sha: "6604daa38d5c7dd73f39311c7094e988e0b87317968ceab9d817650497d669df",
    top: 7,
    enabled: true,
    flag: true,
};
const PHASE_TWO: Case = Case {
    name: "top8-enabled-flag1",
    sha: "a838438b5bd0b7c5895ef5b167f77bee0cce1519a7babe3425bc6ad587e12753",
    top: 8,
    enabled: true,
    flag: true,
};

#[test]
fn top6_regional_direction_matches_native_complete_output() {
    verify(&EXPECTED, TARGET);
}

#[test]
fn flag_false_preserves_anchoring_and_complete_output() {
    verify(&EXPECTED, FLAG_FALSE);
}

#[test]
fn disabled_zaa_preserves_anchoring_and_complete_output() {
    verify(&EXPECTED, DISABLED_ZAA);
}

#[test]
fn phase_zero_preserves_anchoring_and_complete_output() {
    verify(&EXPECTED, PHASE_ZERO);
}

#[test]
fn phase_two_preserves_anchoring_and_complete_output() {
    verify(&EXPECTED, PHASE_TWO);
}
