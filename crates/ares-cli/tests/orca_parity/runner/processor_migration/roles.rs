//! UsedFilaments cpp:1621-1649,5722,6036: whole map, including original f64 bits.
use super::{generator_only, settings, slice};
use crate::runner::{OrcaRunner, cache_identity};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::Value;
use std::path::Path;

const PROJECT: &[u8] =
    include_bytes!("../../../../../../tests/parity/role-filament-usage/wasm-plate1.3mf");

// Migrate slice A's subprocess policy proof with the real role entrypoint.
// Deliberately missing gdb prevents launching Native; this is not parity evidence.
#[test]
fn role_oracle_gdb_availability_follows_required_optional_policy() {
    let root = tempfile::Builder::new()
        .prefix("processor-policy-")
        .tempdir()
        .unwrap()
        .keep();
    for required in [true, false] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "runner::processor_migration::roles::native_role_map_is_bound_to_same_run_footer",
                "--nocapture",
            ])
            .env("ARES_REQUIRE_ORCA", if required { "1" } else { "0" })
            .env("ARES_ORCA_BIN", std::env::current_exe().unwrap())
            .env("ARES_ORCA_NATIVE_BIN", std::env::current_exe().unwrap())
            .env("ARES_ORCA_RESOURCES_DIR", &root)
            .env("ARES_PARITY_ARTIFACT_ROOT", &root)
            .env_remove("ARES_GDB_BIN")
            .env_remove("ARES_ROLE_ORACLE_LIVE")
            .output()
            .unwrap();
        let diagnostic = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::write(root.join(format!("required-{required}.log")), &diagnostic).unwrap();
        assert!(
            diagnostic.contains("running 1 test") && diagnostic.contains("ARES_GDB_BIN"),
            "{diagnostic}"
        );
        assert_eq!(output.status.success(), !required, "{diagnostic}");
        assert_eq!(diagnostic.contains("SKIPPED:"), !required, "{diagnostic}");
        if !required {
            assert!(diagnostic.contains("not executed"), "{diagnostic}");
        }
    }
}

fn record(capture: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(capture.join("role-map.json")).unwrap()).unwrap()
}

fn compare_roles(actual: &ares_core::PrintEstimatedStatistics, native: &Value) {
    let statistics = serde_json::to_value(actual).unwrap();
    let roles = statistics["used_filaments_per_role"].as_object().unwrap();
    let rows = native["roles"].as_array().unwrap();
    assert_eq!(roles.len(), rows.len(), "complete Native role key set");
    for row in rows {
        let role = row["role"].as_str().unwrap();
        let pair = roles[role].as_array().unwrap();
        assert_eq!(pair.len(), 2);
        for (index, value) in pair.iter().enumerate() {
            assert_eq!(
                value.as_f64().unwrap().to_bits(),
                row["bits"][index].as_u64().unwrap(),
                "{role}: {index} (meters, grams)"
            );
        }
    }
}

fn footer_proof(capture: &Path) -> (Vec<u8>, Value) {
    let native = record(capture);
    let gcode = std::fs::read(capture.join("plate_1.gcode")).unwrap();
    assert_eq!(native["input_sha256"], cache_identity::raw_digest(PROJECT));
    assert_eq!(native["gcode_sha256"], cache_identity::raw_digest(&gcode));
    assert_eq!(native["footer_check"]["passed"], true);
    let rows = native["roles"].as_array().unwrap();
    assert_eq!(rows.len(), 8);
    let mut meters = 0.0;
    let mut grams = 0.0;
    for row in rows {
        meters += f64::from_bits(row["bits"][0].as_u64().unwrap());
        grams += f64::from_bits(row["bits"][1].as_u64().unwrap());
    }
    for (unit, expected) in [
        ("mm", format!("{:.2}", meters * 1000.0)),
        ("g", format!("{grams:.2}")),
    ] {
        let prefix = format!("; filament used [{unit}] = ");
        let values = std::str::from_utf8(&gcode)
            .unwrap()
            .lines()
            .filter_map(|line| line.strip_prefix(&prefix))
            .collect::<Vec<_>>();
        assert_eq!(values, [expected]);
    }
    (gcode, native)
}

#[test]
fn native_role_map_is_bound_to_same_run_footer() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let path = runner.work.join("input.3mf");
    std::fs::write(&path, PROJECT).unwrap();
    let Some(capture) = runner.role_slice_reference(&path).unwrap() else {
        return;
    };
    let (gcode, native) = footer_proof(&capture);
    let text = std::str::from_utf8(&gcode).unwrap();
    let actual = ares_core::gcode_processor::process_gcode(text, &settings(text), false, false);
    compare_roles(&actual.result.print_statistics, &native);
}

#[tokio::test]
async fn processed_role_filament_usage_matches_native_map() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let path = runner.work.join("input.3mf");
    std::fs::write(&path, PROJECT).unwrap();
    let Some(capture) = runner.role_slice_reference(&path).unwrap() else {
        return;
    };
    let (gcode, native) = footer_proof(&capture);
    let output = crate::runner::application::slice_project_plate_processed(
        PROJECT.to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    std::fs::write(runner.work.join("ares.gcode"), &output.gcode).unwrap();
    assert!(
        generator_only(&gcode, "OrcaSlicer") == generator_only(&output.gcode, "Ares"),
        "whole slicing output differs; artifacts {:?}",
        runner.work
    );
    compare_roles(&output.result.print_statistics, &native);
    let text = std::str::from_utf8(&gcode).unwrap();
    let processed = ares_core::gcode_processor::process_gcode(text, &settings(text), false, false);
    compare_roles(&processed.result.print_statistics, &native);
}

#[test]
fn role_cache_boundaries_match_native_processor() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let source = slice(&runner, PROJECT);
    let text = std::str::from_utf8(&source).unwrap();
    let header = text
        .split_inclusive('\n')
        .find(|line| line.starts_with("; generated by "))
        .unwrap();
    let (_, config) = text.split_once("; CONFIG_BLOCK_START").unwrap();
    // Handwritten command recipe; the producer/config bytes come from this run.
    let body = "G90\nM83\nT0\n;TYPE:Outer wall\nG1 X10 Y10 Z0.2 F1200\nG1 X20 E2\nT1\nG1 X30 E3\n;TYPE:Outer wall\nG1 X40 E4\n;TYPE:Support\nG1 X50 E2\n;TYPE:Support interface\nG1 X60 E1\n;TYPE:Support transition\nG1 X70 E1\n;TYPE:Prime tower\nG1 X80 E1\n; WIPE_TOWER_END\n;TYPE:Outer wall\nG1 X90 E1\n;COLOR_CHANGE,T1,#00FF00\nM600\nG1 X100 E1\n;PAUSE_PRINT\nM400\nG1 E-1\nG1 E1\nG1 X110\n;WIPE_START\nG1 X120 E1\n;WIPE_END\n; FLUSH_START\nG1 E5\n; FLUSH_END\n; VFLUSH_START\nVG1 E5\n; VFLUSH_END\n;TYPE:Inner wall\nG1 X130 E1\n";
    let mut input = format!("{header}{body}; CONFIG_BLOCK_START{config}");
    for (key, value) in [
        ("filament_diameter", "1.75,2.85"),
        ("filament_density", "1.24,1.5"),
        ("filament_map", "1,1"),
        ("filament_cost", "20,20"),
    ] {
        let prefix = format!("; {key} = ");
        input = input
            .split_inclusive('\n')
            .map(|line| {
                if line.starts_with(&prefix) {
                    format!("{prefix}{value}\n")
                } else {
                    line.to_owned()
                }
            })
            .collect();
    }
    let path = runner.work.join("boundaries.gcode");
    std::fs::write(&path, &input).unwrap();
    let Some(capture) = runner.processor_reference(&path).unwrap() else {
        return;
    };
    let native = record(&capture);
    assert_eq!(
        native["input_sha256"],
        cache_identity::raw_digest(input.as_bytes())
    );
    assert!(native["footer_check"].is_null());
    assert_eq!(native["roles"].as_array().unwrap().len(), 6);
    let mut config = settings(&input);
    config.project.gcode.filament_map.0 = vec![ares_core::OrcaInt(1), ares_core::OrcaInt(1)];
    let actual = ares_core::gcode_processor::process_gcode(&input, &config, false, false);
    compare_roles(&actual.result.print_statistics, &native);
}

fn arc_extrusion_delta(flavor: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = runner.work.join("input.3mf");
    std::fs::write(&project, PROJECT).unwrap();
    let Some(capture) = runner.role_slice_reference(&project).unwrap() else {
        return;
    };
    let (source, _) = footer_proof(&capture);
    let source = std::str::from_utf8(&source).unwrap();
    let header = source
        .split_inclusive('\n')
        .find(|line| line.starts_with("; generated by "))
        .unwrap();
    let (_, config) = source.split_once("; CONFIG_BLOCK_START").unwrap();
    let config = config
        .split_inclusive('\n')
        .map(|line| {
            if line.starts_with("; gcode_flavor = ") {
                format!("; gcode_flavor = {flavor}\n")
            } else {
                line.to_owned()
            }
        })
        .collect::<String>();
    // Fractional arc E exercises the upstream float store; another role keeps G1 independent.
    let body = "G90\nM83\nT0\n;TYPE:Outer wall\nG1 X130.732 Y124.268 Z0.2 F1200\nG1 X124.612 Y124.268 E.73221\n;TYPE:Sparse infill\nG3 X126.388 Y125.631 I-1.798 J4.18 E.27057\nG1 X128.612 Y129.369 E.52045\nG2 X130.388 Y130.732 I3.574 J-2.817 E.27057\n;TYPE:Outer wall\nG1 X124.268 Y130.732 E.73221\n";
    let input = format!("{header}{body}; CONFIG_BLOCK_START{config}");
    let path = runner.work.join("arc-extrusion.gcode");
    std::fs::write(&path, &input).unwrap();
    let Some(capture) = runner.processor_reference(&path).unwrap() else {
        return;
    };
    let native = record(&capture);
    assert_eq!(
        native["input_sha256"],
        cache_identity::raw_digest(input.as_bytes())
    );
    assert_eq!(native["roles"].as_array().unwrap().len(), 2);
    let actual = ares_core::gcode_processor::process_gcode(&input, &settings(&input), false, false);
    let native_moves = std::fs::read_to_string(capture.join("moves.csv")).unwrap();
    let native_moves = native_moves
        .lines()
        .skip(1)
        .map(|line| line.split(',').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    for (index, _) in input
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with("G2 ") || line.starts_with("G3 "))
    {
        let id = index as u32 + 1;
        let native_deltas = native_moves
            .iter()
            .filter(|row| row[0].parse::<u32>().unwrap() == id && row[1] == "10" && row[18] == "0")
            .map(|row| row[8].parse::<f32>().unwrap().to_bits())
            .collect::<Vec<_>>();
        let ares_deltas = actual
            .result
            .moves
            .iter()
            .filter(|vertex| {
                vertex.gcode_id == id
                    && vertex.move_type == ares_core::gcode_processor::EMoveType::Extrude
                    && !vertex.internal_only
            })
            .map(|vertex| vertex.delta_extruder.to_bits())
            .collect::<Vec<_>>();
        std::fs::write(
            runner.work.join(format!("arc-{id}-deltas.json")),
            serde_json::to_vec_pretty(&serde_json::json!({
                "gcode_id": id, "native": &native_deltas, "ares": &ares_deltas,
            }))
            .unwrap(),
        )
        .unwrap();
        assert_ne!(
            native_deltas.first(),
            native_deltas.last(),
            "Native endpoint residual"
        );
        assert_eq!(
            ares_deltas, native_deltas,
            "all arc E deltas, including the final residual"
        );
    }
    std::fs::write(
        runner.work.join("ares-statistics.json"),
        serde_json::to_vec_pretty(&actual.result.print_statistics).unwrap(),
    )
    .unwrap();
    compare_roles(&actual.result.print_statistics, &native);
}

#[test]
fn klipper_arc_extrusion_delta_float_storage_matches_native() {
    arc_extrusion_delta("klipper");
}

#[test]
fn marlin_arc_extrusion_delta_float_storage_matches_native() {
    arc_extrusion_delta("marlin2");
}
