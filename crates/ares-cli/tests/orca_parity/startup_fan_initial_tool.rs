//! GCode::_do_export, GCode.cpp:2822–2826: initial-filament startup fan-off gate.
//! Success-required public RED; failure inventory: startup-fan-initial-tool-spec.md.

use crate::used_extruder_case::{Witness, preflight};

const UPSTREAM: &[&str] = &[
    "GCode.cpp:2822-2826;GCodeWriter.cpp:1095-1155;Config.hpp:624-627",
    "Tab.cpp:4575,4079,4301,2671-2677",
];

const CASES: [Witness; 3] = [
    Witness {
        label: "aux-on-both-open",
        sha: "75fb14cec1cdab283cc62db7cfe0141093950930a03826f64bc8881b46c7069f",
        tool: "1",
    },
    Witness {
        label: "aux-on-initial-t1-closed",
        sha: "8305d6fcd9f2f9f04199cc17070a4b167b419ab9b1e8d5ffef213b23d44d11d6",
        tool: "1",
    },
    Witness {
        label: "aux-off-initial-t1-closed",
        sha: "c8b08779a0204b8cf2809c04d34c2bab6f171fcacb0deb6ad6f7b3ac3622164f",
        tool: "1",
    },
];

#[test]
fn orca_parity_startup_fan_open_control() {
    CASES[0].check("startup-fan-initial-tool", UPSTREAM);
}

#[test]
fn orca_parity_startup_fan_initial_t1_closed() {
    CASES[1].check("startup-fan-initial-tool", UPSTREAM);
}

#[test]
fn orca_parity_startup_fan_no_aux_control() {
    CASES[2].check("startup-fan-initial-tool", UPSTREAM);
}

#[test]
fn orca_parity_startup_fan_native_preflight() {
    preflight("startup-fan-initial-tool", UPSTREAM, &CASES);
}
