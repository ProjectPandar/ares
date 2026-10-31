//! GCode.cpp:5808-5813 loop-only preselection; PerimeterGenerator.cpp:230-232,
//! 528-566 bare thin-wall / closed-loop / open-multipath production.

#[path = "small_loop_speed/model.rs"]
mod model;
#[path = "small_loop_speed/transport.rs"]
mod transport;

use super::{CaseInputs, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::{Value, json};
use std::fs;

fn check(
    name: &str,
    shape: model::Shape,
    generator: &str,
    small_perimeter: (&str, &str),
    skirt: bool,
) {
    if !crate::runner::reference_enabled("ARES_PARITY_SMALL_LOOP_SPEED_REFERENCE") {
        return;
    }
    let (threshold, speed) = small_perimeter;
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("small-loop-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let model_path = directory.join("boundary.stl");
    fs::write(&model_path, model::stl(shape)).unwrap();
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let overhang = matches!(shape, model::Shape::OverhangLoft);
    let scarf = name == "true-loop-scarf-ramp";
    let scarf_settings = [
        ("seam_slope_type", "external"),
        ("seam_slope_conditional", "0"),
        ("seam_slope_min_length", "10"),
        ("seam_slope_steps", "4"),
        ("scarf_joint_speed", "100%"),
        ("slow_down_layers", "5"),
        ("initial_layer_speed", "20"),
    ];
    let settings = [
        ("wall_generator", generator),
        (
            "detect_thin_wall",
            if matches!(shape, model::Shape::ThinWall) {
                "1"
            } else {
                "0"
            },
        ),
        ("wall_loops", "1"),
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        (
            "outer_wall_line_width",
            if overhang { "0.42" } else { "0.4" },
        ),
        ("outer_wall_speed", if overhang { "200" } else { "120" }),
        ("inner_wall_speed", "200"),
        ("small_perimeter_threshold", threshold),
        ("small_perimeter_speed", speed),
        ("sparse_infill_density", "0%"),
        ("top_shell_layers", "0"),
        ("bottom_shell_layers", "0"),
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("disable_m73", "0"),
        ("enable_overhang_speed", if overhang { "1" } else { "0" }),
        ("overhang_1_4_speed", "50"),
        ("overhang_2_4_speed", "30"),
        ("overhang_3_4_speed", "20"),
        ("overhang_4_4_speed", "15"),
        ("slowdown_for_curled_perimeters", "0"),
        ("enable_arc_fitting", "0"),
        ("resonance_avoidance", "0"),
        ("slow_down_layers", if scarf { "5" } else { "0" }),
        ("seam_gap", "0"),
        ("seam_position", "nearest"),
        ("brim_type", "no_brim"),
        ("skirt_loops", if skirt { "1" } else { "0" }),
        ("skirt_height", if skirt { "3" } else { "1" }),
        ("skirt_distance", "2"),
        ("support_speed", "80"),
        ("skirt_speed", "0"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), Value::String(value.into()));
    }
    if scarf {
        for (key, value) in scarf_settings {
            process.insert(key.into(), Value::String(value.into()));
        }
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    for filament in &mut filaments {
        filament.insert("slow_down_for_layer_cooling".into(), json!(["0"]));
        filament.insert("filament_max_volumetric_speed".into(), json!(["0"]));
    }
    let label = format!("small-loop-speed/{name}");
    fs::write(
        directory.join("request.json"),
        serde_json::to_vec_pretty(&json!({
            "label":label,"upstream":"345efec64c7b95a05ce5cf96df0dde66a158835c",
            "generator":generator,"threshold":threshold,"speed":speed,"explicit_skirt_speed":skirt,
            "scope":"real byte-input slicing E2E; no test-only production API",
            "model_sha256":super::cache_identity::raw_digest(&fs::read(&model_path).unwrap())
        }))
        .unwrap(),
    )
    .unwrap();
    let mut overrides = parity::smoke_overrides();
    overrides.remove("wall_generator");
    overrides.remove("detect_thin_wall");
    let exported = runner.export_case(
        &CaseInputs {
            label: &label,
            machine: &machine,
            process: &process,
            filaments: &filaments,
        },
        &overrides,
        &model_path,
    );
    let exported = match exported {
        Ok(value) => value,
        Err(error) => {
            fs::write(directory.join("export-error.txt"), error.to_string()).unwrap();
            panic!("{label}: {error}; {directory:?}");
        }
    };
    let exported = transport::threshold_input(exported, threshold, &directory);
    for (key, value) in settings {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    if scarf {
        for (key, value) in scarf_settings {
            application::verify_value(&exported.project, key, &json!(value)).unwrap();
        }
    }
    fs::write(directory.join("input.3mf"), &exported.project).unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    fs::write(directory.join("native.gcode"), &sliced.reference).unwrap();
    let outcome = parity::compare_case(&sliced);
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    let observed = feedrates(native, if skirt { "Skirt" } else { "Outer wall" });
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "label":label,"status":outcome.status,"detail":outcome.detail,"feedrates":observed,
        "input_sha256":super::cache_identity::raw_digest(&sliced.project),
        "native_sha256":super::cache_identity::raw_digest(&sliced.reference),
        "comparison":"complete bytes, only generated-by identity normalization; M73/footer retained"
    })).unwrap()).unwrap();
    let expected = match name {
        "true-loop-percent" | "threshold-equal" | "threshold-above" => 2160.0,
        "explicit-skirt" => 4800.0,
        _ => 7200.0,
    };
    if overhang {
        assert!(
            observed.contains(&12000.0),
            "Native long-loop role speed absent: {label}; {directory:?}"
        );
    } else if scarf {
        assert_eq!(
            observed,
            vec![1392.0, 1584.0],
            "Native scarf early-layer ramp witness: {directory:?}"
        );
    } else {
        assert_eq!(
            observed,
            vec![expected],
            "Native source-owned feedrate witness: {label}; {directory:?}"
        );
    }
    assert!(native.contains("M73 P"), "M73 witness absent");
    assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
}

fn feedrates(gcode: &str, feature: &str) -> Vec<f64> {
    let mut role = "";
    let mut layer = 0;
    let mut current = 0.0;
    let mut result = Vec::new();
    for line in gcode.lines() {
        if line == ";LAYER_CHANGE" {
            layer += 1;
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        if line.starts_with("G1 ")
            && let Some(value) = line
                .split_whitespace()
                .find_map(|word| word.strip_prefix('F'))
        {
            current = value.parse().unwrap();
        }
        if line.starts_with("G1 ")
            && layer > 1
            && role == feature
            && line
                .split_whitespace()
                .any(|w| w.starts_with('X') || w.starts_with('Y'))
            && line.split_whitespace().any(|w| {
                w.strip_prefix('E')
                    .and_then(|v| v.parse::<f64>().ok())
                    .is_some_and(|e| e > 0.0)
            })
        {
            result.push(current);
        }
    }
    result.sort_by(f64::total_cmp);
    result.dedup();
    result
}

#[test]
fn small_loop_bare_path() {
    check(
        "bare-path",
        model::Shape::ThinWall,
        "classic",
        ("5", "30%"),
        false,
    );
}
#[test]
fn small_loop_open_multipath() {
    check(
        "open-multipath",
        model::Shape::ThinWall,
        "arachne",
        ("5", "30%"),
        false,
    );
}
#[test]
fn small_loop_true_loop_percent() {
    check(
        "true-loop-percent",
        model::Shape::Square,
        "classic",
        ("5", "30%"),
        false,
    );
}
#[test]
fn small_loop_long_loop_overhang_fragments() {
    check(
        "long-loop-overhang-fragments",
        model::Shape::OverhangLoft,
        "classic",
        ("5", "30%"),
        false,
    );
}
#[test]
fn small_loop_threshold_below() {
    check(
        "threshold-below",
        model::Shape::Square,
        "classic",
        ("4.8383102699936167", "30%"),
        false,
    );
}
#[test]
fn small_loop_threshold_equal() {
    check(
        "threshold-equal",
        model::Shape::Square,
        "classic",
        ("4.8383102699936176", "30%"),
        false,
    );
}
#[test]
fn small_loop_threshold_above() {
    check(
        "threshold-above",
        model::Shape::Square,
        "classic",
        ("4.8383102699936185", "30%"),
        false,
    );
}
#[test]
fn small_loop_explicit_skirt() {
    check(
        "explicit-skirt",
        model::Shape::Square,
        "classic",
        ("100", "30%"),
        true,
    );
}

#[test]
fn small_loop_true_loop_scarf_ramp() {
    check(
        "true-loop-scarf-ramp",
        model::Shape::Square,
        "classic",
        ("5", "30%"),
        false,
    );
}
