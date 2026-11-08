//! Brim.cpp::outer_inner_brim_area inner-only branch; approved SPEC 696cd308a.
//! Failure inventory is in inner-only-brim-rewrite-spec.md. Public byte API
//! and real CLI are the seams; Native complete output is the independent oracle.
use crate::{
    golden,
    runner::{OrcaRunner, stages},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

#[path = "inner_only_brim/witness.rs"]
mod witness;

const PROVENANCE: &str = include_str!("../../../../tests/parity/inner-only-brim/provenance.json");

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn compare(label: &str, project: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let provenance: Value = serde_json::from_str(PROVENANCE).unwrap();
    let case = &provenance["cases"][label];
    assert_eq!(sha(project), case["input_sha256"].as_str().unwrap());
    let settings = crate::runner::application::settings(project).unwrap();
    for (key, expected) in provenance["common_settings"].as_object().unwrap() {
        assert_eq!(&settings[key], expected, "input {key}");
    }
    for key in ["brim_width", "brim_object_gap"] {
        assert_eq!(settings[key], case[key], "input {key}");
    }
    assert_eq!(settings["nozzle_diameter"], json!(["0.4"]));
    for key in [
        "enable_support",
        "enable_prime_tower",
        "single_extruder_multi_material",
    ] {
        assert_eq!(settings[key], json!("0"), "input {key}");
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("inner-brim-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), project).unwrap();
    fs::write(
        root.join("input.provenance.json"),
        serde_json::to_vec_pretty(case).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("input.settings.json"),
        serde_json::to_vec_pretty(&settings).unwrap(),
    )
    .unwrap();
    let native_dir = root.join("native");
    fs::create_dir(&native_dir).unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        native_dir.to_str().unwrap().into(),
        root.join("input.3mf").to_str().unwrap().into(),
    ];
    let producer = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(root.join("native.launch.json"), serde_json::to_vec_pretty(&json!({
        "label":label,"input_sha256":sha(project),"wrapper":runner.bin(),
        "args":args,"native_binary":producer,"native_binary_sha256":sha(&fs::read(&producer).unwrap()),
        "seed":std::env::var("ORCA_REPLAY_SEED").ok(),
        "source":"Brim.cpp:448-458,529-570,678-711,903-904"
    })).unwrap()).unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let result: Value =
        serde_json::from_slice(&fs::read(native_dir.join("result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], 0, "Native admission: {root:?}");
    let reference = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    let text = std::str::from_utf8(&reference).unwrap();
    let effective: serde_json::Map<String, Value> = text
        .lines()
        .filter_map(|line| {
            let (key, value) = line.strip_prefix("; ")?.split_once(" = ")?;
            Some((key.to_owned(), json!(value)))
        })
        .collect();
    fs::write(
        root.join("native.effective-footer.json"),
        serde_json::to_vec_pretty(&effective).unwrap(),
    )
    .unwrap();
    for key in [
        "brim_type",
        "brim_width",
        "brim_object_gap",
        "brim_use_efc_outline",
    ] {
        assert_eq!(
            effective[key], settings[key],
            "Native effective {key}: {root:?}"
        );
    }
    let native_witness = witness::capture(&reference, case);
    fs::write(
        root.join("native.brim-witness.json"),
        serde_json::to_vec_pretty(&native_witness).unwrap(),
    )
    .unwrap();
    witness::verify(&native_witness, case, &root);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let public = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        &project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    if let Ok(gcode) = &public {
        fs::write(root.join("ares.validated.gcode"), gcode).unwrap();
        fs::write(
            root.join("ares.brim-witness.json"),
            serde_json::to_vec_pretty(&witness::capture(gcode, case)).unwrap(),
        )
        .unwrap();
    }
    let api_comparison = public
        .as_ref()
        .map_err(|error| format!("validated API: {error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, gcode));
    let output = root.join("ares.cli.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = crate::runner::application::CliCommand::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_comparison = fs::read(&output)
        .map_err(|error| format!("CLI output: {error}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, &gcode));
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "label":label,"input_sha256":sha(project),"native_sha256":sha(&reference),
        "native_brim_extrusions":native_witness["motions"].as_array().unwrap().len(),
        "api_admitted":public.is_ok(),"api_error":public.as_ref().err().map(|e|format!("{e:?}")),
        "api_comparison":api_comparison,"cli_comparison":cli_comparison,
        "cli_code":cli.status.code(),"cli_binary":binary,"cli_sha256":sha(&fs::read(binary).unwrap()),
        "comparator":"golden::compare_ordered_bytes_generator_only"
    })).unwrap()).unwrap();
    assert!(
        public.is_ok(),
        "API admission is not geometry RED: {root:?}: {public:?}"
    );
    assert!(
        cli.status.success(),
        "CLI admission is not geometry RED: {root:?}: {cli:?}"
    );
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    witness::verify(
        &witness::capture(public.as_ref().unwrap(), case),
        case,
        &root,
    );
    witness::verify(
        &witness::capture(&fs::read(output).unwrap(), case),
        case,
        &root,
    );
}

macro_rules! case {
    ($name:ident, $label:literal) => {
        #[test]
        fn $name() {
            compare(
                $label,
                include_bytes!(concat!(
                    "../../../../tests/parity/inner-only-brim/",
                    $label,
                    ".3mf"
                )),
            );
        }
    };
}
case!(inner_only_rectangle_whole_output, "rectangle");
case!(inner_only_concave_whole_output, "concave");
case!(inner_only_two_holes_whole_output, "two-holes");
case!(inner_only_narrow_neck_whole_output, "narrow-neck");
case!(inner_only_positive_gap_whole_output, "positive-gap");
case!(inner_only_collapsed_inset_whole_output, "collapsed-inset");
case!(inner_only_zero_width_whole_output, "zero-width");
case!(inner_only_sub_pair_width_whole_output, "sub-pair-width");
case!(inner_only_no_hole_whole_output, "no-hole");
