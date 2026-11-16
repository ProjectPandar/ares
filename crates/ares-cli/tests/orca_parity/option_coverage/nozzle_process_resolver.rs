//! Print.cpp:5113–5184; PrintConfig.cpp:798–837 process-column lookup.
use crate::{OrcaRunner, presets::VendorProfiles, runner};
use serde_json::{Value, json};
use std::io::{Cursor, Read, Write};

fn project(runner: &OrcaRunner, case: &str) -> Vec<u8> {
    let singleton = case == "singleton";
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "BBL",
    )
    .unwrap();
    let printer = if singleton {
        "Bambu Lab X1 Carbon 0.4 nozzle"
    } else {
        "Bambu Lab H2D 0.4 nozzle"
    };
    let selected = crate::select_printer(&profiles, "BBL", printer).unwrap();
    let machine = profiles.machine(&selected.printer).unwrap();
    let mut process = profiles.process(&selected.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selected
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &runner::CaseInputs {
                label: "process-nozzle-cube10",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &runner::repo_root().join(if matches!(case, "overhang" | "support") {
                "tests/parity/overhang.stl"
            } else {
                "tests/parity/cube10.stl"
            }),
        )
        .unwrap();
    let mut source = zip::ZipArchive::new(Cursor::new(exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut member = source.by_index(index).unwrap();
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        if member.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            let (ids, variants, speeds) = match case {
                "singleton" => (
                    json!(["1"]),
                    json!(["Direct Drive Standard"]),
                    json!(["200"]),
                ),
                "matched" => (
                    json!(["1", "2"]),
                    json!(["Direct Drive Standard", "Direct Drive Standard"]),
                    json!(["500", "200"]),
                ),
                "same_id" => (
                    json!(["2", "2"]),
                    json!(["Bowden Standard", "Bowden Standard"]),
                    json!(["500", "200"]),
                ),
                "equal_speed" => (
                    json!(["7"]),
                    json!(["Direct Drive Standard", "Direct Drive Standard"]),
                    json!(["200", "200"]),
                ),
                _ => (
                    json!(["7"]),
                    json!(["Direct Drive Standard", "Direct Drive Standard"]),
                    json!(["500", "200"]),
                ),
            };
            if !singleton {
                settings["extruder_variant_list"] =
                    json!(["Direct Drive Standard", "Direct Drive Standard"]);
            }
            settings["print_extruder_id"] = ids;
            settings["print_extruder_variant"] = variants;
            settings["outer_wall_speed"] = speeds;
            settings["role_based_wipe_speed"] = json!("1");
            settings["filament_retraction_length"] = json!(["0.4"]);
            settings["filament_wipe_distance"] = json!(["1"]);
            if case == "small_loop" {
                settings["small_perimeter_threshold"] = json!(["100", "0"]);
                settings["small_perimeter_speed"] = json!(["50%", "100%"]);
            }
            if case == "skirt" {
                settings["skirt_loops"] = json!("2");
                settings["initial_layer_speed"] = json!(["25", "50"]);
                settings["initial_layer_infill_speed"] = json!(["40", "70"]);
            }
            if case == "overhang" {
                settings["overhang_1_4_speed"] = json!(["15", "30"]);
                settings["overhang_2_4_speed"] = json!(["15", "30"]);
                settings["overhang_3_4_speed"] = json!(["15", "30"]);
                settings["overhang_4_4_speed"] = json!(["15", "30"]);
            }
            if case == "support" {
                settings["enable_support"] = json!("1");
                settings["support_speed"] = json!(["80", "120"]);
                settings["support_interface_speed"] = json!(["40", "60"]);
            }
            if case == "kinematics" {
                settings["outer_wall_acceleration"] = json!(["1200", "2400"]);
                settings["outer_wall_jerk"] = json!(["5", "9"]);
                settings["initial_layer_travel_acceleration"] = json!(["1300", "2600"]);
            }
            if case == "percent_wipe" {
                settings["role_based_wipe_speed"] = json!("0");
                settings["wipe_speed"] = json!("80%");
                settings["travel_speed"] = json!(["300", "150"]);
            }
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                member.name(),
                zip::write::SimpleFileOptions::default().compression_method(member.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn wipe(gcode: &[u8]) -> Vec<String> {
    let text = std::str::from_utf8(gcode).unwrap();
    let lines = text.lines().collect::<Vec<_>>();
    let layer = lines
        .iter()
        .position(|line| *line == "; Z_HEIGHT: 0.4")
        .unwrap();
    let wall = (layer..lines.len())
        .find(|&i| lines[i] == "; FEATURE: Outer wall")
        .unwrap();
    let start = (wall..lines.len())
        .find(|&i| lines[i].contains("WIPE_START"))
        .unwrap();
    let before = (wall..start)
        .rev()
        .find(|&i| lines[i].starts_with("G1 E-"))
        .unwrap();
    let end = (start..lines.len())
        .find(|&i| lines[i].contains("WIPE_END"))
        .unwrap();
    let recovered = (end..lines.len())
        .find(|&i| lines[i].starts_with("G1 E") && !lines[i].starts_with("G1 E-"))
        .unwrap();
    lines[before..=recovered]
        .iter()
        .map(|line| (*line).to_owned())
        .collect()
}

fn compare(case: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = project(&runner, case);
    let hash = runner::native_api_cli::sha(&project);
    let (native, _) = runner::native_stream_case::qualify(
        (
            "process-nozzle-resolver",
            &["Print.cpp:5113–5184", "PrintConfig.cpp:798–837"],
        ),
        case,
        &project,
        &hash,
        false,
    );
    if case != "singleton" {
        assert!(native.native.contains("\n; filament_map = 2\n"));
        let (ids, variants, speeds) = match case {
            "matched" => (
                "1,2",
                "\"Direct Drive Standard\";\"Direct Drive Standard\"",
                "500,200",
            ),
            "same_id" => ("2,2", "\"Bowden Standard\";\"Bowden Standard\"", "500,500"),
            "equal_speed" => (
                "7,7",
                "\"Direct Drive Standard\";\"Direct Drive Standard\"",
                "200,200",
            ),
            _ => (
                "7,7",
                "\"Direct Drive Standard\";\"Direct Drive Standard\"",
                "500,200",
            ),
        };
        for (key, value) in [
            ("print_extruder_id", ids),
            ("print_extruder_variant", variants),
            ("outer_wall_speed", speeds),
        ] {
            assert!(
                native.native.contains(&format!("\n; {key} = {value}\n")),
                "finalized {key}: {:?}",
                native.root
            );
        }
    }
    let native_wipe = wipe(native.native.as_bytes());
    assert!(
        native_wipe
            .iter()
            .any(|line| line.starts_with("G1 X") && line.contains(" E-"))
    );
    std::fs::write(
        native.root.join("target-wipe.json"),
        serde_json::to_vec_pretty(&native_wipe).unwrap(),
    )
    .unwrap();
    let (api, cli) = native.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
    assert_eq!(native_wipe, wipe(&api));
    assert_eq!(native_wipe, wipe(&cli));
}

#[test]
fn nozzle_process_resolver_unmatched_id() {
    compare("unmatched_id");
}
#[test]
fn nozzle_process_resolver_exact_match() {
    compare("matched");
}
#[test]
fn nozzle_process_resolver_same_id_without_variant() {
    compare("same_id");
}
#[test]
fn nozzle_process_resolver_singleton() {
    compare("singleton");
}
#[test]
fn nozzle_process_resolver_equal_speed_control() {
    compare("equal_speed");
}
#[test]
fn nozzle_process_resolver_acceleration_and_jerk() {
    compare("kinematics");
}
#[test]
fn nozzle_process_resolver_percentage_wipe_keeps_physical_travel() {
    compare("percent_wipe");
}
#[test]
fn nozzle_process_resolver_small_loop_threshold() {
    compare("small_loop");
}
#[test]
fn nozzle_process_resolver_initial_layer_and_skirt() {
    compare("skirt");
}
#[test]
fn nozzle_process_resolver_overhang_speeds() {
    compare("overhang");
}
#[test]
fn nozzle_process_resolver_support_speeds() {
    compare("support");
}
