//! GCode.cpp3181–3196/3501–3516 at8500fcdcc: writer-used exhaust fan reduction.
//! Success-required public RED; failure inventory: exhaust-used-extruder-spec.md.

use crate::used_extruder_case::{Witness, preflight};

const UPSTREAM: &[&str] = &[
    "GCode.cpp:3181-3196,3501-3516,2760,2791,5646-5648",
    "GCodeWriter.cpp:63-80,1157-1171;Print.cpp:522-547",
    "PrintConfig.cpp:1886-1921,3899-3904;Tab.cpp:4108-4123,4304-4312,4577",
];

const CASES: [Witness; 4] = [
    Witness {
        label: "unused-high",
        sha: "335014559fc941ce8519926d2ae50c00ef9b8c3df5f67fa2b5359be3b6b9d297",
        tool: "0",
    },
    Witness {
        label: "equal",
        sha: "94c7e72870e46fbb42b481daf45497780434e02dfb929fec0aa0503dcf8635e6",
        tool: "0",
    },
    Witness {
        label: "unused-disabled",
        sha: "4663aa8d224711c8c025db0de5851ca6cedfc7faf3e2fb5d7f3d744850c5ce3a",
        tool: "0",
    },
    Witness {
        label: "capability-off",
        sha: "4da717b2ae507a446782e5265a4803a9aae687e096fde757cedde6adf7f9c1f5",
        tool: "0",
    },
];

#[test]
fn orca_parity_exhaust_unused_high_speed() {
    CASES[0].check("exhaust-used-extruder", UPSTREAM);
}

#[test]
fn orca_parity_exhaust_equal_speed() {
    CASES[1].check("exhaust-used-extruder", UPSTREAM);
}

#[test]
fn orca_parity_exhaust_unused_disabled() {
    CASES[2].check("exhaust-used-extruder", UPSTREAM);
}

#[test]
fn orca_parity_exhaust_capability_off() {
    CASES[3].check("exhaust-used-extruder", UPSTREAM);
}

#[test]
fn orca_parity_exhaust_native_preflight() {
    preflight("exhaust-used-extruder", UPSTREAM, &CASES);
}
