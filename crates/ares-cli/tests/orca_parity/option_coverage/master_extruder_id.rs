//! Bounded active master selection, not an arbitrary integer-index domain.
//! Public failure inventory: refused/rewritten request, ignored master, config-only
//! activity, wrong actual nozzle, API/CLI divergence, baseline-only accounting.
use super::*;
use crate::golden;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, process::Command};

#[path = "master_extruder_id_input.rs"]
mod input;

fn outside_config(bytes: &[u8]) -> Vec<&str> {
    let mut config = false;
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| match *line {
            "; CONFIG_BLOCK_START" => {
                config = true;
                false
            }
            "; CONFIG_BLOCK_END" => {
                config = false;
                false
            }
            _ => !config,
        })
        .collect()
}

fn used_nozzles(bytes: &[u8]) -> BTreeSet<(usize, usize, String)> {
    let mut active = None;
    let mut role = "";
    let mut result = BTreeSet::new();
    for line in outside_config(bytes) {
        if let Some(marker) = line.strip_prefix("; G4_ACTIVE F") {
            let (filament, nozzle) = marker.split_once(" N").unwrap();
            active = Some((
                filament.parse::<usize>().unwrap(),
                nozzle.parse::<usize>().unwrap(),
            ));
        }
        if let Some(value) = line.strip_prefix("; FEATURE: ") {
            role = value;
        }
        if !line.starts_with("G1 ")
            || !matches!(role, "Outer wall" | "Inner wall" | "Sparse infill")
        {
            continue;
        }
        let words = line.split_ascii_whitespace().collect::<Vec<_>>();
        let xy = words
            .iter()
            .any(|w| w.starts_with('X') || w.starts_with('Y'));
        let extrudes = words
            .iter()
            .filter_map(|w| w.strip_prefix('E'))
            .any(|v| v.parse::<f64>().unwrap() > 0.0);
        if xy && extrudes {
            let (filament, nozzle) = active.unwrap();
            result.insert((filament, nozzle, role.into()));
        }
    }
    result
}

fn compare(runner: &OrcaRunner, label: &str, master: &str, mode: &str) -> Vec<u8> {
    let exported = input::project(runner, label, master, mode);
    let sliced = runner.slice_case(&exported).unwrap();
    let directory = tempfile::Builder::new()
        .prefix("master-extruder-id-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(directory.join("input.3mf"), &sliced.project).unwrap();
    fs::write(directory.join("native.gcode"), &sliced.reference).unwrap();
    let text = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(
        text.lines()
            .any(|line| line == format!("; master_extruder_id = {master}")),
        "{directory:?}: Native rewrote master"
    );
    assert!(
        text.lines()
            .any(|line| line == format!("; filament_map_mode = {mode}")),
        "{directory:?}: Native rewrote mode"
    );
    let expected = if mode == "Manual" || master == "1" {
        [1, 2]
    } else {
        [2, 1]
    };
    let observation = used_nozzles(&sliced.reference);
    for (filament, nozzle, role) in &observation {
        assert_eq!(*nozzle, expected[*filament], "{directory:?}: {role}");
        assert_eq!(
            *filament,
            usize::from(role == "Sparse infill"),
            "{directory:?}: wrong wall/infill role"
        );
    }
    assert_eq!(
        observation.iter().map(|v| v.0).collect::<BTreeSet<_>>(),
        BTreeSet::from([0, 1]),
        "{directory:?}: inactive context"
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let api = runtime.block_on(ares_core::slice_project_plate_validated(
        &sliced.project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    if let Ok(gcode) = &api {
        fs::write(directory.join("ares.validated.gcode"), gcode).unwrap();
    }
    let api_comparison = api
        .as_ref()
        .map_err(|e| format!("{e:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&sliced.reference, gcode));
    let output = directory.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&output)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&output);
    let cli_comparison = cli_bytes
        .as_ref()
        .map_err(|e| format!("{e}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&sliced.reference, gcode));
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label":label,"master":master,"mode":mode,"expected_map":expected,
            "native_observation":observation,"api_comparison":api_comparison,
            "cli_comparison":cli_comparison,"cli_code":cli.status.code(),
            "native_lines":text.lines().count(),
            "api_lines":api.as_ref().ok().map(|v| std::str::from_utf8(v).unwrap().lines().count()),
            "input_sha256":Sha256::digest(&sliced.project).iter().map(|byte| format!("{byte:02x}")).collect::<String>()
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "{directory:?}: {cli:?}");
    api_comparison.unwrap_or_else(|error| panic!("{directory:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{directory:?}: {error}"));
    sliced.reference
}

#[test]
fn orca_parity_master_extruder_id_has_active_bounded_native_scope() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let mut captures = Vec::new();
    for (label, master, mode) in [
        ("M1", "1", "Auto For Flush"),
        ("M2", "2", "Auto For Flush"),
        ("MC1", "1", "Manual"),
        ("MC2", "2", "Manual"),
    ] {
        captures.push(compare(
            &runner,
            &format!("master-extruder-id/{label}"),
            master,
            mode,
        ));
    }
    assert_ne!(used_nozzles(&captures[0]), used_nozzles(&captures[1]));
    assert_eq!(used_nozzles(&captures[2]), used_nozzles(&captures[3]));
    let plan = domains::load(&crate::runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "master_extruder_id")
        .unwrap();
    let outcome = OptionOutcome::executed(&plan, 2, Vec::new(), 0, None);
    fs::write(crate::artifacts::root_from_env().unwrap().join("master-extruder-id-classification.json"),
        serde_json::to_vec_pretty(&json!({"status":outcome.status,"detail":outcome.detail,"cases":plan.cases.iter().map(|case| json!({"label":case.label,"value":case.value})).collect::<Vec<_>>(),"domain":plan.tool_metadata})).unwrap()).unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "master_extruder_id: {}",
        outcome.detail
    );
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.value.clone())
            .collect::<Vec<_>>(),
        [Some(json!("1")), Some(json!("2"))]
    );
}

// PrintConfig.cpp:5399; ToolOrdering.cpp:1150,1200-1201: bounded H2D TPU branch.
pub(super) fn generate(key: &str) -> Option<(Vec<domains::OptionCase>, Value)> {
    (key == "master_extruder_id").then(|| {
        let cases = [("M1", "1"), ("M2", "2")].into_iter()
            .map(|(label, value)| domains::OptionCase { label: label.into(), value: Some(json!(value)) })
            .collect();
        (cases, json!({
            "domain":"bounded H2D TPU master selector",
            "definition":"PrintConfig.cpp:5399-5403",
            "consumer":"ToolOrdering.cpp:1150-1207; FilamentGroup.cpp:40-49",
            "input":"master_extruder_id_input::project",
            "mode":"Auto For Flush", "nozzle_diameter":["0.4","0.4"],
            "filament_type":["TPU","PLA"], "wall_filament":1,"sparse_infill_filament":2,
            "controls":"Manual [1,2] with master1 and master2 retains actual motion",
            "deferred":["Auto For Match","non-TPU optimization","AMS grouping","wipe tower offsets","invalid indices","arbitrary printer contexts"]
        }))
    })
}

pub(super) fn execute_plan(runner: &OrcaRunner, plan: &domains::OptionPlan) -> OptionOutcome {
    for case in &plan.cases {
        compare(
            runner,
            &format!("master-extruder-id/{}", case.label),
            case.value.as_ref().unwrap().as_str().unwrap(),
            "Auto For Flush",
        );
    }
    OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, None)
}
