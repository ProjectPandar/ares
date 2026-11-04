//! Printer smoke sweep across OrcaSlicer vendor machine presets.

use std::path::PathBuf;

use crate::{
    self as parity,
    presets::VendorProfiles,
    runner::{self, CaseInputs, OrcaRunner},
};

#[path = "smoke/nozzle.rs"]
mod nozzle;

#[path = "smoke/snapmaker_artisan_detour.rs"]
mod snapmaker_artisan_detour;

#[path = "smoke/layer_fan_priority.rs"]
mod layer_fan_priority;

#[path = "smoke/custom_gcode_functions.rs"]
mod custom_gcode_functions;
#[path = "smoke/custom_gcode_interpolate_table.rs"]
mod custom_gcode_interpolate_table;
#[path = "smoke/custom_gcode_pause_vars.rs"]
mod custom_gcode_pause_vars;
#[path = "smoke/custom_gcode_timelapse.rs"]
mod custom_gcode_timelapse;
#[path = "smoke/custom_gcode_unknown_function.rs"]
mod custom_gcode_unknown_function;
#[path = "smoke/custom_gcode_vector_scalar.rs"]
mod custom_gcode_vector_scalar;

/// Focused re-check of the nearest-seam / layer-1 deposition cluster
/// after seam-placement changes (env-gated, prints outcomes).
#[test]
fn nearest_cluster_check() {
    if !crate::runner::reference_enabled("ARES_PARITY_CLUSTER") {
        eprintln!("skipping: set ARES_PARITY_CLUSTER=1");
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("explicit cluster requires an Orca executable");
    // `ARES_RACE_TARGET="vendor|printer"` drives a single case dump for
    // the fresh-oracle race-batch script; the default runs the cluster pair.
    let targets: Vec<(String, String)> = std::env::var("ARES_RACE_TARGET")
        .ok()
        .and_then(|spec| {
            let (vendor, printer) = spec.split_once('|')?;
            Some(vec![(vendor.to_owned(), printer.to_owned())])
        })
        .unwrap_or_else(|| {
            vec![
                (
                    "Anycubic".to_owned(),
                    "Anycubic Kobra 2 Neo 0.4 nozzle".to_owned(),
                ),
                (
                    "Anycubic".to_owned(),
                    "Anycubic Kobra 3 0.4 nozzle".to_owned(),
                ),
            ]
        });
    let root = profiles_root();
    let model = cube_model();
    for (vendor, printer) in &targets {
        let Ok(profiles) = VendorProfiles::load(&root, vendor) else {
            eprintln!("SKIP {vendor}/{printer}: load failed");
            continue;
        };
        let outcome = match parity::select_printer(&profiles, vendor, printer) {
            Ok(selection) => {
                if std::env::var("CLUSTER_DUMP").is_ok() {
                    let case = parity::build_selection_case(&runner, &profiles, &selection, &model)
                        .map(dump_cluster_case)
                        .map_err(|error| error.to_string());
                    eprintln!("DUMP: {case:?}");
                    assert!(case.is_ok());
                    continue;
                }
                match parity::build_selection_case(&runner, &profiles, &selection, &model) {
                    Ok(case) => parity::compare_case(&case),
                    Err(error) => parity::ares_error(&format!("{vendor}/{printer}"), error),
                }
            }
            Err(error) => parity::ares_error(&format!("{vendor}/{printer}"), error),
        };
        eprintln!(
            "CLUSTER {} {} | {}",
            outcome.status, outcome.label, outcome.detail
        );
    }
}

fn dump_cluster_case(case: parity::ParityCase) {
    let dir = std::path::Path::new("/tmp/kobra2");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("case.3mf"), &case.project).unwrap();
    std::fs::write(dir.join("ref.gcode"), &case.reference).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("tokio runtime");
    match runtime.block_on(async {
        crate::runner::application::slice_project(
            &case.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        )
        .await
    }) {
        Ok(gcode) => {
            let _ = std::fs::write(dir.join("inproc.gcode"), &gcode);
            eprintln!("INPROC {} bytes", gcode.len());
        }
        Err(error) => eprintln!("INPROC failed: {error}"),
    }
}

fn profiles_root() -> PathBuf {
    runner::repo_root().join("OrcaSlicer/resources/profiles")
}

fn cube_model() -> PathBuf {
    runner::repo_root().join("tests/parity/cube10.stl")
}

/// Single-printer smoke case used to develop the harness and record the
/// first divergences on a plain Marlin i3 profile.
#[test]
fn lh_stinger_smoke() {
    assert_printer_smoke("LH", "LH Stinger 0.4 nozzle");
}

#[test]
fn kobra_s1_max_025_smoke() {
    assert_printer_smoke("Anycubic", "Anycubic Kobra S1 Max 0.25 nozzle");
}

#[test]
fn ender3_smoke() {
    assert_printer_smoke("Creality", "Creality Ender-3 0.4 nozzle");
}

#[test]
fn fuzzy_skin_enum_smoke() {
    for value in [
        "none",
        "external",
        "hole",
        "all",
        "allwalls",
        "disabled_fuzzy",
    ] {
        assert_process_option_smoke("fuzzy_skin", value);
    }
}

#[test]
fn internal_octagram_smoke() {
    assert_process_option_smoke("internal_solid_infill_pattern", "octagramspiral");
}

#[test]
fn bottom_hilbert_smoke() {
    assert_process_option_smoke("bottom_surface_pattern", "hilbertcurve");
}

#[test]
fn extra_bridge_layer_smoke() {
    for value in [
        "apply_to_all",
        "disabled",
        "external_bridge_only",
        "internal_bridge_only",
    ] {
        assert_process_option_smoke("enable_extra_bridge_layer", value);
    }
}

#[test]
fn gap_fill_target_smoke() {
    for value in ["everywhere", "nowhere", "topbottom"] {
        assert_process_option_smoke("gap_fill_target", value);
    }
}

#[test]
fn top_concentric_smoke() {
    assert_process_option_smoke("top_surface_pattern", "concentric");
}

#[test]
fn top_rectilinear_smoke() {
    assert_process_option_smoke("top_surface_pattern", "rectilinear");
}

#[test]
fn reduce_crossing_wall_smoke() {
    for value in ["0", "1"] {
        assert_process_option_smoke("reduce_crossing_wall", value);
    }
}

#[test]
fn random_seam_smoke() {
    assert_process_option_smoke("seam_position", "random");
}

#[test]
fn staggered_inner_seams_smoke() {
    for value in ["0", "1"] {
        assert_process_option_smoke("staggered_inner_seams", value);
    }
}

#[test]
fn seam_slope_type_smoke() {
    for value in ["none", "external", "all"] {
        assert_process_option_smoke("seam_slope_type", value);
    }
}

#[test]
fn skirt_loops_smoke() {
    for value in ["0", "4", "10"] {
        assert_process_option_smoke("skirt_loops", value);
    }
}

#[test]
fn max_raft_wipe_smoke() {
    assert_process_option_smoke("raft_layers", "100");
}

#[test]
fn sovol_sv06_high_speed_smoke() {
    assert_printer_smoke("Sovol", "Sovol SV06 0.4 High-Speed nozzle");
}

#[test]
fn kobra3_02_wipe_smoke() {
    assert_printer_smoke("Anycubic", "Anycubic Kobra 3 0.2 nozzle");
}

#[test]
fn rolohaun_delta_flyer_smoke() {
    assert_printer_smoke("RolohaunDesign", "Rolohaun Delta Flyer Refit 0.4 nozzle");
}

#[test]
fn wemake_tinybot_smoke() {
    assert_printer_smoke("WEMAKE3D", "WEMAKE3D TinyBotV1 0.4mm nozzle");
}

#[test]
fn zaa_enabled_smoke() {
    for value in ["0", "1"] {
        assert_process_option_smoke("zaa_enabled", value);
    }
}

#[test]
fn spiral_mode_smoke() {
    for value in ["0", "1"] {
        assert_process_option_smoke("spiral_mode", value);
    }
}

#[test]
fn ironing_solid_smoke() {
    assert_process_option_smoke("ironing_type", "solid");
}

#[test]
fn afinia_hs_06_smoke() {
    assert_printer_smoke("Afinia", "Afinia H+1(HS) 0.6 nozzle");
}

#[test]
fn kobra_max_smoke() {
    assert_printer_smoke("Anycubic", "Anycubic Kobra Max 0.4 nozzle");
}

#[test]
fn artillery_x3_pro_smoke() {
    assert_printer_smoke("Artillery", "Artillery Sidewinder X3 Pro 0.4 nozzle");
}

#[test]
fn prusa_core_one_dump() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        eprintln!("skipping: no OrcaSlicer CLI available");
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Prusa").unwrap();
    let selection =
        parity::select_printer(&profiles, "Prusa", "Prusa CORE One 0.4 nozzle").unwrap();
    let result = parity::build_selection_case(&runner, &profiles, &selection, &cube_model())
        .map(|case| {
            let dir = std::path::Path::new("/tmp/prusa");
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join("case.3mf"), &case.project).unwrap();
            std::fs::write(dir.join("ref.gcode"), &case.reference).unwrap();
        })
        .map_err(|error| error.to_string());
    eprintln!("RESULT: {result:?}");
    assert!(result.is_ok());
}

#[test]
fn ratrig_vcast_smoke() {
    assert_printer_smoke("Ratrig", "RatRig V-Cast 0.4 nozzle");
}

fn assert_printer_smoke(vendor: &str, printer: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        eprintln!("skipping: no OrcaSlicer CLI available");
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), vendor).unwrap();
    let selection = parity::select_printer(&profiles, vendor, printer).unwrap();
    let exported =
        parity::export_selection_case(&runner, &profiles, &selection, &cube_model()).unwrap();
    let outcome = parity::compare_exported(&runner, &exported);
    eprintln!(
        "printer smoke: {} {} {}",
        outcome.status, outcome.label, outcome.detail
    );
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn assert_process_option_smoke(key: &str, value: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert(key.to_owned(), serde_json::Value::String(value.to_owned()));
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let mut overrides = parity::smoke_overrides();
    overrides.remove(key);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &format!("option/{key}/{value}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &overrides,
            &cube_model(),
        )
        .unwrap();
    let outcome = parity::compare_exported(&runner, &exported);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[path = "smoke/sweep.rs"]
mod sweep;

/// Full vendor sweep; writes `tests/parity/printer-smoke-summary.md` and
/// fails when any printer diverges, so each fix moves the summary to green.
/// Gated behind `ARES_PARITY_SWEEP=1` because slicing every vendor preset
/// through both slicers takes hours.
#[test]
fn printer_sweep() {
    sweep::run_printer_sweep();
}
