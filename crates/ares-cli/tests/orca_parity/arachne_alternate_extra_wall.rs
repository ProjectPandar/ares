//! Public Native/API/CLI RED for process_arachne alternate count; SPEC c3d238a8a.
use crate::{
    golden,
    runner::{OrcaRunner, stages},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
    process::Command,
};

#[path = "arachne_alternate_extra_wall/witness.rs"]
mod witness;

const PROVENANCE: &str =
    include_str!("../../../../tests/parity/arachne-alternate-extra-wall/provenance.json");

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The fixture with `edits` applied to its project settings; no edits keep its bytes.
fn with_settings(fixture: &[u8], edits: &[(&str, &str)]) -> Vec<u8> {
    if edits.is_empty() {
        return fixture.to_vec();
    }
    let mut input = zip::ZipArchive::new(Cursor::new(fixture)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            for (key, value) in edits {
                settings[*key] = json!(value);
            }
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn compare_pair(label: &str, edits: &[(&str, &str)], off: &[u8], on: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("Arachne alternate wall: Native oracle not configured");
        return;
    };
    let manifest: Value = serde_json::from_str(PROVENANCE).unwrap();
    let root = tempfile::Builder::new()
        .prefix(&format!("arachne-alternate-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let mut native_witnesses = Vec::new();
    let mut project_settings = Vec::new();
    let mut failures = Vec::new();
    for (mode, fixture) in [("off", off), ("on", on)] {
        let input = &manifest["pairs"][label]["cases"][mode];
        assert_eq!(sha(fixture), input["input_sha256"].as_str().unwrap());
        let settings = crate::runner::application::settings(fixture).unwrap();
        for (key, value) in input["settings"].as_object().unwrap() {
            assert_eq!(&settings[key], value, "input {label}/{mode}/{key}");
        }
        let project = &with_settings(fixture, edits)[..];
        let settings = crate::runner::application::settings(project).unwrap();
        let case_root = root.join(mode);
        let native = case_root.join("native");
        fs::create_dir_all(&native).unwrap();
        fs::write(case_root.join("input.3mf"), project).unwrap();
        fs::write(
            case_root.join("input.settings.json"),
            serde_json::to_vec_pretty(&settings).unwrap(),
        )
        .unwrap();
        let args = vec![
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            native.to_str().unwrap().into(),
            case_root.join("input.3mf").to_str().unwrap().into(),
        ];
        let producer = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
        fs::write(
            case_root.join("native.launch.json"),
            serde_json::to_vec_pretty(&json!({
                "wrapper":runner.bin(),"args":args,"native_binary":producer,
                "native_binary_sha256":sha(&fs::read(&producer).unwrap()),
                "seed":std::env::var("ORCA_REPLAY_SEED").ok(),"input_sha256":sha(project),
                "source":"PerimeterGenerator.cpp::process_arachne:2131-2175,2224-2244"
            }))
            .unwrap(),
        )
        .unwrap();
        stages::run(runner.bin(), args, stages::Stage::Slice, &case_root).unwrap();
        let result: Value =
            serde_json::from_slice(&fs::read(native.join("result.json")).unwrap()).unwrap();
        assert_eq!(result["return_code"], 0, "Native admission: {case_root:?}");
        let reference = fs::read(native.join("plate_1.gcode")).unwrap();
        let text = std::str::from_utf8(&reference).unwrap();
        let effective: serde_json::Map<String, Value> = text
            .lines()
            .filter_map(|line| {
                let (key, value) = line.strip_prefix("; ")?.split_once(" = ")?;
                Some((key.to_owned(), json!(value)))
            })
            .collect();
        fs::write(
            case_root.join("native.effective.json"),
            serde_json::to_vec_pretty(&effective).unwrap(),
        )
        .unwrap();
        for key in [
            "wall_generator",
            "wall_loops",
            "sparse_infill_density",
            "only_one_wall_top",
            "only_one_wall_first_layer",
            "spiral_mode",
        ]
        .into_iter()
        .chain(edits.iter().map(|(key, _)| *key))
        {
            assert_eq!(
                effective[key], settings[key],
                "effective {key}: {case_root:?}"
            );
        }
        // PrintConfig.cpp:8567-8580 normalizes the spiral control's alternate
        // flag to false before the perimeter owner is invoked.
        let expected_alternate = if label == "spiral" {
            json!("0")
        } else {
            settings["alternate_extra_wall"].clone()
        };
        assert_eq!(effective["alternate_extra_wall"], expected_alternate);
        let captured = witness::capture(&reference);
        fs::write(
            case_root.join("native.wall-witness.json"),
            serde_json::to_vec_pretty(&captured).unwrap(),
        )
        .unwrap();
        native_witnesses.push(captured);
        project_settings.push(settings);
        failures.extend(compare_public(&case_root, project, &reference));
    }
    witness::verify(
        label,
        &project_settings,
        &native_witnesses[0],
        &native_witnesses[1],
    );
    fs::write(
        root.join("pair-evidence.json"),
        serde_json::to_vec_pretty(&json!({
            "label":label,"native_witnesses":native_witnesses,"public_failures":failures,
            "scope":"full ordered bytes + independent emitted wall/infill witnesses"
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        failures.is_empty(),
        "{label}: {failures:?}; artifacts {root:?}"
    );
}

fn compare_public(root: &Path, project: &[u8], reference: &[u8]) -> Vec<String> {
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
            root.join("ares.wall-witness.json"),
            serde_json::to_vec_pretty(&witness::capture(gcode)).unwrap(),
        )
        .unwrap();
    }
    let api_comparison = public
        .as_ref()
        .map_err(|error| format!("API: {error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(reference, gcode));
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
    let cli_comparison = fs::read(&output)
        .map_err(|e| format!("CLI output: {e}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(reference, &gcode));
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "input_sha256":sha(project),"native_sha256":sha(reference),
        "api_admitted":public.is_ok(),"api_error":public.as_ref().err().map(|e| format!("{e:?}")),
        "api_sha256":public.as_ref().ok().map(|g| sha(g)),"api_comparison":api_comparison,
        "cli_code":cli.status.code(),"cli_binary_sha256":sha(&fs::read(binary).unwrap()),
        "cli_sha256":fs::read(&output).ok().map(|g| sha(&g)),"cli_comparison":cli_comparison,
        "comparator":"golden::compare_ordered_bytes_generator_only"
    })).unwrap()).unwrap();
    let mut failures = Vec::new();
    if !cli.status.success() {
        failures.push(format!("CLI admission: {:?}", cli.status));
    }
    for (seam, result) in [("API", api_comparison), ("CLI", cli_comparison)] {
        if let Err(error) = result {
            failures.push(format!("{seam}: {error}"));
        }
    }
    failures
}

macro_rules! pair {
    ($test:ident, $label:literal $(, $edit:expr)*) => {
        #[test]
        fn $test() {
            compare_pair(
                $label,
                &[$($edit),*],
                include_bytes!(concat!(
                    "../../../../tests/parity/arachne-alternate-extra-wall/",
                    $label,
                    "-off.3mf"
                )),
                include_bytes!(concat!(
                    "../../../../tests/parity/arachne-alternate-extra-wall/",
                    $label,
                    "-on.3mf"
                )),
            );
        }
    };
}
pair!(arachne_alternate_prism_odd_layer_count, "prism");
pair!(arachne_alternate_zero_density_inert, "density-zero");
pair!(
    arachne_alternate_half_density_int_conversion,
    "density-half"
);
pair!(arachne_alternate_one_density_active, "density-one");
pair!(
    arachne_alternate_zero_top_shell_keeps_top_walls,
    "single-top"
);
pair!(
    arachne_alternate_zero_top_shell_keeps_stepped_walls,
    "stepped-top"
);
pair!(
    arachne_alternate_zero_bottom_shell_keeps_first_layer_walls,
    "single-first"
);
pair!(arachne_alternate_zero_configured_walls, "zero-walls");
pair!(arachne_alternate_two_islands_independent, "two-islands");
pair!(arachne_alternate_effective_spiral_control, "spiral");
pair!(
    arachne_alternate_top_shell_thins_topmost_layer,
    "single-top",
    ("top_shell_layers", "1")
);
