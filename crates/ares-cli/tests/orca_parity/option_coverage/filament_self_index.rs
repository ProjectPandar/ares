//! Public failure inventory: import overwrites IDs, variant payload too short,
//! ignored slot selection, inactive palette, temperature/control-only echo,
//! Native refusal, validated API/CLI byte disparity, incomplete classification.
use super::*;
use crate::{golden, runner::ParityCase};
use serde_json::json;
use std::{collections::BTreeSet, fs, path::Path, process::Command};

#[path = "filament_self_index_input.rs"]
mod input;

const CASES: [(&str, &[&str], i32); 4] = [
    ("F12", &["1", "2"], 211),
    ("F21", &["2", "1"], 233),
    ("F1122", &["1", "1", "2", "2"], 211),
    ("F2211", &["2", "2", "1", "1"], 233),
];

fn commands(bytes: &[u8]) -> Vec<&str> {
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

fn temperatures(bytes: &[u8]) -> Vec<i32> {
    commands(bytes)
        .into_iter()
        .filter(|line| line.starts_with("M104 ") || line.starts_with("M109 "))
        .flat_map(|line| {
            line.split_ascii_whitespace()
                .filter_map(|word| word.strip_prefix('S')?.parse::<i32>().ok())
        })
        .filter(|value| matches!(value, 211 | 233))
        .collect()
}

fn used_filaments(bytes: &[u8]) -> BTreeSet<usize> {
    let mut active = None;
    let mut role = "";
    let mut result = BTreeSet::new();
    for line in commands(bytes) {
        if let Some(marker) = line.strip_prefix("; G4_ACTIVE F") {
            active = Some(marker.split_once(" N").unwrap().0.parse::<usize>().unwrap());
        }
        if let Some(value) = line.strip_prefix("; FEATURE: ") {
            role = value;
        }
        if !line.starts_with("G1 ")
            || !matches!(role, "Outer wall" | "Inner wall" | "Sparse infill")
        {
            continue;
        }
        let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
        let xy = fields
            .iter()
            .any(|word| word.starts_with('X') || word.starts_with('Y'));
        let e = fields
            .iter()
            .filter_map(|word| word.strip_prefix('E'))
            .any(|value| value.parse::<f64>().unwrap() > 0.0);
        if xy && e {
            result.insert(active.unwrap());
        }
    }
    result
}

fn capture_native(
    runner: &OrcaRunner,
    label: &str,
    ids: &[&str],
    first: i32,
    root: &Path,
) -> ParityCase {
    let project = input::project(runner, &format!("filament-self-index/{label}"), ids);
    let case = runner.slice_case(&project).unwrap();
    let directory = root.join(label);
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("input.3mf"), &case.project).unwrap();
    fs::write(directory.join("native.gcode"), &case.reference).unwrap();
    let text = std::str::from_utf8(&case.reference).unwrap();
    assert!(
        text.lines()
            .any(|line| line == format!("; filament_self_index = {}", ids.join(","))),
        "{directory:?}: Native rewrote self-index"
    );
    let actual = temperatures(&case.reference);
    fs::write(
        directory.join("native-consumption.json"),
        serde_json::to_vec_pretty(&json!({
            "label":label,"requested_ids":ids,"first_expected_temperature":first,
            "actual_temperatures":actual,"used_filaments":used_filaments(&case.reference)
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        used_filaments(&case.reference),
        BTreeSet::from([0, 1]),
        "{directory:?}: configured is not used"
    );
    assert!(
        actual.contains(&211) && actual.contains(&233),
        "{directory:?}: temperatures not consumed"
    );
    assert_eq!(
        actual[0], first,
        "{directory:?}: wrong first materialized slot"
    );
    case
}

fn compare_public(case: &ParityCase, directory: &Path) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let api = runtime.block_on(ares_core::slice_project_plate_validated(
        &case.project,
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
        .map_err(|error| format!("{error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&case.reference, gcode));
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
        .map_err(|error| format!("{error}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&case.reference, gcode));
    fs::write(directory.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "label":case.label,"api_comparison":api_comparison,"cli_comparison":cli_comparison,
        "cli_code":cli.status.code(),"native_lines":std::str::from_utf8(&case.reference).unwrap().lines().count(),
        "api_lines":api.as_ref().ok().map(|v| std::str::from_utf8(v).unwrap().lines().count()),
        "cli_lines":cli_bytes.as_ref().ok().map(|v| std::str::from_utf8(v).unwrap().lines().count())
    })).unwrap()).unwrap();
    assert!(cli.status.success(), "{directory:?}: {cli:?}");
    api_comparison.unwrap_or_else(|error| panic!("{directory:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{directory:?}: {error}"));
}

#[test]
fn orca_parity_filament_self_index_has_active_bounded_native_scope() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("filament-self-index-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let captures = CASES
        .iter()
        .map(|&(label, ids, first)| capture_native(&runner, label, ids, first, &root))
        .collect::<Vec<_>>();
    assert_ne!(
        temperatures(&captures[0].reference),
        temperatures(&captures[1].reference)
    );
    assert_ne!(
        temperatures(&captures[2].reference),
        temperatures(&captures[3].reference)
    );
    for (case, &(label, _, _)) in captures.iter().zip(&CASES) {
        compare_public(case, &root.join(label));
    }
    let plan = domains::load(&crate::runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "filament_self_index")
        .unwrap();
    let outcome = OptionOutcome::executed(&plan, 4, Vec::new(), 0, None);
    fs::write(root.join("classification.json"), serde_json::to_vec_pretty(&json!({
        "status":outcome.status,"detail":outcome.detail,"scope":plan.tool_metadata,
        "cases":plan.cases.iter().map(|case| json!({"label":case.label,"value":case.value})).collect::<Vec<_>>()
    })).unwrap()).unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{root:?}: filament_self_index: {}",
        outcome.detail
    );
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.value.clone())
            .collect::<Vec<_>>(),
        CASES
            .iter()
            .map(|case| Some(json!(case.1)))
            .collect::<Vec<_>>()
    );
}

// PrintApply.cpp:1172; PrintConfig.cpp:9004-9054,9805-9910: complete slot IDs.
pub(super) fn generate(key: &str) -> Option<(Vec<domains::OptionCase>, Value)> {
    (key == "filament_self_index").then(|| {
        let cases = CASES.iter().map(|case| domains::OptionCase {
            label: case.0.into(), value: Some(json!(case.1))
        }).collect();
        (cases, json!({
            "domain":"bounded H2D filament self-index slots",
            "definition":"PrintConfig.cpp:5432-5437",
            "consumer":"PresetBundle.cpp:172-195; PrintApply.cpp:1172; PrintConfig.cpp:9004-9054,9805-9910",
            "input":"filament_self_index_input::project", "mode":"Manual",
            "filament_map":["1","2"], "nozzle_volume_type":["Standard","Standard"],
            "nozzle_diameter":["0.4","0.4"],"wall_filament":1,"sparse_infill_filament":2,
            "consumption":"M104/M109 first211 vs233; real wall/infill bothused",
            "deferred":["malformed or short IDs","generated IDs","active High Flow","arbitrary vectors","exotic stride","production changes"]
        }))
    })
}

pub(super) fn execute_plan(runner: &OrcaRunner, plan: &domains::OptionPlan) -> OptionOutcome {
    let root = tempfile::Builder::new()
        .prefix("filament-self-index-matrix-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    for requested in &plan.cases {
        let &(label, ids, first) = CASES.iter().find(|case| case.0 == requested.label).unwrap();
        assert_eq!(requested.value, Some(json!(ids)));
        let case = capture_native(runner, label, ids, first, &root);
        compare_public(&case, &root.join(label));
    }
    OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, None)
}
