//! `DoExport::update_print_estimated_stats` (`GCode.cpp:1904-1940`): the
//! `PrintStatistics` totals the preview Summary legend reads
//! (`GCodeViewer.cpp:4003-4024`), against Native `print.print_statistics()`
//! after export for the inputs in `tests/parity/summary/` (see its README),
//! compared bit for bit.
use crate::runner::{self, OrcaRunner};

fn processed(project: &[u8]) -> ares_core::ProcessedGCode {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_processed(
            project,
            ares_core::GenerationMetadata::default(),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap()
}

fn assert_native(fixture: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let input = runner::repo_root().join(format!("tests/parity/summary/{fixture}.3mf"));
    let Some((native, entry)) = runner
        .print_statistics_reference(&input)
        .unwrap_or_else(|error| panic!("{error}"))
    else {
        return;
    };
    let output = processed(&std::fs::read(&input).unwrap());
    let statistics = &output.result.print_statistics;
    let artifacts = tempfile::Builder::new()
        .prefix(&format!("print-statistics-{fixture}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    std::fs::write(
        artifacts.join("statistics.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "fixture": fixture,
            "input_sha256": runner::native_api_cli::sha(&std::fs::read(&input).unwrap()),
            "native_entry": entry,
            "actual_totals": output.print_statistics,
            "actual_normal_time": statistics.normal_time,
            "actual_prepare_time": statistics.normal_prepare_time,
            "native": native,
        }))
        .unwrap(),
    )
    .unwrap();
    eprintln!("{fixture}: statistics artifact {artifacts:?}");
    // The shared sources must already agree before the totals can.
    // Native's std::map holds only extruders that extruded.
    let volumes = statistics
        .total_volumes_per_extruder
        .iter()
        .enumerate()
        .filter(|(_, volume)| **volume != 0.0)
        .map(|(id, volume)| (id.to_string(), volume.to_bits()))
        .collect::<Vec<_>>();
    let native_volumes = native["total_volumes_per_extruder"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(id, volume)| (id.clone(), volume.as_f64().unwrap().to_bits()))
        .collect::<Vec<_>>();
    assert_eq!(
        volumes, native_volumes,
        "{fixture}: total_volumes_per_extruder; {entry:?}"
    );
    // The existing Native probe prints this source with %.9g, so keep its
    // f32 projection; the double totals below are compared without rounding.
    assert_eq!(
        (statistics.normal_time as f32).to_bits(),
        (native["normal_time"].as_f64().unwrap() as f32).to_bits(),
        "{fixture}: modes[Normal].time {}; {entry:?}",
        statistics.normal_time
    );
    // The footer's Prepare and Model printing time rows
    // (`GCodeViewer.cpp:4627-4640`); only Normal mode is ported.
    let native_prepare = native["normal_prepare_time"].as_f64().unwrap() as f32;
    assert!(
        native_prepare > 0.0 && native["stealth_time"].as_f64().unwrap() == 0.0,
        "{fixture}: fixture must have a prepare stage and no stealth machine; {entry:?}"
    );
    assert_eq!(
        statistics.normal_prepare_time.to_bits(),
        native_prepare.to_bits(),
        "{fixture}: modes[Normal].prepare_time {} != native {native_prepare}; {entry:?}",
        statistics.normal_prepare_time
    );
    let actual = output.print_statistics;
    for (field, value) in [
        ("total_extruded_volume", actual.total_extruded_volume),
        ("total_used_filament", actual.total_used_filament),
        ("total_weight", actual.total_weight),
        ("total_cost", actual.total_cost),
    ] {
        let expected = native[field].as_f64().unwrap();
        assert_eq!(
            value.to_bits(),
            expected.to_bits(),
            "{fixture}: {field} {value:?} != native {expected:?}; {entry:?}"
        );
    }
}

/// Two used filaments with distinct densities and costs, `time_cost` 1.7.
#[test]
fn print_estimated_stats_time_cost() {
    assert_native("time-cost");
}

/// ToolOrdering.cpp:1506-1527: missing count entries still create both X2D groups.
#[test]
fn cli_time_cost_missing_nozzle_counts_matches_native() {
    use crate::runner::native_api_cli::{NativeCase, sha};
    use serde_json::json;
    use std::{collections::BTreeMap, fs, io::Cursor};

    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = include_bytes!("../../../../tests/parity/summary/time-cost.3mf");
    let mut archive = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let settings: serde_json::Value =
        serde_json::from_reader(archive.by_name("Metadata/project_settings.config").unwrap())
            .unwrap();
    assert_eq!(settings["nozzle_diameter"], json!(["0.4", "0.4"]));
    assert!(settings.get("extruder_max_nozzle_count").is_none());
    let reference = NativeCase::slice(&runner, project, "nozzle-group-extruder-count-");
    let output_path = reference.root.join("ares.cli.gcode");
    // The shared cli_command injects a preset catalog, hiding the missing count.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&output_path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &output.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &output.stderr).unwrap();
    fs::write(
        reference.root.join("cli-status.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": sha(project), "native_sha256": sha(reference.native.as_bytes()),
            "cli_exit": output.status.code(), "success": output.status.success(),
            "upstream": "40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19",
            "boundary": "ToolOrdering.cpp:1506-1527 build_nozzle_groups",
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "CLI must retain both physical groups, status {}; artifacts {:?}: {}",
        output.status,
        reference.root,
        String::from_utf8_lossy(&output.stderr)
    );
    let cli = fs::read_to_string(&output_path).unwrap();
    let mut comparison = BTreeMap::new();
    for key in [
        "nozzle_diameter",
        "filament_map",
        "filament_map_mode",
        "filament_nozzle_map",
        "filament_volume_map",
    ] {
        let prefix = format!("; {key} = ");
        let native = reference
            .native
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap();
        let actual = cli
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap();
        comparison.insert(key, json!({"native": native, "cli": actual}));
    }
    fs::write(
        reference.root.join("nozzle-group-comparison.json"),
        serde_json::to_vec_pretty(&comparison).unwrap(),
    )
    .unwrap();
    assert!(
        comparison["filament_map"]["native"]
            .as_str()
            .unwrap()
            .split(',')
            .any(|head| head == "2")
    );
    for (key, values) in comparison {
        assert_eq!(
            values["native"], values["cli"],
            "{key}; {:?}",
            reference.root
        );
    }
}

/// Filament 2 is configured but unused (not in `m_writer.extruders()`).
#[test]
fn print_estimated_stats_unused_filament() {
    assert_native("unused-filament");
}
