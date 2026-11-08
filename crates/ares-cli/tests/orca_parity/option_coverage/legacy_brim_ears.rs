//! PrintConfig.cpp:1751-1756; Brim.cpp:455-458,536-554.
//! Failure inventory: legacy bool selecting geometry, lost config echo,
//! inactive brim selector, collapsed outer/ear shapes, API/CLI mismatch.
use crate::{
    golden,
    runner::{OrcaRunner, stages},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
};
struct Case {
    label: &'static str,
    selector: &'static str,
    enabled: bool,
    project: &'static [u8],
}
const CASES: [Case; 6] = [
    Case {
        label: "no-brim-legacy-disabled",
        selector: "no_brim",
        enabled: false,
        project: include_bytes!(
            "../../../../../tests/parity/legacy-brim-ears/no-brim-legacy-disabled.3mf"
        ),
    },
    Case {
        label: "no-brim-legacy-enabled",
        selector: "no_brim",
        enabled: true,
        project: include_bytes!(
            "../../../../../tests/parity/legacy-brim-ears/no-brim-legacy-enabled.3mf"
        ),
    },
    Case {
        label: "outer-only-legacy-disabled",
        selector: "outer_only",
        enabled: false,
        project: include_bytes!(
            "../../../../../tests/parity/legacy-brim-ears/outer-only-legacy-disabled.3mf"
        ),
    },
    Case {
        label: "outer-only-legacy-enabled",
        selector: "outer_only",
        enabled: true,
        project: include_bytes!(
            "../../../../../tests/parity/legacy-brim-ears/outer-only-legacy-enabled.3mf"
        ),
    },
    Case {
        label: "ears-legacy-disabled",
        selector: "brim_ears",
        enabled: false,
        project: include_bytes!(
            "../../../../../tests/parity/legacy-brim-ears/ears-legacy-disabled.3mf"
        ),
    },
    Case {
        label: "ears-legacy-enabled",
        selector: "brim_ears",
        enabled: true,
        project: include_bytes!(
            "../../../../../tests/parity/legacy-brim-ears/ears-legacy-enabled.3mf"
        ),
    },
];
fn verify_input(case: &Case) {
    let mut zip = zip::ZipArchive::new(Cursor::new(case.project)).unwrap();
    let mut bytes = Vec::new();
    zip.by_name("Metadata/project_settings.config")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    let settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for (key, value) in [
        ("brim_type", case.selector),
        ("brim_ears", if case.enabled { "1" } else { "0" }),
        ("brim_width", "4"),
        ("brim_object_gap", "0.1"),
        ("brim_ears_max_angle", "125"),
        ("brim_ears_detection_length", "1"),
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("single_extruder_multi_material", "0"),
    ] {
        assert_eq!(settings[key], json!(value), "{key}");
    }
    assert_eq!(settings["nozzle_diameter"], json!(["0.4"]));
}
fn commands(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with(';'))
        .collect()
}
fn brim_motion(bytes: &[u8]) -> Vec<&str> {
    let mut role = "";
    let mut out = Vec::new();
    for l in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(value) = l.strip_prefix(";TYPE:") {
            role = value;
        }
        if role == "Brim" && l.starts_with("G1 ") && l.contains(" E") {
            out.push(l);
        }
    }
    out
}
fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn native(runner: &OrcaRunner, directory: &Path, project: &[u8]) -> Vec<u8> {
    fs::create_dir(directory).unwrap();
    let path = directory.join("input.3mf");
    fs::write(&path, project).unwrap();
    let output = directory.join("native");
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        output.to_str().unwrap().into(),
        path.to_str().unwrap().into(),
    ];
    let producer = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(
        directory.join("native.launch.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256":sha(project),"native_binary":producer,
            "native_sha256":sha(&fs::read(&producer).unwrap()),
            "wrapper":runner.bin(),"args":args,
            "seed":std::env::var("ORCA_REPLAY_SEED").ok(),
            "cpu":std::env::var("ORCA_REPLAY_CPU").ok()
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, directory).unwrap();
    let result = fs::read(output.join("result.json")).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(metadata["return_code"], 0, "{directory:?}");
    let reference = fs::read(output.join("plate_1.gcode")).unwrap();
    assert!(!reference.is_empty());
    reference
}

fn compare(label: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    compare_with(&runner, CASES.iter().find(|c| c.label == label).unwrap());
}
fn compare_with(runner: &OrcaRunner, case: &Case) {
    verify_input(case);
    let root = tempfile::Builder::new()
        .prefix(&format!("legacy-brim-{}-", case.label))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let project = case.project;
    let reference = native(runner, &directory, project);
    let text = std::str::from_utf8(&reference).unwrap();
    assert!(text.contains(&format!("; brim_ears = {}\n", u8::from(case.enabled))));
    assert!(text.contains(&format!("; brim_type = {}\n", case.selector)));
    let brim_lines = brim_motion(&reference);
    assert_eq!(brim_lines.is_empty(), case.selector == "no_brim");
    if case.enabled {
        let other = CASES
            .iter()
            .find(|c| c.selector == case.selector && !c.enabled)
            .unwrap();
        let baseline = native(runner, &root.join("bool-off"), other.project);
        assert_eq!(
            commands(&reference),
            commands(&baseline),
            "legacy bool must not change Native geometry"
        );
    }
    if case.label == "ears-legacy-disabled" {
        let outer = native(runner, &root.join("outer"), CASES[2].project);
        assert_ne!(
            brim_lines,
            brim_motion(&outer),
            "ear selector must differ from outer contour"
        );
    }
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
        fs::write(directory.join("ares.validated.gcode"), gcode).unwrap();
    }
    let api_comparison = public
        .as_ref()
        .map_err(|error| format!("validated API: {error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, gcode));
    let output_dir = directory.join("ares");
    fs::create_dir(&output_dir).unwrap();
    let output = output_dir.join("plate_1.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = crate::runner::application::CliCommand::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &cli.stderr).unwrap();
    let cli_comparison = fs::read(&output)
        .map_err(|error| format!("CLI output: {error}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, &gcode));
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label":case.label,"selector":case.selector,"bool":case.enabled,
            "input_sha256":sha(project),"native_sha256":sha(&reference),
            "validated_api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "CLI_code":cli.status.code(),"CLI_binary":binary,
            "CLI_binary_sha256":sha(&fs::read(binary).unwrap()),
            "brim_extrusion_count":brim_lines.len(),
            "api_error":public.as_ref().err().map(|error| format!("{error:?}"))
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
}

#[test]
fn legacy_brim_no_brim_legacy_disabled_whole_output() {
    compare("no-brim-legacy-disabled");
}

#[test]
fn legacy_brim_no_brim_legacy_enabled_whole_output() {
    compare("no-brim-legacy-enabled");
}

#[test]
fn legacy_brim_outer_only_legacy_disabled_whole_output() {
    compare("outer-only-legacy-disabled");
}

#[test]
fn legacy_brim_outer_only_legacy_enabled_whole_output() {
    compare("outer-only-legacy-enabled");
}

#[test]
fn legacy_brim_ears_legacy_disabled_whole_output() {
    compare("ears-legacy-disabled");
}

#[test]
fn legacy_brim_ears_legacy_enabled_whole_output() {
    compare("ears-legacy-enabled");
}
