//! Klipper `M204` sets the live acceleration without profile clamping
//! (`docs/architecture/klipper-m204-acceleration-spec.md`).
use super::{inverse, settings, strict};
use crate::runner::{OrcaRunner, repo_root};
use crate::{CaseInputs, VendorProfiles};
use ares_core::gcode_processor::process_gcode;
use serde_json::{Map, Value, json};

#[test]
fn klipper_m204_sets_live_acceleration_processor_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let mut above_limit = Map::new();
    above_limit.insert(
        "machine_max_acceleration_extruding".into(),
        json!(["1000", "1000"]),
    );
    let reference = native(
        &runner,
        ("Creality", "Creality K1 (0.4 nozzle)"),
        above_limit,
    );
    let text = std::str::from_utf8(&reference).unwrap();
    let limit = setting(text, "machine_max_acceleration_extruding")
        .split(',')
        .next()
        .unwrap()
        .parse::<f32>()
        .unwrap();
    let m204 = first_m204_before_accel_limit(text);
    assert!(
        word(m204, 'S').is_some_and(|value| value > limit),
        "K1 must request an M204 S above the extruding limit {limit}: {m204:?}"
    );
    replay(&runner, &reference);

    let reference = native(
        &runner,
        ("Anycubic", "Anycubic Kobra S1 Max 0.25 nozzle"),
        Map::new(),
    );
    let text = std::str::from_utf8(&reference).unwrap();
    let m204 = first_m204_before_accel_limit(text);
    assert!(
        word(m204, 'P').is_some() && word(m204, 'S').is_none() && word(m204, 'T').is_none(),
        "Kobra must request an M204 P without T: {m204:?}"
    );
    replay(&runner, &reference);
}

fn native(
    runner: &OrcaRunner,
    (vendor, printer): (&str, &str),
    extra: Map<String, Value>,
) -> Vec<u8> {
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), vendor).unwrap();
    let selection = crate::select_printer(&profiles, vendor, printer).unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let mut overrides = crate::smoke_case_overrides(&machine, &process);
    overrides.extend(extra);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &format!("klipper-m204/{printer}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &overrides,
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    runner.slice_case(&exported).unwrap().reference
}

fn replay(runner: &OrcaRunner, reference: &[u8]) {
    let text = std::str::from_utf8(reference).unwrap();
    let config = settings(text);
    assert_eq!(
        config.printer.gcode.gcode_flavor,
        ares_core::GCodeFlavor::Klipper
    );
    let input = inverse(text, false);
    let processed = process_gcode(&input, &config, false, true);
    strict(runner, &input, reference, &processed.gcode);
}

fn setting<'a>(text: &'a str, key: &str) -> &'a str {
    let prefix = format!("; {key} = ");
    text.lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
}

fn word(command: &str, letter: char) -> Option<f32> {
    command
        .split_whitespace()
        .skip(1)
        .find_map(|word| word.strip_prefix(letter)?.parse().ok())
}

/// The first `M204` before the first `SET_VELOCITY_LIMIT` that sets `ACCEL`, followed
/// by an extrusion-only move before that `SET_VELOCITY_LIMIT`.
fn first_m204_before_accel_limit(text: &str) -> &str {
    let mut m204 = None;
    for line in text.lines() {
        let command = line.split(';').next().unwrap().trim();
        match command.split_whitespace().next() {
            Some("M204") if m204.is_none() => m204 = Some(command),
            Some("SET_VELOCITY_LIMIT") if command.contains(" ACCEL=") => break,
            Some("G1") if m204.is_some() && extrusion_only(command) => return m204.unwrap(),
            _ => {}
        }
    }
    panic!("no M204 followed by an extrusion-only move before SET_VELOCITY_LIMIT ACCEL: {m204:?}")
}

fn extrusion_only(command: &str) -> bool {
    word(command, 'E').is_some()
        && ['X', 'Y', 'Z']
            .iter()
            .all(|axis| word(command, *axis).is_none())
}
