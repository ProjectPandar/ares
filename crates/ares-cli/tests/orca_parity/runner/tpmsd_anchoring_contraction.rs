//! Fill.cpp:1477-1497 -> FillBase.cpp:105-118 -> FillTpmsD.cpp:101-160.
//! Targets retain strict bridge/wipe E differences; no rounding tolerance.

use super::{Case, Expectations, verify};

const MULTILINE2: Expectations = Expectations {
    input_dir: "tests/parity/tpmsd-anchoring-contraction",
    label: "tpmsd-anchoring-contraction",
    pattern: ares_core::ProcessInfillPattern::TpmsD,
    multiline: 2,
    footer: ["; sparse_infill_pattern = tpmsd", "; fill_multiline = 2"],
};
const MULTILINE1: Expectations = Expectations {
    multiline: 1,
    footer: ["; sparse_infill_pattern = tpmsd", "; fill_multiline = 1"],
    ..MULTILINE2
};

const TOP6_M2: Case = Case {
    name: "top6-multiline2",
    sha: "1e250882f206fd72c46c8a89349cd0e6ce79c3cfe392b819c188c0b3a87bc46f",
    top: 6,
    enabled: false,
    flag: false,
};
const TOP7_M2: Case = Case {
    name: "top7-multiline2",
    sha: "cb6346529e9aa6455c46b90d33a95b2fa25d05bd09f648b9898615027e29c9c4",
    top: 7,
    enabled: false,
    flag: false,
};
const TOP5_M2: Case = Case {
    name: "top5-multiline2",
    sha: "68062dd99ad802baab49e21b2ab2c0cd8746f576c3e7ae1193671068cf5aa63f",
    top: 5,
    enabled: false,
    flag: false,
};
const TOP6_M1: Case = Case {
    name: "top6-multiline1",
    sha: "874443522220d36b71b0c717cb8a93498540f0d018da789d1fe7be240f052989",
    top: 6,
    enabled: false,
    flag: false,
};

#[test]
fn top6_multiline2_matches_native_complete_output() {
    verify(&MULTILINE2, TOP6_M2);
}

#[test]
fn top7_multiline2_matches_native_complete_output() {
    verify(&MULTILINE2, TOP7_M2);
}

#[test]
fn top5_multiline2_phase_control_preserves_complete_output() {
    verify(&MULTILINE2, TOP5_M2);
}

#[test]
fn top6_multiline1_control_preserves_complete_output() {
    verify(&MULTILINE1, TOP6_M1);
}
