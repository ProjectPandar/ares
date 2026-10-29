//! Plate custom G-code emission (slice C,
//! `docs/architecture/plate-custom-gcode-emission.md`):
//! `ToolOrdering::assign_custom_gcodes` (`ToolOrdering.cpp:1454-1516`) and
//! `ProcessLayer::emit_custom_gcode_per_print_z` (`GCode.cpp:4174-4258`,
//! called at `:4825-4828`), as ordered bytes against Native in both modes.
//! Each case also checks the Native witness first: how many pause/custom
//! tags Native writes and which payload lines it keeps or drops.
//!
//! Failure inventory: midpoint off by a layer or `>=`; an event above the top
//! layer dropped; print mode from the filament count alone; the mode
//! mismatch applied to pauses; a ToolChange assigned; a 0/1-based mix-up in
//! `extruder_printing_above`; the `#if 0` M600 block emitted; the final `\n`
//! missing; the pause message emitted; `extra` rendered by the placeholder
//! parser; emission before the second-layer M140, after the BBL timelapse, or
//! in sequential mode.
use super::{CaseInputs, ExportedCase, OrcaRunner, ParityCase, repo_root, vg1_inputs};
use crate::VendorProfiles;
use serde_json::json;
use std::io::{Cursor, Read, Write};

const ENTRY: &str = "Metadata/custom_gcode_per_layer.xml";

/// Expected Native witness: pause tags, custom tags, payload lines present
/// and payload lines absent.
struct Witness {
    pauses: usize,
    customs: usize,
    present: &'static [&'static str],
    absent: &'static [&'static str],
}

const NONE: Witness = Witness {
    pauses: 0,
    customs: 0,
    present: &[],
    absent: &[],
};

const fn customs(present: &'static [&'static str], absent: &'static [&'static str]) -> Witness {
    Witness {
        pauses: 0,
        customs: 1,
        present,
        absent,
    }
}

fn check(runner: &OrcaRunner, label: &str, project: Vec<u8>, witness: &Witness) {
    let name = label.replace('/', "-");
    let project_path = runner.work.join(format!("{name}.3mf"));
    let output_dir = runner.work.join(format!("{name}-orca"));
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    let case: ParityCase = runner
        .slice_case(&ExportedCase {
            label: label.into(),
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    let native = std::str::from_utf8(&case.reference).unwrap();
    let tags = |names: [&str; 2]| {
        native
            .lines()
            .filter(|line| {
                line.strip_prefix(';')
                    .is_some_and(|tag| names.contains(&tag))
            })
            .count()
    };
    assert_eq!(
        (
            tags(["PAUSE_PRINT", " PAUSE_PRINTING"]),
            tags(["CUSTOM_GCODE", " CUSTOM_GCODE"])
        ),
        (witness.pauses, witness.customs),
        "{label}: Native pause/custom tag count"
    );
    for line in witness.present {
        assert!(
            native.lines().any(|native| native == *line),
            "{label}: Native lacks {line:?}"
        );
    }
    // Comment lines are skipped: the trailing config block prints the
    // templates themselves (`; color_change_gcode = M600`).
    for line in witness.absent {
        assert!(
            !native
                .lines()
                .any(|native| !native.starts_with(';') && native.contains(line)),
            "{label}: Native has {line:?}"
        );
    }
    let outcome = crate::compare_case(&case);
    assert_eq!(
        outcome.status, "PASS",
        "{label}: {}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

fn frozen(name: &str, witness: &Witness) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let path = match name {
        "control" => repo_root().join("tests/parity/orca_cli_ender3.3mf"),
        _ => repo_root().join(format!(
            "tests/parity/plate-custom-gcode-emission/{name}.3mf"
        )),
    };
    let project = std::fs::read(path).unwrap();
    check(
        &runner,
        &format!("plate-custom-gcode-emission/{name}"),
        project,
        witness,
    );
}

/// Two filaments, every object on filament 1: print mode `MultiAsSingle`
/// (`ToolOrdering.cpp:1464-1467`). The vg1 X1C two-filament cube10 export,
/// with `entry` added as `Metadata/custom_gcode_per_layer.xml`.
fn two_filaments(name: &str, entry: Option<&str>, witness: &Witness) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let selection =
        crate::select_printer(&profiles, "BBL", "Bambu Lab X1 Carbon 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let filament = profiles.filament(&selection.filaments[0]).unwrap();
    let mut filaments = vec![filament.clone(), filament];
    crate::normalize_filament_defaults(&mut filaments);
    let mut overrides = crate::smoke_case_overrides(&machine, &process);
    overrides.insert("filament_map".into(), json!(["1", "1"]));
    let label = format!("plate-custom-gcode-emission/{name}");
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &overrides,
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let project = vg1_inputs::bind_two_filaments(exported).project;
    let project = match entry {
        Some(xml) => with_entry(&project, xml),
        None => project,
    };
    check(&runner, &label, project, witness);
}

/// `project` with one more archive entry; every other entry is kept.
fn with_entry(project: &[u8], xml: &str) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::DEFAULT;
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        output
            .start_file(file.name(), options.compression_method(file.compression()))
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.start_file(ENTRY, options).unwrap();
    output.write_all(xml.as_bytes()).unwrap();
    output.finish().unwrap().into_inner()
}

#[test]
fn plate_custom_gcode_emission_control_matches_native() {
    frozen("control", &NONE);
}

#[test]
fn plate_custom_gcode_emission_pause_matches_native() {
    frozen(
        "pause",
        &Witness {
            pauses: 1,
            customs: 0,
            present: &["M25"],
            absent: &["Insert magnet"],
        },
    );
}

#[test]
fn plate_custom_gcode_emission_custom_matches_native() {
    frozen(
        "custom",
        &customs(&["G4 S1", "M117 custom [layer_num]"], &[]),
    );
}

#[test]
fn plate_custom_gcode_emission_template_matches_native() {
    frozen(
        "template",
        &customs(&["G4 P10"], &["[current_extruder]", "[layer_height]"]),
    );
}

#[test]
fn plate_custom_gcode_emission_color_change_matches_native() {
    frozen("color-change", &customs(&[], &["M600"]));
}

#[test]
fn plate_custom_gcode_emission_same_layer_matches_native() {
    frozen("same-layer", &customs(&["M117 second"], &["M117 first"]));
}

#[test]
fn plate_custom_gcode_emission_bounds_matches_native() {
    frozen("bounds", &customs(&["M117 above"], &["M117 below"]));
}

#[test]
fn plate_custom_gcode_emission_mode_mismatch_matches_native() {
    frozen(
        "mode-mismatch",
        &Witness {
            pauses: 1,
            customs: 0,
            present: &["M25"],
            absent: &[],
        },
    );
}

#[test]
fn plate_custom_gcode_emission_tool_change_matches_native() {
    frozen("tool-change", &customs(&["M117 after tool change"], &[]));
}

#[test]
fn plate_custom_gcode_emission_by_object_matches_native() {
    frozen("by-object", &NONE);
}

#[test]
fn plate_custom_gcode_emission_two_filament_control_matches_native() {
    two_filaments("two-filament-control", None, &NONE);
}

#[test]
fn plate_custom_gcode_emission_multi_as_single_matches_native() {
    // ColorChange for filament 2 on layer 5 is dropped (filament 2 never
    // prints at or above it); the one for filament 1 on layer 7 is kept
    // (`ToolOrdering.cpp:1506-1508`). The explicit mode keeps slice A's mode
    // check from turning `extruder > 1` into `MultiExtruder`.
    two_filaments(
        "multi-as-single",
        Some(concat!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n",
            "<custom_gcodes_per_layer>\n<plate>\n<plate_info id=\"1\"/>\n",
            "<layer top_z=\"1\" type=\"0\" extruder=\"2\" color=\"#0000FF\" extra=\"\"/>\n",
            "<layer top_z=\"1.4\" type=\"0\" extruder=\"1\" color=\"#00FF00\" extra=\"\"/>\n",
            "<mode value=\"MultiAsSingle\"/>\n</plate>\n</custom_gcodes_per_layer>",
        )),
        &customs(&[], &[]),
    );
}
