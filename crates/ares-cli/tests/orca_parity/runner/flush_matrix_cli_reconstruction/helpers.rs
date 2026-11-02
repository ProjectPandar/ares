// Failure inventory: a fully specified missing matrix triggers reconstruction;
// supplied complete matrices are retained. Missing AMS must not decide it,
// and export must scale each nozzle segment once.
fn project(runner: &OrcaRunner, name: &str, changes: &[(&str, Option<Value>)]) -> ExportedCase {
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let selection = crate::select_printer(&profiles, "BBL", "Bambu Lab X2D 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let filament = profiles.filament(&selection.filaments[0]).unwrap();
    let mut filaments = vec![filament.clone(), filament];
    crate::normalize_filament_defaults(&mut filaments);
    let end = format!(
        "; X2D_FM_BEGIN\nM83\nG92 E0\nG1 X10 Y10 F1200\nT1 H-1\nSYNC T5\nM1020 S1\nT0 H-1\nM1020 S0\nG1 X20 Y10 F1200\n; X2D_FM_END\n{}",
        machine["machine_end_gcode"].as_str().unwrap()
    );
    let mut overrides = crate::smoke_case_overrides(&machine, &process);
    overrides.insert("machine_end_gcode".into(), json!(end));
    overrides.insert("disable_m73".into(), json!("0"));
    overrides.insert("filament_map".into(), json!(["1", "1"]));
    let label = format!("x2d-fm/nozzle-types-unchanged/custom-matrix/{name}");
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &overrides,
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let mut exported = vg1_two_filament_inputs::bind_two_filaments(exported);
    let types = json!(["Standard", "Standard"]);
    let matrix = json!(["0", "140", "140", "0", "0", "210", "210", "0"]);
    let requested = [
        ("machine_end_gcode", json!(end)),
        ("filament_map", json!(["1", "1"])),
        ("filament_colour", json!(["#F2754E", "#0000FF"])),
        ("extruder_ams_count", json!([])),
        ("nozzle_volume_type", types),
        ("flush_multiplier", json!(["0.3", "1"])),
        ("flush_volumes_matrix", matrix),
    ];
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            for (key, value) in &requested {
                settings[*key] = value.clone();
            }
            let object = settings.as_object_mut().unwrap();
            for (key, value) in changes {
                match value {
                    Some(value) => object.insert((*key).to_owned(), value.clone()),
                    None => object.remove(*key),
                };
            }
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    exported.project = output.finish().unwrap().into_inner();
    std::fs::write(&exported.project_path, &exported.project).unwrap();
    for (key, value) in requested {
        if !changes.iter().any(|(changed, _)| *changed == key) {
            application::verify_value(&exported.project, key, &value).unwrap();
        }
    }
    for (key, value) in changes {
        if let Some(value) = value {
            application::verify_value(&exported.project, key, value).unwrap();
        } else {
            assert!(
                !application::settings(&exported.project)
                    .unwrap()
                    .contains_key(*key)
            );
        }
    }
    assert_eq!(
        application::settings(&exported.project).unwrap()["nozzle_diameter"],
        json!(["0.4", "0.4"])
    );
    exported
}

fn check(name: &str, changes: &[(&str, Option<Value>)], matrix: Option<&str>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = project(&runner, name, changes);
    let case = runner.slice_case(&exported).unwrap();
    let reference = std::str::from_utf8(&case.reference).unwrap();
    assert!(reference.contains("; X2D_FM_BEGIN\n"));
    assert!(reference.contains("; X2D_FM_END\n"));
    assert!(reference.contains("T1 H-1\n"));
    assert!(reference.contains("T0 H-1\n"));
    if let Some(expected_matrix) = matrix {
        assert!(reference.contains(expected_matrix), "{expected_matrix}");
    }
    assert!(reference.contains("; flush_multiplier = 0.3,1\n"));
    if !changes.iter().any(|(key, _)| *key == "extruder_ams_count") {
        assert!(reference.contains("; extruder_ams_count = 1#0|4#1;1#0|4#1\n"));
    }
    // The v2 receipt, not a profile name or historic cached byte hash, declares
    // actual Native executable/resources/policy for this same-input slice.
    let receipt = std::fs::read_dir(&runner.work)
        .unwrap()
        .map(|entry| entry.unwrap().path().join("cache.json"))
        .filter(|path| path.is_file())
        .map(|path| serde_json::from_slice::<Value>(&std::fs::read(path).unwrap()).unwrap())
        .find(|receipt| receipt["stage"] == "Slice")
        .unwrap();
    let outcome = crate::artifacts::compare(
        &crate::artifacts::root_from_env().unwrap(),
        &case.label,
        Ok(&case.project),
        Ok(&case.reference),
        json!({"kind":"native-cache-v2", "producer_identity":receipt["producer_digest"],
            "producer":receipt["producer"], "request_digest":receipt["request_digest"]}),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
