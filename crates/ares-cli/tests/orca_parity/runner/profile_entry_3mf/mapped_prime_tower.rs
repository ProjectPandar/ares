use std::fs;

use serde_json::{Value, json};

use crate::runner::{
    OrcaRunner, join_paths,
    profile_entry::{placed_cube, vendor, write_json},
    repo_root, stages,
};

struct Case<'a> {
    name: &'a str,
    filament_count: usize,
    overrides: Value,
}

fn run_case(case: Case<'_>) {
    if !crate::runner::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "LH").unwrap();
    let machine_name = "LH Stinger 0.4 nozzle";
    let process_name = profiles.compatible_process(machine_name).unwrap();
    let filament_name = profiles.compatible_filament(machine_name).unwrap();
    let catalog = json!({
        "vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "LH")],
        "imports": [],
    });
    let selection = json!({
        "machine": machine_name,
        "process": process_name,
        "filaments": vec![filament_name.clone(); case.filament_count],
    });
    let stl = placed_cube(&fs::read(root.join("tests/parity/cube10.stl")).unwrap());
    fs::write(runner.work.join("input.stl"), stl).unwrap();
    let mut machine = profiles.machine(machine_name).unwrap();
    machine.insert("type".to_owned(), json!("machine"));
    machine.insert("from".to_owned(), json!("system"));
    machine.insert("instantiation".to_owned(), json!("true"));
    write_json(&runner.work.join("machine.json"), &Value::Object(machine));
    let mut process = profiles.process(&process_name).unwrap();
    process.insert("type".to_owned(), json!("process"));
    process.insert("from".to_owned(), json!("system"));
    process.insert("instantiation".to_owned(), json!("true"));
    process.insert(
        "filament_settings_id".to_owned(),
        json!(vec![filament_name.clone(); case.filament_count]),
    );
    write_json(&runner.work.join("process.json"), &Value::Object(process));
    let filament = profiles.filament(&filament_name).unwrap();
    for index in 0..case.filament_count {
        let mut filament = filament.clone();
        filament.insert("type".to_owned(), json!("filament"));
        filament.insert("from".to_owned(), json!("system"));
        filament.insert("instantiation".to_owned(), json!("true"));
        write_json(
            &runner.work.join(format!("filament-{index}.json")),
            &Value::Object(filament),
        );
    }
    let filament_paths = (0..case.filament_count)
        .map(|index| runner.work.join(format!("filament-{index}.json")))
        .map(|path| path.to_str().unwrap().to_owned())
        .collect::<Vec<_>>()
        .join(";");
    let project_path = runner.work.join(format!("{}.3mf", case.name));
    stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            join_paths(
                [
                    runner.work.join("machine.json"),
                    runner.work.join("process.json"),
                ]
                .iter(),
            ),
            "--load-filaments".to_owned(),
            filament_paths,
            "--arrange".to_owned(),
            "1".to_owned(),
            "--export-3mf".to_owned(),
            project_path.to_str().unwrap().to_owned(),
            runner.work.join("input.stl").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Export,
        &runner.work,
    )
    .unwrap();

    let native_presets = crate::runner::profile_entry::native_user_presets::resolved(
        &runner.work,
        &profiles_root,
        &catalog,
        &selection,
    );
    let overrides_json = serde_json::to_vec(&case.overrides).unwrap();
    let overrides = ares_core::ConfigOverrides::from_json(&overrides_json).unwrap();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let request = || ares_core::ProfileSliceRequest {
        filename: "input.3mf",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: ares_core::Placement::None,
    };
    let output = runner.work.join("native");
    fs::create_dir(&output).unwrap();
    let native_args = case
        .overrides
        .as_object()
        .unwrap()
        .iter()
        .flat_map(|(key, value)| {
            let delimiter = if key == "wrapping_exclude_area" {
                ","
            } else {
                ";"
            };
            let value = value
                .as_str()
                .map(str::to_owned)
                .or_else(|| {
                    value.as_array().map(|values| {
                        values
                            .iter()
                            .map(|item| item.as_str().unwrap())
                            .collect::<Vec<_>>()
                            .join(delimiter)
                    })
                })
                .unwrap();
            [format!("--{}", key.replace('_', "-")), value]
        });
    stages::run(
        &runner.bin,
        [
            "--datadir".to_owned(),
            native_presets.datadir.to_str().unwrap().to_owned(),
            "--load-settings".to_owned(),
            join_paths(native_presets.settings.iter()),
            "--load-filaments".to_owned(),
            join_paths(native_presets.filaments.iter()),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            project_path.to_str().unwrap().to_owned(),
        ]
        .into_iter()
        .chain(native_args),
        stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    let native = fs::read(output.join("plate_1.gcode")).unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &fs::read(&project_path).unwrap(),
            request(),
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&native, &actual).unwrap();
}

#[test]
fn mapped_tower_keeps_tower_for_distinct_cli_colours() {
    run_case(Case {
        name: "distinct-cli-colours",
        filament_count: 2,
        overrides: json!({"filament_colour": ["#102030", "#E0C040"]}),
    });
}
