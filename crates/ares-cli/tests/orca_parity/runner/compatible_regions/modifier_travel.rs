use super::{CaseInputs, ExportedCase, OrcaRunner, application, parity, repo_root, volume_fixture};
use crate::presets::VendorProfiles;
use serde_json::json;

#[test]
fn modifier_preserves_layer_63_native_feedrate() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs(true);
    let expected = layer_63_object(&reference);
    assert!(expected.contains(";TYPE:Internal solid infill\n;WIDTH:0.419999\nG1 F5093\n"));
    assert_eq!(layer_63_object(&actual), expected);
}

#[test]
fn modifier_without_wall_routing_preserves_layer_63_native_feedrate() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs(false);
    let expected = layer_63_object(&reference);
    assert!(expected.contains(";TYPE:Internal solid infill\n;WIDTH:0.419999\nG1 F"));
    assert_eq!(layer_63_object(&actual), expected);
}

#[test]
fn modifier_without_cooling_slowdown_preserves_layer_63_native_feedrate() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs_with_cooling(true, false);
    let expected = layer_63_object(&reference);
    assert!(expected.contains(";TYPE:Internal solid infill\n;WIDTH:0.419999\nG1 F"));
    assert!(!expected.contains("G1 F5093\n"));
    assert_eq!(layer_63_object(&actual), expected);
}

#[test]
fn modifier_without_cooling_slowdown_preserves_complete_layer_63_motion() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs_with_cooling(true, false);
    let expected = layer_63(&reference);
    assert!(expected.contains("G1 X175.61 Y104.41 F18000\nG1 X175.59 Y104.39\n"));
    assert_eq!(layer_63(&actual), expected);
}

#[test]
fn modifier_without_cooling_or_wall_routing_preserves_complete_layer_63_motion() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs_with_cooling(false, false);
    let expected = layer_63(&reference);
    assert!(!expected.contains("G1 X175.61 Y104.41 F18000\nG1 X175.59 Y104.39\n"));
    assert!(expected.contains(";TYPE:Outer wall\n"));
    assert_eq!(layer_63(&actual), expected);
}

#[test]
fn modifier_preserves_final_use_cooldown_section() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs(true);
    let expected = cooldown_section(&reference);
    assert!(expected.contains("M104 S0 T0 ; set nozzle temperature ;cooldown\n"));
    assert_eq!(cooldown_section(&actual), expected);
}

fn cooldown_section(bytes: &[u8]) -> &str {
    let text = std::str::from_utf8(bytes).unwrap();
    let start = text
        .rfind("M104 S0 T0 ; set nozzle temperature ;cooldown\n")
        .unwrap();
    text[start..].split_once("; CONFIG_BLOCK_START").unwrap().0
}

fn layer_63(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes)
        .unwrap()
        .split_once(";Z:10.28\n")
        .unwrap()
        .1
        .split_once(";LAYER_CHANGE\n")
        .unwrap()
        .0
}

fn layer_63_object(bytes: &[u8]) -> &str {
    layer_63(bytes)
        .split_once("; printing object overhang.stl id:0 copy 0\n")
        .unwrap()
        .1
        .split_once("; stop printing object overhang.stl id:0 copy 0\n")
        .unwrap()
        .0
}

#[test]
fn modifier_preserves_ninth_layer_native_approach() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs(true);
    let expected = ninth_layer_approach(&reference);
    assert!(expected.contains("G1 X165.821 Y104.78\n"));
    assert_eq!(ninth_layer_approach(&actual), expected);
}

#[test]
fn modifier_without_wall_routing_preserves_ninth_layer_native_approach() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs(false);
    let expected = ninth_layer_approach(&reference);
    assert!(expected.contains("G1 X164.539 Y102.539"));
    assert!(!expected.contains("G1 X165.821 Y104.78\n"));
    assert_eq!(ninth_layer_approach(&actual), expected);
}

fn ninth_layer_approach(bytes: &[u8]) -> &str {
    let layer = std::str::from_utf8(bytes)
        .unwrap()
        .split_once(";Z:1.48\n")
        .unwrap()
        .1
        .split_once(";LAYER_CHANGE\n")
        .unwrap()
        .0;
    layer
        .split_once("; printing object overhang.stl id:0 copy 0\n")
        .unwrap()
        .1
        .split_once("G1 E.8 F1500\n")
        .unwrap()
        .0
}

#[test]
fn modifier_preserves_fourth_layer_outer_wall_exit() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs(true);
    let expected = fourth_layer_object(&reference);
    assert!(expected.ends_with("G1 X166.79 Y104.781\n"));
    assert_eq!(fourth_layer_object(&actual), expected);
}

#[test]
fn modifier_without_wall_routing_preserves_fourth_layer_outer_wall_exit() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs(false);
    let expected = fourth_layer_object(&reference);
    assert!(expected.ends_with("G1 X166.79 Y104.781\n"));
    assert_eq!(fourth_layer_object(&actual), expected);
}

fn fourth_layer_object(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes)
        .unwrap()
        .split(";Z:0.84\n")
        .nth(1)
        .unwrap()
        .split("; stop printing object overhang.stl id:0 copy 0\n")
        .next()
        .unwrap()
}

#[test]
fn modifier_preserves_second_layer_native_approach() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs(true);
    let expected = approach(&reference);
    assert!(expected.contains("G3 Z.92"));
    assert!(expected.contains("G1 X165.499 Y103.499"));
    assert_eq!(approach(&actual), expected);
}

#[test]
fn modifier_without_wall_routing_preserves_second_layer_native_approach() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (reference, actual) = outputs(false);
    let expected = approach(&reference);
    assert!(expected.contains("G3 Z.92"));
    assert!(expected.contains("G1 X165.499 Y103.499"));
    assert_eq!(approach(&actual), expected);
}

fn approach(bytes: &[u8]) -> String {
    let text = std::str::from_utf8(bytes).unwrap();
    let layer = text.split(";Z:0.52\n").nth(1).unwrap();
    let travel = layer.split("M204 T10000\n").nth(1).unwrap();
    travel.split("G1 E.8 F1500\n").next().unwrap().to_owned()
}

fn outputs(reduce_crossing_wall: bool) -> (Vec<u8>, Vec<u8>) {
    outputs_with_cooling(reduce_crossing_wall, true)
}

fn outputs_with_cooling(
    reduce_crossing_wall: bool,
    slow_down_for_layer_cooling: bool,
) -> (Vec<u8>, Vec<u8>) {
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Snapmaker",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Snapmaker", "Snapmaker J1 (0.4 nozzle)").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("filament_map_mode".into(), json!("Manual"));
    machine.insert("filament_map".into(), json!(["1", "2"]));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("sparse_infill_density", "0%"),
        ("sparse_infill_pattern", "rectilinear"),
        ("sparse_infill_filament_id", "2"),
        (
            "reduce_crossing_wall",
            if reduce_crossing_wall { "1" } else { "0" },
        ),
    ] {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
    parity::normalize_filament_defaults(&mut filaments);
    for (filament, colour) in filaments.iter_mut().zip(["#F2754E", "#0000FF"]) {
        filament.insert("filament_colour".into(), json!([colour]));
        filament.insert(
            "slow_down_for_layer_cooling".into(),
            json!([if slow_down_for_layer_cooling {
                "1"
            } else {
                "0"
            }]),
        );
    }
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "compatible-regions/modifier-travel",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    let exported = runner
        .bind_filament_map_mode(exported, &json!("Manual"))
        .unwrap();
    let project = volume_fixture::add_volumes(
        &exported.project,
        "modifier_part",
        &[(0.6, -6.0)],
        &[("sparse_infill_density", "50%")],
    );
    application::verify_value(
        &project,
        "reduce_crossing_wall",
        &json!(if reduce_crossing_wall { "1" } else { "0" }),
    )
    .unwrap();
    application::verify_value(
        &project,
        "slow_down_for_layer_cooling",
        &json!(vec![
            if slow_down_for_layer_cooling {
                "1"
            } else {
                "0"
            };
            2
        ]),
    )
    .unwrap();
    let project_path = runner.work.join("modifier-travel.3mf");
    let output_dir = runner.work.join("modifier-travel-native");
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    let sliced = runner
        .slice_case(&ExportedCase {
            label: exported.label,
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    let outcome = parity::compare_case(&sliced);
    eprintln!("complete output: {}: {}", outcome.status, outcome.detail);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let actual = runtime
        .block_on(crate::runner::application::slice_project(
            &sliced.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    (sliced.reference, actual)
}
