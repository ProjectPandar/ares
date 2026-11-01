//! `OrcaSlicer.cpp:2816-2817` sizes `inherits_group` to
//! `filament_count + 2` string slots (`[process | filaments | machine]`),
//! written slot-wise for the loaded machine (`:2840-2842`), process
//! (`:2983-2985`), and filaments (`:3163`). A user process keeps its
//! `inherits` ancestor (`OrcaSlicer.cpp:2085-2086`), so slot 0 carries the
//! system ancestor name while the system filament/machine slots stay empty.
//! The composed profile group must reach the overlay as one value per slot
//! (`ConfigOptionStrings` elements); the per-token serialization
//! (`escape_strings_cstyle`, `Config.cpp:72-120`) happens only at export.

use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest};
use serde_json::{Value, json};

use super::{
    OrcaRunner,
    profile_entry::{vendor, write_json},
    stages,
};
use crate::presets::VendorProfiles;

fn config_line<'a>(gcode: &'a str, key: &str) -> Option<&'a str> {
    gcode
        .lines()
        .find(|line| line.starts_with(&format!("; {key} = ")))
}

fn line_token(line: &str) -> &str {
    line.split_once(" = ").map(|(_, token)| token).unwrap_or("")
}

#[tokio::test]
async fn profile_inherits_group_slots_match_native() {
    let root = super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = VendorProfiles::load(&profiles_root, "BBL").unwrap();
    let machine = "Bambu Lab X1 Carbon 0.4 nozzle";
    let system_process = "0.20mm Standard @BBL X1C";
    // A user copy of the system process keeps `inherits`
    // (`OrcaSlicer.cpp:2085-2086` user branch).
    let process = "0.20mm Standard @BBL X1C - Mine";
    let filament = "Bambu PLA Basic @BBL X1C";
    let catalog = json!({
        "vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "BBL")],
        "imports": [{
            "profiles": [{
                "type": "process",
                "name": process,
                "from": "user",
                "inherits": system_process,
                "instantiation": "true",
            }],
        }],
    });
    let selection = json!({"machine": machine, "process": process, "filaments": [filament]});
    let directory = tempfile::Builder::new()
        .prefix("profile-inherits-group-")
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    // The native oracle loads the same user process as a flattened preset:
    // the system chain with the user deltas and the surviving `inherits`.
    let mut process_fields = profiles.process(system_process).unwrap();
    process_fields.insert("name".to_owned(), Value::String(process.to_owned()));
    process_fields.insert("from".to_owned(), Value::String("user".to_owned()));
    process_fields.insert(
        "inherits".to_owned(),
        Value::String(system_process.to_owned()),
    );
    for (kind, fields) in [
        ("machine", profiles.machine(machine).unwrap()),
        ("process", process_fields),
        ("filament", profiles.filament(filament).unwrap()),
    ] {
        write_json(
            &directory.join(format!("{kind}.json")),
            &Value::Object(fields),
        );
    }
    let input = include_bytes!("../../../../../tests/parity/validation/preset-json-ender3-0.2.3mf");
    std::fs::write(directory.join("input.3mf"), input).unwrap();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let request = ProfileSliceRequest {
        filename: "input.3mf",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: Placement::None,
    };
    let ares = ares_core::slice_with_profiles(
        input,
        request,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let ares = String::from_utf8_lossy(&ares).into_owned();
    let ares_line =
        config_line(&ares, "inherits_group").expect("ares output lacks ; inherits_group = line");
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
    let output = directory.join("native");
    std::fs::create_dir(&output).unwrap();
    let (datadir, process_path) =
        super::profile_entry::native_user_presets::stage(&directory, &profiles_root);
    stages::run(
        runner.bin(),
        [
            "--datadir".to_owned(),
            datadir.to_str().unwrap().to_owned(),
            "--load-settings".to_owned(),
            format!("{};{}", path("machine.json"), process_path.display()),
            "--load-filaments".to_owned(),
            path("filament.json"),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            path("input.3mf"),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let native = std::fs::read_to_string(output.join("plate_1.gcode")).unwrap();
    let native_line = config_line(&native, "inherits_group")
        .expect("native output lacks ; inherits_group = line");
    assert_eq!(
        line_token(ares_line),
        line_token(native_line),
        "; inherits_group mismatch"
    );
    // `filament_count + 2` slots: process | one filament | machine
    // (`OrcaSlicer.cpp:2817`).
    assert_eq!(
        line_token(ares_line).matches(';').count(),
        2,
        "inherits_group must carry one slot per process|filament|machine: {ares_line}"
    );
}
