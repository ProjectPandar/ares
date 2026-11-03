//! PrintObject.cpp93-130 group-frame public RED, approved SPEC49cf8a57a.
use crate::{
    golden,
    runner::{OrcaRunner, stages},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

const MANIFEST: &str =
    include_str!("../../../../tests/parity/print-object-centering/provenance.json");

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn compare(label: &str, bytes: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("centering Native oracle not configured");
        return;
    };
    let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
    let case = &manifest["cases"][label];
    assert_eq!(sha(bytes), case["input_sha256"].as_str().unwrap());
    let project = ares_core::load_project(bytes).unwrap();
    assert_eq!(
        project.objects().len(),
        case["source_objects"].as_u64().unwrap() as usize
    );
    let object = project.objects().first().unwrap();
    assert_eq!(
        object.volumes().len(),
        case["source_volumes"].as_u64().unwrap() as usize
    );
    assert_eq!(
        object.instances().len(),
        case["instances"].as_array().unwrap().len()
    );
    for (actual, expected) in object
        .instances()
        .iter()
        .zip(case["instances"].as_array().unwrap())
    {
        assert_eq!(actual.printable(), expected["printable"].as_bool().unwrap());
    }
    let settings = crate::runner::application::settings(bytes).unwrap();
    for (key, value) in case["settings"].as_object().unwrap() {
        assert_eq!(&settings[key], value, "input {key}");
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("centering-c1-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), bytes).unwrap();
    fs::create_dir(root.join("native")).unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        root.join("native").to_str().unwrap().into(),
        root.join("input.3mf").to_str().unwrap().into(),
    ];
    let native_binary = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(
        root.join("launch.json"),
        serde_json::to_vec_pretty(&json!({
            "wrapper":runner.bin(), "native_binary":native_binary,
            "native_binary_sha256":sha(&fs::read(&native_binary).unwrap()), "args":args,
            "input_sha256":sha(bytes), "case":case, "settings":settings,
            "source":"Model.cpp1586-1599; PrintObject.cpp93-130; PrintApply.cpp138-170"
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let result: Value =
        serde_json::from_slice(&fs::read(root.join("native/result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], 0, "Native admission {root:?}");
    let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let effective: serde_json::Map<String, Value> = std::str::from_utf8(&reference)
        .unwrap()
        .lines()
        .filter_map(|line| {
            let (k, v) = line.strip_prefix("; ")?.split_once(" = ")?;
            Some((k.into(), json!(v)))
        })
        .collect();
    for key in [
        "wall_generator",
        "seam_position",
        "enable_support",
        "skirt_loops",
        "brim_type",
        "brim_width",
    ] {
        assert_eq!(
            effective[key], settings[key],
            "Native effective {key}: {root:?}"
        );
    }
    fs::write(
        root.join("native.effective.json"),
        serde_json::to_vec_pretty(&effective).unwrap(),
    )
    .unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(gcode) = &api {
        fs::write(root.join("ares.api.gcode"), gcode).unwrap();
    }
    let api_comparison = api
        .as_ref()
        .map_err(|e| format!("API: {e:?}"))
        .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g));
    let output = root.join("ares.cli.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = Command::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&output);
    let cli_comparison = cli_bytes
        .as_ref()
        .map_err(|e| format!("CLI: {e}"))
        .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g));
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "label":label,"input_sha256":sha(bytes),"native_sha256":sha(&reference),
        "api_admitted":api.is_ok(),"api_error":api.as_ref().err().map(|e|format!("{e:?}")),
        "api_sha256":api.as_ref().ok().map(|g|sha(g)),"api_comparison":api_comparison,
        "cli_code":cli.status.code(),"cli_sha256":cli_bytes.as_ref().ok().map(|g|sha(g)),
        "cli_binary_sha256":sha(&fs::read(binary).unwrap()),"cli_comparison":cli_comparison,
        "strict_parity_required":true,"comparator":"unchanged golden::compare_ordered_bytes_generator_only"
    })).unwrap()).unwrap();
    assert!(api.is_ok(), "C1 public API admission RED {root:?}: {api:?}");
    assert!(
        cli.status.success(),
        "C1 CLI admission RED {root:?}: {cli:?}"
    );
    api_comparison.unwrap_or_else(|e| panic!("{root:?}: {e}"));
    cli_comparison.unwrap_or_else(|e| panic!("{root:?}: {e}"));
    assert_eq!(
        api.unwrap(),
        cli_bytes.unwrap(),
        "API/CLI raw equality {root:?}"
    );
}

macro_rules! case {
    ($name:ident, $label:literal) => {
        #[test]
        fn $name() {
            compare(
                $label,
                include_bytes!(concat!(
                    "../../../../tests/parity/print-object-centering/",
                    $label,
                    ".3mf"
                )),
            );
        }
    };
}
case!(c1_single, "single");
case!(c1_xy_copy, "xy-copy");
case!(c1_rotation_group, "rotation-group");
case!(c1_scale_group, "scale-group");
case!(c1_mirror_group, "mirror-group");
case!(c1_rotation_thirty, "rotation-thirty");
case!(c1_reversed_source_order, "reversed-source-order");
case!(c1_nonprintable_first, "nonprintable-first");
case!(c1_fractional_shifts, "fractional-shifts");
case!(c1_z_shrink, "z-shrink");
case!(c1_skirt_groups, "skirt-groups");
case!(c1_brim_groups, "brim-groups");
case!(c1_support_groups, "support-groups");

#[test]
fn c1_rotation_exclusion() {
    let input =
        include_bytes!("../../../../tests/parity/print-object-centering/rotation-exclusion.3mf");
    let settings = crate::runner::application::settings(input).unwrap();
    assert_eq!(settings["exclude_object"], json!("1"));
    compare("rotation-exclusion", input);
}
