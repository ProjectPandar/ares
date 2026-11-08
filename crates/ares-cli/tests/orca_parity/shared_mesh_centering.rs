//! bbs_3mf.cpp4867-5052 shared mesh public RED, approved SPEC5d0f9f2d9.
use crate::{
    golden,
    runner::{OrcaRunner, stages},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

const MANIFEST: &str =
    include_str!("../../../../tests/parity/shared-mesh-centering/provenance.json");

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn compare(label: &str, bytes: &[u8], plate: u32) {
    compare_scope(label, bytes, plate, Some(plate), MANIFEST);
}

pub(super) fn compare_scope(
    label: &str,
    bytes: &[u8],
    native_plate: u32,
    plate: Option<u32>,
    manifest: &str,
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("centering Native oracle not configured");
        return;
    };
    let manifest: Value = serde_json::from_str(manifest).unwrap();
    let case = &manifest["cases"][label];
    assert_eq!(sha(bytes), case["input_sha256"].as_str().unwrap());
    let project = ares_core::load_project(bytes).unwrap();
    let settings = crate::runner::application::settings(bytes).unwrap();
    for (key, value) in case["settings"].as_object().unwrap() {
        assert_eq!(&settings[key], value, "input {key}");
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("centering-c2-{label}-plate{native_plate}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), bytes).unwrap();
    fs::create_dir(root.join("native")).unwrap();
    let args = vec![
        "--slice".into(),
        native_plate.to_string(),
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
            "source":"bbs_3mf.cpp4867-5052; Model.cpp1241-1307/2600-2617/2821-2824", "plate":plate
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let result: Value =
        serde_json::from_slice(&fs::read(root.join("native/result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], 0, "Native admission {root:?}");
    let reference =
        fs::read(root.join(format!("native/plate_{}.gcode", native_plate.max(1)))).unwrap();
    if label.starts_with("facet-") {
        assert!(
            crate::option_coverage::normal_support_output::has_support_extrusion(
                std::str::from_utf8(&reference).unwrap()
            ),
            "painted current-volume support must be active: {root:?}"
        );
    }
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
            plate,
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
    let mut command = crate::runner::application::CliCommand::new(binary);
    command.arg("slice");
    if let Some(plate) = plate {
        command.args(["--plate", &plate.to_string()]);
    }
    let cli = command
        .arg("-o")
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
        "label":label,"plate":plate,"input_sha256":sha(bytes),"native_sha256":sha(&reference),
        "api_admitted":api.is_ok(),"api_error":api.as_ref().err().map(|e|format!("{e:?}")),
        "api_sha256":api.as_ref().ok().map(|g|sha(g)),"api_comparison":api_comparison,
        "cli_code":cli.status.code(),"cli_sha256":cli_bytes.as_ref().ok().map(|g|sha(g)),
        "cli_binary_sha256":sha(&fs::read(binary).unwrap()),"cli_comparison":cli_comparison,
        "strict_parity_required":true,"comparator":"unchanged golden::compare_ordered_bytes_generator_only"
    })).unwrap()).unwrap();
    fs::write(
        root.join("import-membership.json"),
        serde_json::to_vec_pretty(&json!({
            "objects":project.objects().iter().map(|object| json!({
                "id":object.id(), "volume_count":object.volumes().len()
            })).collect::<Vec<_>>(),
            "expected":case, "native_plate":native_plate, "public_plate":plate
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        project.objects().len(),
        case["object_ids"].as_array().unwrap().len()
    );
    for (object, count) in project
        .objects()
        .iter()
        .zip(case["volume_counts"].as_array().unwrap())
    {
        assert_eq!(object.volumes().len(), count.as_u64().unwrap() as usize);
    }
    assert!(api.is_ok(), "C2 public API admission RED {root:?}: {api:?}");
    assert!(
        cli.status.success(),
        "C2 CLI admission RED {root:?}: {cli:?}"
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
                    "../../../../tests/parity/shared-mesh-centering/",
                    $label,
                    ".3mf"
                )),
                1,
            );
        }
    };
}
case!(c2_implicit_shared, "implicit-shared");
case!(c2_distinct_mesh, "distinct-mesh");
case!(c2_explicit_shared, "explicit-shared");
case!(c2_unseen_key, "unseen-key");
case!(c2_minus_one_loaded, "minus-one-loaded");
case!(c2_minus_one_fresh, "minus-one-fresh");
case!(c2_negative_key, "negative-key");
case!(c2_signed_prefix, "signed-prefix");
case!(c2_zero_no_digits, "zero-no-digits");
case!(c2_reversed_build, "reversed-build");
case!(c2_different_owner_shift, "different-owner-shift");
case!(c2_fresh_rotation, "fresh-rotation");
case!(c2_shared_rotation, "shared-rotation");
case!(c2_fresh_scale, "fresh-scale");
case!(c2_shared_scale, "shared-scale");
case!(c2_same_path_repeat, "same-path-repeat");
case!(c2_nested_bfs, "nested-bfs");
case!(c2_volume_settings, "volume-settings");
case!(c2_facet_isolation, "facet-isolation");
case!(c2_facet_control, "facet-control");

const PLATES: &[u8] =
    include_bytes!("../../../../tests/parity/shared-mesh-centering/plate-local.3mf");

#[test]
fn c2_plate_one_import_scope() {
    compare("plate-local", PLATES, 1);
}

#[test]
fn c2_plate_two_import_scope() {
    compare("plate-local", PLATES, 2);
}

fn view_geometry(project: &ares_core::Project, plate: u32) -> Value {
    let selected = project.select_plate(Some(plate)).unwrap();
    json!(
        selected
            .objects()
            .iter()
            .map(|object| {
                object.volumes().iter().map(|volume| json!({
            "id":volume.id(), "source":volume.source_model_path(),
            "vertices":volume.mesh().vertices().iter().map(|p|[p.x,p.y,p.z]).collect::<Vec<_>>(),
            "triangles":volume.mesh().triangles(),
            "transform":format!("{:?}",volume.transform())
        })).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    )
}

#[test]
fn c2_same_project_selection_a_b_a() {
    let project = ares_core::load_project(PLATES).unwrap();
    let a = view_geometry(&project, 1);
    compare("plate-local", PLATES, 1);
    let b = view_geometry(&project, 2);
    compare("plate-local", PLATES, 2);
    let again = view_geometry(&project, 1);
    compare("plate-local", PLATES, 1);
    let mut builder = tempfile::Builder::new();
    builder.prefix("centering-c2-repeat-selection-");
    let root = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    fs::write(root.join("selection.json"), serde_json::to_vec_pretty(&json!({
        "sequence":[1,2,1],"a":a,"b":b,"a_again":again,
        "seam":"one loaded Project::select_plate; strict byte API/CLI per plate independently imports the same bytes"
    })).unwrap()).unwrap();
    assert_eq!(a, again, "select A,B,A must preserve the original A view");
    // Authored source widths, independently exercised by both Native plate imports.
    for (view, expected_width) in [(&a, 10.0), (&b, 16.0)] {
        let vertices = view[0][0]["vertices"].as_array().unwrap();
        let min = vertices
            .iter()
            .map(|p| p[0].as_f64().unwrap())
            .fold(f64::INFINITY, f64::min);
        let max = vertices
            .iter()
            .map(|p| p[0].as_f64().unwrap())
            .fold(f64::NEG_INFINITY, f64::max);
        assert_eq!(max - min, expected_width, "plate-local owner {root:?}");
    }
}
