use super::super::{
    OrcaRunner, cache_identity,
    command::{self, Limits},
    repo_root,
    stages::Stage,
};
use super::{vendor, write_json};
use ares_core::{
    ConfigOverrides, GenerationMetadata, ProfileSliceRequest, SliceError, SlicerContext,
};
use serde_json::json;
use std::time::Duration;

#[test]
fn validated_profile_checks_cli_overrides_before_loading() {
    check(false);
}

#[test]
fn processed_validated_profile_checks_cli_overrides_before_loading() {
    check(true);
}

fn check(processed: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = repo_root();
    let input =
        std::fs::read(root.join("tests/parity/validation/minimum-width-ender3s1plus-0.2.3mf"))
            .unwrap();
    assert_eq!(
        cache_identity::raw_digest(&input),
        "edb48192a609ede526e8e09e805b845ff99494c0d2d2b31a8655ca9b0ac89110"
    );
    let path = runner.work.join("input.3mf");
    std::fs::write(&path, &input).unwrap();
    let native_dir = runner.work.join("native");
    std::fs::create_dir(&native_dir).unwrap();
    let error = command::run(
        runner.bin(),
        [
            "--bridge-flow".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            native_dir.to_string_lossy().into_owned(),
            path.to_string_lossy().into_owned(),
        ],
        &[("LC_ALL", "C")],
        Limits {
            stage: Stage::Slice,
            work: &runner.work,
            timeout: Duration::from_secs(120),
        },
    )
    .unwrap_err();
    let captured = error.captured.unwrap();
    let expected = "bridge_flow: invalid value 0.000000";
    assert_eq!(captured.code, Some(if cfg!(unix) { 254 } else { -2 }));
    assert_eq!(
        captured.stderr,
        format!("Params in command line error: \n{expected}\nsetup params error\n").as_bytes()
    );
    assert!(captured.stdout.is_empty());
    assert!(!native_dir.join("plate_1.gcode").exists());
    assert!(!native_dir.join("result.json").exists());
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "Creality").unwrap();
    let machine = "Creality Ender-3 S1 Plus 0.4 nozzle";
    let process = profiles.compatible_process(machine).unwrap();
    let catalog = json!({
        "vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "Creality")],
        "imports": [],
    });
    let selection = json!({
        "machine": machine, "process": process, "filaments": ["Generic PLA @Creality"],
    });
    write_json(&runner.work.join("catalog.json"), &catalog);
    write_json(&runner.work.join("selection.json"), &selection);
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::from_json(br#"{"bridge_flow":"0"}"#).unwrap();
    let request = ProfileSliceRequest {
        filename: "input.3mf",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: ares_core::Placement::None,
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let metadata = GenerationMetadata::default();
    let result = if processed {
        runtime
            .block_on(ares_core::slice_with_profiles_processed_validated(
                &input,
                request,
                metadata,
                SlicerContext::new(0),
            ))
            .map(|output| output.gcode)
    } else {
        runtime.block_on(ares_core::slice_with_profiles_validated(
            &input,
            request,
            metadata,
            SlicerContext::new(0),
        ))
    };
    write_json(
        &runner.work.join("public-result.json"),
        &json!({
            "processed": processed, "input_sha256": cache_identity::raw_digest(&input),
            "native_exit": captured.code, "native_stderr": String::from_utf8_lossy(&captured.stderr),
            "public": result.as_ref().map(Vec::len).map_err(|error| format!("{error:?}")),
        }),
    );
    assert!(
        matches!(result, Err(SliceError::InvalidInput(ref message)) if message == expected),
        "CLI override preflight must run first: {result:?}; {:?}",
        runner.work
    );
}
