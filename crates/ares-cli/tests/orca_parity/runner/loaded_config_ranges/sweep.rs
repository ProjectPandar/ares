use super::*;

pub(super) async fn run() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    // K2 0.4 now resolves its `solid_infill_filament`/
    // `internal_solid_filament_id` legacy-alias collision via upstream
    // overwrite semantics (`f5c869df`), so every machine below reaches the
    // O3 range loop and is held to strict parity.
    const CASES: &[(&str, &str)] = &[
        ("Creality", "Creality K2 0.4 nozzle"),
        ("Creality", "Creality SPARKX i7 0.2 nozzle"),
        ("Creality", "Creality SPARKX i7 0.4 nozzle"),
        ("Creality", "Creality SPARKX i7 0.6 nozzle"),
        ("Creality", "Creality SPARKX i7 0.8 nozzle"),
        ("Anycubic", "Anycubic Kobra S1 Max 0.25 nozzle"),
        ("Anycubic", "Anycubic Kobra S1 Max 0.4 nozzle"),
        ("Anycubic", "Anycubic Kobra S1 Max 0.6 nozzle"),
        ("Anycubic", "Anycubic Kobra S1 Max 0.8 nozzle"),
        ("Anycubic", "Anycubic Kobra X 0.4 nozzle"),
        ("SeeMeCNC", "SeeMeCNC BOSSdelta 500 0521 0.4 nozzle"),
        ("SeeMeCNC", "SeeMeCNC BOSSdelta 500 0521 0.5 nozzle"),
        ("SeeMeCNC", "SeeMeCNC BOSSdelta 500 0521 0.7 nozzle"),
        ("SeeMeCNC", "SeeMeCNC BOSSdelta 500 0521 1.0 nozzle"),
    ];
    let root = crate::runner::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let input = fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    let mut report = Vec::new();
    let mut failures = Vec::new();
    for (vendor_name, machine) in CASES {
        let profiles = crate::presets::VendorProfiles::load(&profiles_root, vendor_name).unwrap();
        let selection = crate::select_printer(&profiles, vendor_name, machine)
            .unwrap_or_else(|error| panic!("{vendor_name}/{machine}: {error}"));
        let catalog = json!({"vendors": [
            vendor(&profiles_root, "OrcaFilamentLibrary"),
            vendor(&profiles_root, vendor_name),
        ], "imports": []});
        let sel_json = json!({
            "machine": machine,
            "process": selection.process,
            "filaments": selection.filaments,
        });
        let directory = tempfile::Builder::new()
            .prefix(&format!("o3-sweep-{}-", machine.replace(' ', "_")))
            .tempdir()
            .unwrap()
            .keep();
        for (kind, fields) in [
            ("machine", profiles.machine(machine).unwrap()),
            ("process", profiles.process(&selection.process).unwrap()),
            (
                "filament",
                profiles.filament(&selection.filaments[0]).unwrap(),
            ),
        ] {
            write_native_preset(&directory.join(format!("{kind}.json")), kind, fields);
        }
        fs::write(directory.join("cube10.stl"), &input).unwrap();
        write_json(&directory.join("catalog.json"), &catalog);
        write_json(&directory.join("selection.json"), &sel_json);
        // PresetBundle.cpp:7781-7848 resolves the same selected vendor leaves as Ares.
        let presets = super::super::native_user_presets::resolved(
            &directory,
            &profiles_root,
            &catalog,
            &sel_json,
        );
        let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
        let native = stages::run(
            runner.bin(),
            [
                "--datadir".to_owned(),
                presets.datadir.to_str().unwrap().to_owned(),
                "--load-settings".to_owned(),
                crate::runner::join_paths(presets.settings.iter()),
                "--load-filaments".to_owned(),
                crate::runner::join_paths(presets.filaments.iter()),
                "--arrange".to_owned(),
                "1".to_owned(),
                "--export-3mf".to_owned(),
                path("native.3mf"),
                path("cube10.stl"),
            ],
            stages::Stage::Export,
            &directory,
        );
        let native_err = native
            .as_ref()
            .err()
            .and_then(|failure| failure.captured.as_ref())
            .map(|captured| String::from_utf8_lossy(&captured.stderr).into_owned());
        let request = ProfileSliceRequest {
            filename: "cube10.stl",
            catalog_json: &serde_json::to_string(&catalog).unwrap(),
            selection_json: &sel_json.to_string(),
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
        };
        let validated = ares_core::validate_with_profiles(&input, request);
        report.push(json!({
            "machine": machine,
            "process": selection.process,
            "native_stderr": native_err,
            "ares_validated": validated.as_ref().map_err(ToString::to_string),
        }));
        // Ares must reject with the identical `key: text` lines (the body
        // after the `Param values in 3mf/config error: ` header).
        let native_text = native_err
            .as_deref()
            .and_then(|text| text.split("Param values in 3mf/config error: \n").nth(1))
            .map(|body| body.trim_end().to_owned());
        let matched = matches!(
            (&validated, native_text.as_deref()),
            (Err(SliceError::InvalidValuesInProject(text)), Some(native)) if text == native
        );
        report.last_mut().unwrap()["native_text"] = json!(native_text);
        report.last_mut().unwrap()["matched"] = json!(matched);
        if !matched {
            failures.push(format!(
                "{vendor_name}/{machine}: ares={validated:?} native={native_text:?}"
            ));
        }
    }
    write_json(
        &runner.work.join("o3-sweep-report.json"),
        &Value::Array(report),
    );
    assert!(
        failures.is_empty(),
        "O3 sweep divergences:\n{}",
        failures.join("\n")
    );
}
