//! Junction-deviation first-layer emission (`GCode.cpp:4772-4773`,
//! emitter `GCodeWriter.cpp:352-361`): with
//! `gcode_flavor = marlin2` and `default_junction_deviation > 0` Native
//! writes `M205 J<value with 3 fixed decimals>` at the first layer —
//! clamped to `machine_max_junction_deviation` — re-emitted at every
//! sequential object's first layer (`GCode.cpp:4589`/`:3367`). Other
//! flavors and the zero default emit nothing (the machine-envelope
//! start line is a separate already-ported block). Ares never emits
//! the first-layer line, so the four target legs must FAIL on the
//! strict whole-output compare (the first difference is the layer-1
//! `M205 J` line); the zero and flavor controls must PASS.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const BELOW_LIMIT: &[u8] =
    include_bytes!("../../../../tests/parity/junction-deviation/below-limit.3mf");
const BELOW_LIMIT_SHA256: &str = "b42d7a1062a2395eb5e1ef60a91916100c4443a9bf40f65b898e55ddf73c1d78";
const AT_LIMIT: &[u8] = include_bytes!("../../../../tests/parity/junction-deviation/at-limit.3mf");
const AT_LIMIT_SHA256: &str = "2693695ca738261b5cea2cd8c7dd46c8cf7fc19066103030e0e42cea5b9f5281";
const CLAMP: &[u8] = include_bytes!("../../../../tests/parity/junction-deviation/clamp.3mf");
const CLAMP_SHA256: &str = "da109bf511889ec03a5c62bfc34589345fb4afec68f80ada9a74874ba3ba36db";
const BY_OBJECT: &[u8] =
    include_bytes!("../../../../tests/parity/junction-deviation/by-object.3mf");
const BY_OBJECT_SHA256: &str = "a5b3a3160b3457329a2f544c6d227767f90b3d0eb1f74d6e5a81eb775130fd18";
const ORDERING: &[u8] = include_bytes!("../../../../tests/parity/junction-deviation/ordering.3mf");
const ORDERING_SHA256: &str = "73d8da6121263ee8d179f5320dd2573ffc8de0ef9763e25c8b87f975a4e99ecd";
const ZERO_CONTROL: &[u8] =
    include_bytes!("../../../../tests/parity/junction-deviation/zero-control.3mf");
const ZERO_CONTROL_SHA256: &str =
    "b7f3c689d8d6f72625bee341b437bc47dcb9ac5b107b02873fbe225f2cf29013";
const FLAVOR_CONTROL: &[u8] =
    include_bytes!("../../../../tests/parity/junction-deviation/flavor-control.3mf");
const FLAVOR_CONTROL_SHA256: &str =
    "025dbbc703e534501db7ee56adb8e798ea27c3711c98b0713370312d142e9b2e";

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn slice_native(runner: &OrcaRunner, root: &std::path::Path, input: &[u8], prefix: &str) -> String {
    let directory = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("orca");
    fs::write(&path, input).unwrap();
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        output.to_str().unwrap().to_owned(),
        path.to_str().unwrap().to_owned(),
    ];
    stages::run(
        runner.bin(),
        args.iter().cloned(),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    fs::read_to_string(output.join("plate_1.gcode")).unwrap()
}

fn compare(
    root: &std::path::Path,
    label: &str,
    input: &[u8],
    reference: &str,
    source: &str,
) -> crate::ParityOutcome {
    crate::artifacts::compare(
        root,
        label,
        Ok(input),
        Ok(reference.as_bytes()),
        json!({
            "kind":"junction_deviation",
            "input_sha256":sha256(input),
            "source":source,
        }),
    )
    .unwrap()
}

#[test]
fn junction_deviation_below_limit_matches_native() {
    assert_eq!(sha256(BELOW_LIMIT), BELOW_LIMIT_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, BELOW_LIMIT, "jd-below-limit-");
    assert_eq!(
        reference.lines().filter(|l| *l == "M205 J0.080").count(),
        1,
        "machine-envelope start line"
    );
    assert_eq!(
        reference.lines().filter(|l| *l == "M205 J0.005").count(),
        1,
        "first-layer line, unclamped"
    );
    let outcome = compare(
        &root,
        "jd-below-limit",
        BELOW_LIMIT,
        &reference,
        "GCode.cpp:4772-4773;GCodeWriter.cpp:352-361",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn junction_deviation_at_limit_matches_native() {
    assert_eq!(sha256(AT_LIMIT), AT_LIMIT_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, AT_LIMIT, "jd-at-limit-");
    assert_eq!(reference.lines().filter(|l| *l == "M205 J0.080").count(), 2);
    let outcome = compare(
        &root,
        "jd-at-limit",
        AT_LIMIT,
        &reference,
        "GCode.cpp:4772-4773;GCodeWriter.cpp:352-361",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn junction_deviation_clamp_matches_native() {
    assert_eq!(sha256(CLAMP), CLAMP_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, CLAMP, "jd-clamp-");
    // default_junction_deviation = 0.2 clamps to machine_max 0.08.
    assert_eq!(reference.lines().filter(|l| *l == "M205 J0.080").count(), 2);
    let outcome = compare(
        &root,
        "jd-clamp",
        CLAMP,
        &reference,
        "GCode.cpp:4772-4773;GCodeWriter.cpp:352-361",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn junction_deviation_by_object_matches_native() {
    assert_eq!(sha256(BY_OBJECT), BY_OBJECT_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, BY_OBJECT, "jd-by-object-");
    assert_eq!(
        reference
            .lines()
            .filter(|l| *l == "M1003 S1" || *l == "M1003 S0")
            .count(),
        0,
        "PLR neutralized to printer_configuration"
    );
    assert_eq!(
        reference.lines().filter(|l| *l == "M205 J0.005").count(),
        2,
        "one first-layer line per sequential object"
    );
    assert_eq!(
        reference.lines().filter(|l| *l == "M205 J0.080").count(),
        1,
        "machine-envelope start line"
    );
    let outcome = compare(
        &root,
        "jd-by-object",
        BY_OBJECT,
        &reference,
        "GCode.cpp:4772-4773,4589,3367;GCodeWriter.cpp:352-361",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn junction_deviation_ordering_matches_native() {
    assert_eq!(sha256(ORDERING), ORDERING_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, ORDERING, "jd-ordering-");
    // GCode.cpp:4761-4774: within the first-layer block the initial
    // acceleration, the initial jerk, and the junction deviation emit
    // in that order (M204 -> M205 X/Y -> M205 J), after the layer
    // change marker and before the layer-change retract.
    let first_layer: &str = &reference[reference.find(";_SET_FAN_SPEED_CHANGING_LAYER").unwrap()..];
    let m204 = first_layer.find("M204 P500\n").unwrap();
    let jerk = first_layer[m204..].find("M205 X8 Y8\n").unwrap() + m204;
    let junction = first_layer[jerk..].find("M205 J0.005\n").unwrap() + jerk;
    assert!(
        m204 < jerk && jerk < junction,
        "ordering {m204} {jerk} {junction}"
    );
    let outcome = compare(
        &root,
        "jd-ordering",
        ORDERING,
        &reference,
        "GCode.cpp:4761-4774;GCodeWriter.cpp:352-361",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn junction_deviation_zero_control_matches_native() {
    assert_eq!(sha256(ZERO_CONTROL), ZERO_CONTROL_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, ZERO_CONTROL, "jd-zero-control-");
    assert_eq!(
        reference.lines().filter(|l| *l == "M205 J0.080").count(),
        1,
        "envelope line only, no first-layer line"
    );
    let outcome = compare(
        &root,
        "jd-zero-control",
        ZERO_CONTROL,
        &reference,
        "GCode.cpp:4772-4773;GCodeWriter.cpp:352-361",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn junction_deviation_flavor_control_matches_native() {
    assert_eq!(sha256(FLAVOR_CONTROL), FLAVOR_CONTROL_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, FLAVOR_CONTROL, "jd-flavor-control-");
    assert_eq!(
        reference
            .lines()
            .filter(|l| l.starts_with("M205 J"))
            .count(),
        0,
        "legacy flavor: neither envelope nor first-layer line"
    );
    let outcome = compare(
        &root,
        "jd-flavor-control",
        FLAVOR_CONTROL,
        &reference,
        "GCode.cpp:4772-4773;GCodeWriter.cpp:352-361",
    );
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
