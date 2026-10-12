//! `*_rotate_template` meta-language admission + interpreter parity
//! (docs/architecture/infill-rotate-template-meta.md).
//!
//! Upstream `calculate_infill_rotation_angle` (`Fill.cpp:52-210`) accepts the
//! full template meta-language; Ares' `simple_rotation_angle` only covers the
//! non-meta list path (`Fill.cpp:206-210`) and rejects everything else.
//! Each case is Native-backed: the release oracle defines the expected angle
//! sequence byte-for-byte.

use crate::{
    CaseInputs, OrcaRunner, VendorProfiles, normalize_filament_defaults,
    normalize_process_defaults, runner, select_printer, smoke_overrides,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

const MESH_SHA256: &str = "a7f79fdf8a5c136095b6ef2384de24dcaa3166c7df4e92768a434c454db55031";

macro_rules! template_case {
    ($name:ident, $channel:literal, $template:literal) => {
        #[test]
        fn $name() {
            check($channel, $template);
        }
    };
}

// Meta-language branch coverage (RED matrix rows T0-T26 of the ARD).
template_case!(t00_sparse_increment, "sparse", "+45");
template_case!(t01_sparse_absolute, "sparse", "45");
template_case!(t02_sparse_percent_turn, "sparse", "+10%");
template_case!(t03_sparse_repeats, "sparse", "+5*3");
template_case!(t04_sparse_one_time, "sparse", "0,45!");
template_case!(t05_sparse_bottom_shell, "sparse", "+30B");
template_case!(t06_sparse_top_shell, "sparse", "+30T");
template_case!(t07_sparse_layers_unit, "sparse", "+45#2");
template_case!(t08_sparse_feet, "sparse", "+45'1");
template_case!(t09_sparse_inches, "sparse", "+45\"1");
template_case!(t10_sparse_centimeters, "sparse", "+45c1");
template_case!(t11_sparse_meters, "sparse", "+45m5");
template_case!(t12_sparse_millimeters, "sparse", "+45mm5");
template_case!(t13_sparse_percent_height, "sparse", "+45%50");
template_case!(t14_sparse_joint_n, "sparse", "+90N4");
template_case!(t15_sparse_joint_z, "sparse", "+90Z4");
template_case!(t16_sparse_joint_arcsin, "sparse", "+90$4");
template_case!(t17_sparse_joint_l, "sparse", "+90L4");
template_case!(t18_sparse_joint_u, "sparse", "+90U4");
template_case!(t19_sparse_joint_q, "sparse", "+90Q4");
template_case!(t20_sparse_joint_middle, "sparse", "+90|4");
template_case!(t21_sparse_joint_end, "sparse", "+90#4");
template_case!(t22_solid_increment, "solid", "+30");
template_case!(t23_solid_list_control, "solid", "20,70");
template_case!(t24_sparse_list_control, "sparse", "0,45,90");
template_case!(t25_sparse_joint_rand_fill, "sparse", "+30~2");
template_case!(t26_sparse_joint_rand_disperse, "sparse", "+30^2");

fn check(channel: &str, template: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix(&format!("rotate-template-{channel}-"))
        .tempdir_in(root)
        .unwrap()
        .keep();
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection = select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    normalize_filament_defaults(&mut filaments);
    let template_key = if channel == "sparse" {
        "sparse_infill_rotate_template"
    } else {
        "solid_infill_rotate_template"
    };
    let process_values = [
        ("sparse_infill_pattern", Value::from("rectilinear")),
        ("sparse_infill_density", Value::from("20%")),
        ("fill_multiline", Value::from("1")),
        (template_key, Value::from(template)),
        ("enable_prime_tower", Value::from("0")),
        ("gcode_comments", Value::from("1")),
    ];
    for (key, value) in &process_values {
        process.insert((*key).into(), value.clone());
    }
    let mesh = runner::repo_root().join("tests/parity/overhang.stl");
    let mesh_bytes = fs::read(&mesh).unwrap();
    assert_eq!(
        Sha256::digest(&mesh_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        MESH_SHA256
    );
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &format!("rotate-template/{channel}/{template}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &smoke_overrides(),
            &mesh,
        )
        .unwrap();
    for (key, value) in &process_values {
        runner::application::verify_value(&exported.project, key, value).unwrap();
    }
    fs::write(directory.join("input.3mf"), &exported.project).unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    fs::write(directory.join("native.gcode"), &sliced.reference).unwrap();
    fs::write(
        directory.join("witness.json"),
        serde_json::to_vec_pretty(&json!({
            "channel": channel,
            "template": template,
            "native_gcode_bytes": sliced.reference.len(),
        }))
        .unwrap(),
    )
    .unwrap();
    let outcome = crate::compare_case(&sliced);
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({"status": outcome.status, "detail": outcome.detail}))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
}
