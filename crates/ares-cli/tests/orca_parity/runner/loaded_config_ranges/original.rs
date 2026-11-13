use super::*;

/// One O3 case: native `--load-settings` rejects with exit 238 and the
/// exact `key: <serialize()> not in range [min,max]` text; Ares must
/// surface the same `SliceError::InvalidValuesInProject` text via
/// `validate_with_profiles` / `slice_with_profiles_validated`.
pub(super) async fn original_profile(case: &RangeCase<'_>) {
    let root = super::super::super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, case.vendor_name).unwrap();
    let machine_fields = profiles.machine(case.machine).unwrap();
    let process_fields = profiles.process(case.process).unwrap();
    let filament_fields = profiles.filament(case.filament).unwrap();
    let catalog = json!({"vendors": [
        vendor(&profiles_root, "OrcaFilamentLibrary"),
        vendor(&profiles_root, case.vendor_name),
    ], "imports": []});
    let selection = json!({
        "machine": case.machine,
        "process": case.process,
        "filaments": [case.filament],
    });
    let directory = tempfile::Builder::new()
        .prefix(&format!("o3-{}-", case.machine.replace(' ', "_")))
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    let input = fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    fs::write(directory.join("cube10.stl"), &input).unwrap();
    for (kind, fields) in [
        ("machine", machine_fields),
        ("process", process_fields),
        ("filament", filament_fields),
    ] {
        write_native_preset(&directory.join(format!("{kind}.json")), kind, fields);
    }
    let expected_stderr = format!("Param values in 3mf/config error: \n{}\n", case.error_lines);
    let native = OrcaRunner::from_env().unwrap().map(|runner| {
        let presets = super::super::native_user_presets::resolved(
            &directory,
            &profiles_root,
            &catalog,
            &selection,
        );
        let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
        let failure = stages::run(
            runner.bin(),
            [
                "--datadir".to_owned(),
                presets.datadir.to_str().unwrap().to_owned(),
                "--load-settings".to_owned(),
                super::super::super::join_paths(presets.settings.iter()),
                "--load-filaments".to_owned(),
                super::super::super::join_paths(presets.filaments.iter()),
                "--arrange".to_owned(),
                "1".to_owned(),
                "--export-3mf".to_owned(),
                path("native.3mf"),
                path("cube10.stl"),
            ],
            stages::Stage::Export,
            &directory,
        )
        .unwrap_err();
        let captured = failure.captured.as_ref().unwrap();
        assert_eq!(captured.code, Some(if cfg!(unix) { 238 } else { -18 }));
        assert_eq!(
            std::str::from_utf8(&captured.stderr).unwrap(),
            expected_stderr,
            "native diagnostic; evidence {directory:?}"
        );
        assert!(!directory.join("native.3mf").exists());
        json!({"code": captured.code, "stderr": expected_stderr, "evidence": failure.evidence})
    });
    let catalog_json = serde_json::to_string(&catalog).unwrap();
    let selection_json = serde_json::to_string(&selection).unwrap();
    let overrides = ConfigOverrides::default();
    let request = ProfileSliceRequest {
        filename: "cube10.stl",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: Placement::None,
    };
    let loaded = ares_core::profile_settings(&input, request).await;
    let placed = ares_core::profile_placed_project(&input, request).await;
    assert!(
        placed.is_ok(),
        "placed scene must allow editing invalid config; evidence {directory:?}"
    );
    let validated = ares_core::validate_with_profiles(&input, request);
    let sliced = ares_core::slice_with_profiles_validated(
        &input,
        request,
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await;
    if let Ok(gcode) = &sliced {
        fs::write(directory.join("ares.gcode"), gcode).unwrap();
    }
    write_json(
        &directory.join("result.json"),
        &json!({
            "native": native,
            "expected": case.error_lines,
            "loaded": loaded.as_ref().map(|view| json!(view)).map_err(ToString::to_string),
            "validated": validated.as_ref().map_err(ToString::to_string),
            "sliced": sliced.as_ref().map(|_| "GCODE").map_err(ToString::to_string),
        }),
    );
    loaded.unwrap();
    assert!(
        matches!(validated, Err(SliceError::InvalidValuesInProject(text)) if text == case.error_lines),
        "validated must surface the same key:text lines; evidence {directory:?}"
    );
    assert!(
        matches!(sliced, Err(SliceError::InvalidValuesInProject(text)) if text == case.error_lines),
        "sliced must surface the same key:text lines; evidence {directory:?}"
    );
    assert!(!directory.join("ares.gcode").exists());
}
