use crate::{
    OrcaRunner, golden,
    runner::{application::cli_command, stages},
};
use std::{fmt::Write, fs};

#[tokio::test]
async fn fresh_stl_loaded_presets_apply_orientation_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("orient-fresh-stl-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets =
        crate::load_settings_cli::preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    // No arrangement is in scope: keep both raw and rotated STL inside a centered bed.
    let mut machine: serde_json::Value =
        serde_json::from_slice(&fs::read(&presets[0]).unwrap()).unwrap();
    machine["printable_area"] = serde_json::json!(["-110x-110", "110x-110", "110x110", "-110x110"]);
    fs::write(&presets[0], serde_json::to_vec_pretty(&machine).unwrap()).unwrap();
    let settings = format!("{};{}", presets[0], presets[1]);
    let filaments = presets[2..].join(";");
    let mut input = String::from("solid asymmetric\n");
    let vertices = [[0, 0, 0], [11, 0, 0], [1, 7, 0], [2, 1, 17]];
    for face in [[0, 2, 1], [0, 1, 3], [1, 2, 3], [2, 0, 3]] {
        input.push_str("facet normal 0 0 0\nouter loop\n");
        for vertex in face {
            let [x, y, z] = vertices[vertex];
            writeln!(input, "vertex {x} {y} {z}").unwrap();
        }
        input.push_str("endloop\nendfacet\n");
    }
    input.push_str("endsolid asymmetric\n");
    let path = root.join("fresh.stl");
    fs::write(&path, &input).unwrap();
    let mut native_streams = Vec::new();
    for orient in ["0", "1"] {
        let output = root.join(format!("native-{orient}"));
        fs::create_dir(&output).unwrap();
        stages::run_live(
            runner.bin(),
            [
                "--orient",
                orient,
                "--load-settings",
                &settings,
                "--load-filaments",
                &filaments,
                "--arrange",
                "0",
                "--slice",
                "0",
                "--outputdir",
                output.to_str().unwrap(),
                path.to_str().unwrap(),
            ]
            .into_iter()
            .map(str::to_owned),
            stages::Stage::Slice,
            &root,
        )
        .unwrap();
        let native = fs::read(output.join("plate_1.gcode")).unwrap();
        let ares_path = root.join(format!("ares-{orient}.gcode"));
        let actual = cli_command()
            .args([
                "slice",
                "--orient",
                orient,
                "--load-settings",
                &settings,
                "--load-filaments",
                &filaments,
                "-o",
            ])
            .arg(&ares_path)
            .arg(&path)
            .output()
            .unwrap();
        fs::write(root.join(format!("ares-{orient}.stdout")), &actual.stdout).unwrap();
        fs::write(root.join(format!("ares-{orient}.stderr")), &actual.stderr).unwrap();
        assert!(actual.status.success(), "Ares failed: {root:?}");
        let ares = fs::read(&ares_path).unwrap();
        fs::write(root.join(format!("comparison-{orient}.json")),serde_json::to_vec_pretty(&serde_json::json!({"input_sha256":crate::runner::native_api_cli::sha(input.as_bytes()),"native_sha256":crate::runner::native_api_cli::sha(&native),"ares_sha256":crate::runner::native_api_cli::sha(&ares),"comparison":"complete ordered bytes; generated-by only"})).unwrap()).unwrap();
        let native =
            golden::normalize_one_generator_line(&native, golden::GeneratorKind::Orca).unwrap();
        let ares =
            golden::normalize_one_generator_line(&ares, golden::GeneratorKind::Ares).unwrap();
        assert!(
            golden::first_difference(&native, &ares).is_none(),
            "{:?}; {root:?}",
            golden::first_difference(&native, &ares).map(|difference| difference.to_string())
        );
        native_streams.push(native);
    }
    assert!(
        native_streams[0] != native_streams[1],
        "Native action must change output"
    );
    let texts: Vec<_> = presets
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect();
    let files: Vec<_> = presets
        .iter()
        .zip(&texts)
        .map(|(label, json)| ares_core::LoadSettingsFile { label, json })
        .collect();
    let overrides = ares_core::ConfigOverrides::default();
    for (index, orient) in [false, true, false].into_iter().enumerate() {
        let mut report = ares_core::SlicedPlate::default();
        let core = ares_core::slice_with_load_settings(
            input.as_bytes(),
            ares_core::LoadSettingsSliceRequest {
                filename: "fresh.stl",
                settings: &files[..2],
                filaments: &files[2..],
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::None,
                orient,
                sliced_plate: &mut report,
            },
            ares_core::GenerationMetadata::default(),
            ares_core::SlicerContext::new(0),
        )
        .await
        .unwrap();
        fs::write(root.join(format!("core-repeat-{index}.gcode")), &core).unwrap();
        fs::write(root.join(format!("core-repeat-{index}.json")), serde_json::to_vec_pretty(&serde_json::json!({"orient":orient,"input_sha256":crate::runner::native_api_cli::sha(input.as_bytes()),"core_sha256":crate::runner::native_api_cli::sha(&core),"comparison":"complete Native ordered bytes; generated-by only"})).unwrap()).unwrap();
        let core =
            golden::normalize_one_generator_line(&core, golden::GeneratorKind::Ares).unwrap();
        let expected = &native_streams[usize::from(orient)];
        assert!(
            golden::first_difference(expected, &core).is_none(),
            "public API replay mismatch: {:?}; {root:?}",
            golden::first_difference(expected, &core).map(|difference| difference.to_string())
        );
    }
    assert_eq!(fs::read(&path).unwrap(), input.as_bytes());
}
