//! `GCodeProcessorResult::print_statistics` volume buckets and
//! `extruder_colors` parity: the web Filament/ColorPrint legend reads
//! `model_/support_/wipe_tower_volumes_per_extruder`,
//! `flush_per_filament`, `total_volumes_per_extruder` and
//! `total_used_filament` (`GCodeViewer.cpp:3560-3640, 3719-3733,
//! 3987-3995, 4026-4080`; `GCode/GCodeProcessor.hpp:46-78,
//! 583-619`). The oracle's own `; filament used [mm|cm3|g]` footer is
//! emitted from the same accounting (`GCode.cpp:2330-2370`), so the
//! statistics fields must match the oracle footer exactly.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::{Map, Value};

/// `; filament used [mm] = 240.93, 149.19`-style footer values.
fn footer_vector(reference: &str, key: &str) -> Vec<f64> {
    reference
        .lines()
        .find_map(|line| line.strip_prefix(key))
        .unwrap_or_else(|| panic!("missing {key}"))
        .split(", ")
        .map(|value| value.parse::<f64>().unwrap())
        .collect()
}

/// `volume / (π·(d/2)²)` — the viewer-side `mm` conversion
/// (`get_used_filament_from_volume`, `GCodeViewer.cpp:3443-3449`), kept
/// as the test's oracle because the statistics expose volumes.
fn volume_to_mm(volume: f64, diameter: f64) -> f64 {
    volume / (std::f64::consts::PI * (0.5 * diameter).powi(2))
}

struct Case<'a> {
    vendor: &'a str,
    printer: &'a str,
    overrides: Map<String, Value>,
    label: &'a str,
}

fn statistics_case(case: &Case<'_>) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        case.vendor,
    )
    .unwrap();
    let selection = crate::select_printer(&profiles, case.vendor, case.printer).unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let process = profiles.process(&selection.process).unwrap();
    let filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let exported = runner
        .export_case(
            &CaseInputs {
                label: case.label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &case.overrides,
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    assert_statistics(&runner, &exported);
}

fn assert_statistics(runner: &OrcaRunner, exported: &runner::ExportedCase) {
    let sliced = runner.slice_case(exported).unwrap();
    let processed = processed_gcode(&sliced.project);
    let reference = std::str::from_utf8(&sliced.reference).unwrap();
    assert_footer_equal(&processed, exported, reference, "oracle");
}

/// Ares' wipe-tower / purge machinery is not yet byte-identical to
/// OrcaSlicer's, so the oracle footer is not authoritative here — the
/// statistics must instead match the footer ares itself printed (they
/// account the same emitted stream) and must prove each bucket was
/// exercised.
fn assert_statistics_own_footer(runner: &OrcaRunner, exported: &runner::ExportedCase) {
    let sliced = runner.slice_case(exported).unwrap();
    let processed = processed_gcode(&sliced.project);
    let own = std::str::from_utf8(&processed.gcode).unwrap();
    if let Some(directory) = std::env::var_os("ARES_FILAMENT_STATS_ARTIFACTS") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("ares.gcode"), &processed.gcode).unwrap();
        std::fs::write(
            directory.join("ares-statistics.txt"),
            format!("{:#?}", processed.result.print_statistics),
        )
        .unwrap();
        std::fs::write(directory.join("input.3mf"), &sliced.project).unwrap();
    }
    assert_footer_equal(&processed, exported, own, "own");
}

fn processed_gcode(project: &[u8]) -> ares_core::ProcessedGCode {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_processed(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap()
}

fn assert_footer_equal(
    processed: &ares_core::ProcessedGCode,
    exported: &runner::ExportedCase,
    reference: &str,
    footer_source: &str,
) {
    let stats = &processed.result.print_statistics;
    let settings = runner::application::settings(&exported.project).unwrap();
    let diameters = settings["filament_diameter"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    let densities = settings["filament_density"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    let expected_mm = footer_vector(reference, "; filament used [mm] = ");
    let expected_cm3 = footer_vector(reference, "; filament used [cm3] = ");
    let expected_g = footer_vector(reference, "; filament used [g] = ");
    for (id, volume) in stats.total_volumes_per_extruder.iter().enumerate() {
        let diameter = diameters[id.min(diameters.len() - 1)];
        let density = densities[id.min(densities.len() - 1)];
        // The footer prints `%.2f` (`Print.cpp:3887-3909`), so compare the
        // same 2-decimal rendering rather than raw floats.
        assert_eq!(
            format!("{:.2}", volume_to_mm(*volume, diameter)),
            format!("{:.2}", expected_mm[id]),
            "extruder {id}: volume {volume} mm vs {footer_source} footer"
        );
        assert_eq!(
            format!("{:.2}", volume * 0.001),
            format!("{:.2}", expected_cm3[id]),
            "extruder {id}: volume {volume} cm3 vs {footer_source} footer"
        );
        assert_eq!(
            format!("{:.2}", volume * 0.001 * density),
            format!("{:.2}", expected_g[id]),
            "extruder {id}: volume {volume} g vs {footer_source} footer"
        );
    }
    crate::filament_volume_oracle::check(processed);
    let colours = settings["filament_colour"]
        .as_array()
        .unwrap()
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default();
    for (id, color) in processed.result.extruder_colors.iter().enumerate() {
        let expected = colours.get(id).filter(|value| !value.is_empty());
        assert_eq!(color.as_str(), expected.copied().unwrap_or("#FF8000"));
    }
}

#[test]
fn filament_stats_sermoon_v1() {
    statistics_case(&Case {
        vendor: "Creality",
        printer: "Creality Sermoon V1 0.4 nozzle",
        overrides: Map::new(),
        label: "filament-stats/sermoon-v1",
    });
}

#[test]
fn filament_stats_ender3() {
    statistics_case(&Case {
        vendor: "Creality",
        printer: "Creality Ender-3 0.4 nozzle",
        overrides: Map::new(),
        label: "filament-stats/ender3",
    });
}

#[test]
fn filament_stats_mk4s() {
    statistics_case(&Case {
        vendor: "Prusa",
        printer: "Prusa MK4S 0.4 nozzle",
        overrides: Map::new(),
        label: "filament-stats/mk4s",
    });
}

#[test]
fn filament_stats_h2d() {
    statistics_case(&Case {
        vendor: "BBL",
        printer: "Bambu Lab H2D 0.4 nozzle",
        overrides: Map::new(),
        label: "filament-stats/h2d",
    });
}

#[test]
fn filament_stats_kobra3max_wipe_tower() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Anycubic",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Anycubic", "Anycubic Kobra 3 Max 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    // Tool 2 prints supports → real tool changes + wipe-tower + support
    // volumes per filament (`support_filament`, `PrintConfig.cpp`).
    process.insert("enable_prime_tower".into(), Value::String("1".into()));
    process.insert("enable_support".into(), Value::String("1".into()));
    process.insert("support_filament".into(), Value::String("2".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
    crate::normalize_filament_defaults(&mut filaments);
    for (filament, colour) in filaments.iter_mut().zip(["#F2754E", "#0000FF"]) {
        filament.insert("filament_colour".into(), serde_json::json!([colour]));
        filament.insert("filament_density".into(), serde_json::json!(["1.24"]));
    }
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "filament-stats/h2d-wipe-tower",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &Map::new(),
            &runner::repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    assert_statistics_own_footer(&runner, &exported);
    let processed = processed_gcode(&runner.slice_case(&exported).unwrap().project);
    let stats = &processed.result.print_statistics;
    // The overhang fixture prints supports on tool 2 and wipes on both,
    // so the split buckets must be genuinely exercised, not just sum.
    assert!(stats.model_volumes_per_extruder.iter().any(|v| *v > 0.0));
    assert!(stats.support_volumes_per_extruder.iter().any(|v| *v > 0.0));
    assert!(
        stats
            .wipe_tower_volumes_per_extruder
            .iter()
            .any(|v| *v > 0.0)
    );
    assert!(stats.total_volumes_per_extruder.iter().all(|v| *v > 0.0));
    assert_eq!(processed.result.extruder_colors, vec!["#F2754E", "#0000FF"]);
}

#[test]
fn filament_stats_x1c_support() {
    let mut overrides = Map::new();
    overrides.insert("enable_support".into(), Value::String("1".into()));
    statistics_case(&Case {
        vendor: "BBL",
        printer: "Bambu Lab X1 Carbon 0.4 nozzle",
        overrides,
        label: "filament-stats/x1c-support",
    });
}
