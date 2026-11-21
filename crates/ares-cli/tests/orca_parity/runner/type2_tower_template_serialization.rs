use super::{
    CaseInputs, OrcaRunner, application, native_api_cli::sha, native_stream_case, repo_root,
};
use serde_json::{Map, Value, json};
use std::{
    fs,
    io::{Cursor, Write},
    path::PathBuf,
};

fn project(tower: bool) -> Vec<u8> {
    let runner = OrcaRunner::from_env().unwrap().expect("Native U1 oracle");
    let profiles = PathBuf::from(
        std::env::var_os("ARES_PARITY_PROFILE_RESOURCES_DIR").expect("pinned profile resources"),
    )
    .join("profiles");
    let vendor = crate::presets::VendorProfiles::load(&profiles, "Snapmaker").unwrap();
    let machine = vendor.machine("Snapmaker U1 (0.4 nozzle)").unwrap();
    let process = vendor
        .process("0.20 Standard @Snapmaker U1 (0.4 nozzle)")
        .unwrap();
    let filament = vendor.filament("Generic PLA @U1 0.4 nozzle").unwrap();
    let cube = repo_root().join("tests/parity/cube10.stl");
    let second = runner.work.join("second-cube10.stl");
    fs::copy(&cube, &second).unwrap();
    let export = runner
        .export_case_multi(
            &CaseInputs {
                label: "u1-type2-tower",
                machine: &machine,
                process: &process,
                filaments: &[filament.clone(), filament],
            },
            &Map::new(),
            &[&cube, &second],
            &[1, 2],
        )
        .unwrap();
    let mut settings = application::settings(&export.project).unwrap();
    assert_eq!(settings["wipe_tower_type"], "type2");
    assert_eq!(settings["nozzle_diameter"].as_array().unwrap().len(), 4);
    for key in ["extruder_type", "nozzle_volume_type"] {
        let first = settings[key].as_array().unwrap()[0].clone();
        settings.insert(key.into(), Value::Array(vec![first; 4]));
    }
    settings.insert("filament_colour".into(), json!(["#0060C0", "#FDBD10"]));
    settings.insert("curr_bed_type".into(), json!("High Temp Plate"));
    assert_eq!(settings["enable_prime_tower"], "1");
    if !tower {
        settings.insert("enable_prime_tower".into(), json!("0"));
    }
    let mut source = zip::ZipArchive::new(Cursor::new(export.project)).unwrap();
    let mut target = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..source.len() {
        let member = source.by_index(i).unwrap();
        if member.name() == "Metadata/project_settings.config" {
            target
                .start_file(member.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            target
                .write_all(&serde_json::to_vec(&settings).unwrap())
                .unwrap();
        } else {
            target.raw_copy_file(member).unwrap();
        }
    }
    target.finish().unwrap().into_inner()
}

fn compare(tower: bool, label: &str) {
    let Some(_runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = project(tower);
    native_stream_case::check(
        (
            "u1-type2-tower",
            &[
                "GCode.cpp:1941-1953,5387-5395",
                "PlaceholderParser.cpp:1064-1067",
            ],
        ),
        label,
        &project,
        &sha(&project),
        (true, |native, _| {
            assert_eq!(native.native.contains("; CP TOOLCHANGE START\n"), tower);
        }),
    );
}

#[test]
fn type2_tower_preserves_the_writer_acceleration_cache() {
    compare(true, "with-tower");
}

#[test]
fn type2_tower_motion_cache_control_without_tower() {
    compare(false, "without-tower");
}
