//! Fill.cpp:1440-1447 -> FillBase.cpp:311-313 -> FillLine.hpp:12-25 / FillBase.hpp:211.
//! Public failures: altered input/flags, absent bridge activity, unstable Native,
//! ordinary Sparse drift, API/CLI drift, or strict complete-output drift.
//! Progress/M73 and feedrates remain in the unchanged whole-file comparator.

use super::{Case, Expectations, verify};

const EXPECTED: Expectations = Expectations {
    input_dir: "tests/parity/line-anchoring-zaa",
    label: "line-anchoring-zaa",
    pattern: ares_core::ProcessInfillPattern::Line,
    multiline: 1,
    footer: ["; sparse_infill_pattern = line", "; fill_multiline = 1"],
};

const TARGET: Case = Case {
    name: "top6-enabled-flag1",
    sha: "ce4695fd7095f07426733d26400c6d93ae8cab0fee0b04fe5fd5bed98d5f310f",
    top: 6,
    enabled: true,
    flag: true,
};
const FLAG_FALSE: Case = Case {
    name: "top6-enabled-flag0",
    sha: "aea1bfca56737f4b58e2b2321bb90d067b814c9b406960b15ca6475376591472",
    top: 6,
    enabled: true,
    flag: false,
};
const DISABLED_ZAA: Case = Case {
    name: "top6-disabled-flag1",
    sha: "f16bbca3785434c6e41e390e27ff3f4b87217413f7bfa694aa69fa4f40c7e3b8",
    top: 6,
    enabled: false,
    flag: true,
};
const PHASE_TOP5: Case = Case {
    name: "top5-enabled-flag1",
    sha: "fbe15bc57cac6a87b53e2afe4a447420bf991e1f24b87c465eff7e95e7493397",
    top: 5,
    enabled: true,
    flag: true,
};
const PHASE_TOP7: Case = Case {
    name: "top7-enabled-flag1",
    sha: "2211372b011e9f8ff10e8548d52fd56c57144e97479968212c9f7b34461b6756",
    top: 7,
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
fn top5_even_phase_preserves_anchoring_and_complete_output() {
    verify(&EXPECTED, PHASE_TOP5);
}

#[test]
fn top7_even_phase_preserves_anchoring_and_complete_output() {
    verify(&EXPECTED, PHASE_TOP7);
}
