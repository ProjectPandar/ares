use std::fs;

use ares_core::{GenerationMetadata, SlicerContext};
use serde_json::{Map, json};

use crate::runner::{CaseInputs, OrcaRunner, application, processor_migration, repo_root};
use crate::{presets::VendorProfiles, select_printer};

const SUMMARY: &str = "; Final statistics: {print_time}|{normal_print_time}|{silent_print_time}|\
     {total_cost}|{total_wipe_tower_cost}|{total_wipe_tower_filament}";

#[tokio::test]
async fn stock_anker_file_start_filament_statistics_match_native() {
    compare(false).await;
}

#[tokio::test]
async fn deferred_time_cost_and_tower_statistics_match_native() {
    compare(true).await;
}

async fn compare(include_summary: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "Anker").unwrap();
    let selected = select_printer(&profiles, "Anker", "Anker M5 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selected.printer).unwrap();
    let process = profiles.process(&selected.process).unwrap();
    let filaments = selected
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    let stock = machine["file_start_gcode"].as_str().unwrap();
    assert!(stock.contains("{total_weight}") && stock.contains("{used_filament}"));
    let template = if include_summary {
        machine.insert("time_cost".into(), json!("3600"));
        let stock = machine["file_start_gcode"].as_str().unwrap();
        format!("{stock}\n{SUMMARY}")
    } else {
        stock.to_owned()
    };
    if include_summary {
        machine.insert("file_start_gcode".into(), json!(template));
    }
    let exported = runner
        .export_case(
            &CaseInputs {
                label: if include_summary {
                    "anker-file-start-summary"
                } else {
                    "anker-file-start-stock"
                },
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &Map::new(),
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    application::verify_value(&exported.project, "file_start_gcode", &json!(template)).unwrap();
    if include_summary {
        application::verify_value(&exported.project, "time_cost", &json!("3600")).unwrap();
    }
    let case = runner.slice_case(&exported).unwrap();
    let reference = std::str::from_utf8(&case.reference).unwrap();
    for (prefix, unit) in [(";Filament weight: ", 'g'), (";Filament used: ", 'm')] {
        let value = reference
            .lines()
            .find_map(|line| line.strip_prefix(prefix))
            .and_then(|value| value.strip_suffix(unit))
            .unwrap();
        assert!(value.parse::<f64>().unwrap() > 0.0);
    }
    if include_summary {
        let line = reference
            .lines()
            .find_map(|line| line.strip_prefix("; Final statistics: "))
            .unwrap();
        let fields: Vec<_> = line.split('|').collect();
        assert_eq!(fields.len(), 6, "{line}");
        assert_eq!(fields[0], fields[1]);
        assert!(fields[0].ends_with('s') && fields[0] != "0s", "{line}");
        assert_eq!(fields[2], "0s");
        assert!(fields[3].parse::<f64>().unwrap() > 0.0, "{line}");
        assert_eq!(&fields[4..], ["0.00", "0.00"]);
    }
    let api = application::slice_project(
        &case.project,
        GenerationMetadata::deterministic(1980, 1, 1, 0, 0, 0),
        SlicerContext::new(0),
    )
    .await;
    let retained = runner.work.join("ares.gcode");
    let cli = application::cli_command()
        .arg("slice")
        .arg("-o")
        .arg(&retained)
        .arg(&exported.project_path)
        .output()
        .unwrap();
    assert!(
        api.is_ok() && cli.status.success(),
        "API: {:?}; CLI: {}: {}",
        api.as_ref().err(),
        cli.status,
        String::from_utf8_lossy(&cli.stderr),
    );
    let api = api.unwrap();
    assert!(api == fs::read(retained).unwrap(), "API differs from CLI");
    let actual = processor_migration::generator_only(&api, "Ares");
    let expected = processor_migration::generator_only(&case.reference, "OrcaSlicer");
    assert!(
        actual == expected,
        "complete output differs at line {:?}; lengths {}/{}",
        actual
            .split(|byte| *byte == b'\n')
            .zip(expected.split(|byte| *byte == b'\n'))
            .position(|(a, b)| a != b)
            .map(|n| n + 1),
        actual.len(),
        expected.len(),
    );
}
