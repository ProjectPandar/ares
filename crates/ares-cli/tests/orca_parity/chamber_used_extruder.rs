//! GCode.cpp:3008–3013,3037,3130–3133,3498–3499: writer-used chamber reduction.
//! Success-required public RED; failure inventory: chamber-used-extruder-spec.md.

use crate::used_extruder_case::{Witness, preflight};

const UPSTREAM: &[&str] = &[
    "GCode.cpp:3008-3013,3037,3130-3133,3498-3499",
    "GCodeWriter.cpp:192-212;GCode.cpp:2760,2791,5646-5648",
    "Tab.cpp:2671-2677,3955-3975,4357,4576,4595-4604",
];

const CASES: [Witness; 4] = [
    Witness {
        label: "chamber-t1-equal",
        sha: "29064c553748e6b59f52041359ed123964570304517215ac86053fc9edd384cc",
        tool: "1",
    },
    Witness {
        label: "chamber-t1-unused-t0-hot",
        sha: "644dc85eb009c0d11926461d8ab4eb7e000238a7e1e91ff3ee4dc018752f5e0f",
        tool: "1",
    },
    Witness {
        label: "chamber-t1-unused-t0-enabled",
        sha: "dfd4b243129b813032b2828cb258e4e21e2d995e5894f80cc61f26197ba4643d",
        tool: "1",
    },
    Witness {
        label: "chamber-t0-hot-used-control",
        sha: "cce1688b2dc25507c351f6c6d003816710fa45928ee55d0df170e70d53ae5af4",
        tool: "0",
    },
];

#[test]
fn chamber_equal() {
    CASES[0].check("chamber-used-extruder", UPSTREAM);
}

#[test]
fn chamber_unused_hot() {
    CASES[1].check("chamber-used-extruder", UPSTREAM);
}

#[test]
fn chamber_unused_enabled() {
    CASES[2].check("chamber-used-extruder", UPSTREAM);
}

#[test]
fn chamber_used_hot_control() {
    CASES[3].check("chamber-used-extruder", UPSTREAM);
}

#[test]
fn chamber_native_preflight() {
    preflight("chamber-used-extruder", UPSTREAM, &CASES);
}

const TEMPLATE_TEMPERATURE_SOURCES: &[&str] = &[
    "GCode.cpp:4001-4013,4031-4034",
    "GCode.cpp:4230-4231,4381,4461,4867-4878,6140-6142,6227,6698",
    "Config.hpp:676-680",
];

#[test]
fn templates_reduce_temperatures_over_used_tools() {
    Witness {
        label: "unused-slot",
        sha: "bda1bb833a2afa578076f98b83d73b14bf8c7242564ca9f0e9b129d6da51cf08",
        tool: "1",
    }
    .check(
        "writer-temperature-placeholders",
        TEMPLATE_TEMPERATURE_SOURCES,
    );
}

#[test]
fn templates_keep_zero_chamber_temperature() {
    Witness {
        label: "zero-chamber",
        sha: "ff4139e6e62c2d5630fd8959945dba090b286d72063f65ca51040b04bbe67a9a",
        tool: "1",
    }
    .check(
        "writer-temperature-placeholders",
        TEMPLATE_TEMPERATURE_SOURCES,
    );
}
