//! PrintConfig.cpp:9004-9055,9634-9683; PrintApply.cpp:1169.
use super::*;
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::PathBuf,
};

#[path = "print_extruder_identity/cases.rs"]
mod cases;
use cases::{CASES, Case};

fn authored_identity(project: &[u8], ids: &Value, variants: &Value) -> Vec<u8> {
    let mut before = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..before.len() {
        let mut file = before.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["print_extruder_id"] = ids.clone();
            settings["print_extruder_variant"] = variants.clone();
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    let result = writer.finish().unwrap().into_inner();
    let mut original = crate::runner::application::settings(project).unwrap();
    let mut actual = crate::runner::application::settings(&result).unwrap();
    assert_eq!(actual.remove("print_extruder_id"), Some(ids.clone()));
    assert_eq!(
        actual.remove("print_extruder_variant"),
        Some(variants.clone())
    );
    original.remove("print_extruder_id");
    original.remove("print_extruder_variant");
    assert_eq!(
        actual, original,
        "all other settings, including master and physical maps, are frozen"
    );
    let mut after = zip::ZipArchive::new(Cursor::new(&result)).unwrap();
    assert_eq!(before.len(), after.len());
    for index in 0..before.len() {
        let mut a = before.by_index(index).unwrap();
        if a.name() == "Metadata/project_settings.config" {
            continue;
        }
        let mut b = after.by_name(a.name()).unwrap();
        let mut aa = Vec::new();
        let mut bb = Vec::new();
        a.read_to_end(&mut aa).unwrap();
        b.read_to_end(&mut bb).unwrap();
        assert_eq!(aa, bb, "{}", a.name());
    }
    result
}

async fn compare_identity(runner: &OrcaRunner, case: &Case) -> PathBuf {
    let label = format!("print-process-identity/{}", case.label);
    let (vendor, printer) = ("BBL", case.printer);
    let (ids, variants) = (json!(case.ids), json!(case.variants));
    let (expected_ids, expected_variants) = (case.expected_ids, case.expected_variants);
    let profiles = VendorProfiles::load(&profiles_root(), vendor).unwrap();
    let selected = parity::select_printer(&profiles, vendor, printer).unwrap();
    let machine = profiles.machine(&selected.printer).unwrap();
    let mut process = profiles.process(&selected.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selected
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &crate::runner::CaseInputs {
                label: &label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &crate::runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let directory = tempfile::Builder::new()
        .prefix("print-process-identity-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(directory.join("original.3mf"), &exported.project).unwrap();
    let project = authored_identity(&exported.project, &ids, &variants);
    let settings = crate::runner::application::settings(&project).unwrap();
    fs::write(
        directory.join("requested-settings.json"),
        serde_json::to_vec_pretty(&settings).unwrap(),
    )
    .unwrap();
    let types = if printer == "Bambu Lab H2D 0.4 nozzle" {
        json!(["Direct Drive", "Direct Drive"])
    } else {
        json!(["Direct Drive"])
    };
    assert_eq!(settings["extruder_type"], types);
    assert_eq!(settings["nozzle_volume_type"], json!(["Standard"]));
    let input = directory.join("input.3mf");
    fs::write(&input, &project).unwrap();
    let native_dir = directory.join("native");
    fs::create_dir(&native_dir).unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        native_dir.to_str().unwrap().into(),
        input.to_str().unwrap().into(),
    ];
    fs::write(
        directory.join("native.args.json"),
        serde_json::to_vec_pretty(&args).unwrap(),
    )
    .unwrap();
    crate::runner::stages::run(
        runner.bin(),
        args,
        crate::runner::stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let native = fs::read_to_string(native_dir.join("plate_1.gcode")).unwrap();
    fs::write(directory.join("native.gcode"), &native).unwrap();
    // The appended full Print config is an independent consumption witness,
    // not a raw input roundtrip (GCode.cpp:5591-5644 append_full_config).
    assert!(
        native.contains(&format!("\n; print_extruder_id = {expected_ids}\n")),
        "Native must materialize physical request order before Ares comparison: {directory:?}"
    );
    assert!(native.contains(&format!(
        "\n; print_extruder_variant = {expected_variants}\n"
    )));
    let result = ares_core::slice_project_plate(
        &project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await;
    fs::write(directory.join("ares.result.txt"), format!("{result:?}")).unwrap();
    let ares = result.unwrap();
    fs::write(directory.join("ares.gcode"), &ares).unwrap();
    assert_eq!(fs::read(&input).unwrap(), project);
    let ares = String::from_utf8(ares).unwrap();
    assert_eq!(
        crate::cli_retained_input::normalize_generator(&ares, "Ares"),
        crate::cli_retained_input::normalize_generator(&native, "OrcaSlicer"),
        "complete process identity stream: {directory:?}"
    );
    directory
}

async fn compare_case(label: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = CASES.iter().find(|case| case.label == label).unwrap();
    compare_identity(&runner, case).await;
}

pub(super) fn generate(key: &str) -> Option<(Vec<super::domains::OptionCase>, Value)> {
    if key != "print_extruder_id" {
        return None;
    }
    Some((
        CASES
            .iter()
            .map(|case| super::domains::OptionCase {
                label: case.label.into(),
                value: Some(json!(case.ids)),
            })
            .collect(),
        json!({
            "domain":"six Native-admitted print identity/materialization contexts",
            "source":"PrintConfig.cpp:5405,9004-9055,9634-9683; PrintApply.cpp:1169",
            "spec":"docs/architecture/2026-10-14-print-extruder-id-matrix-spec.md",
            "cases":CASES.iter().map(|case| json!({
                "label":case.label,"printer":case.printer,"ids":case.ids,
                "variants":case.variants,"expected_ids":case.expected_ids,
                "expected_variants":case.expected_variants
            })).collect::<Vec<_>>(),
            "comparison":"existing complete Native/API stream and materialized config assertions",
            "deferred":["arbitrary ID/vector shapes","other printer/variant contexts","master/physical identity","production changes"]
        }),
    ))
}

pub(super) fn execute_plan(
    runner: &OrcaRunner,
    plan: &super::domains::OptionPlan,
) -> super::OptionOutcome {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &root.join("print-extruder-id-domain.json"),
        &serde_json::to_vec_pretty(plan.tool_metadata.as_ref().unwrap()).unwrap(),
    )
    .unwrap();
    for request in &plan.cases {
        let case = CASES
            .iter()
            .find(|case| case.label == request.label)
            .unwrap();
        assert_eq!(request.value, Some(json!(case.ids)));
        let directory = runtime.block_on(compare_identity(runner, case));
        // The sweep and its public regression can publish this same case concurrently.
        let mut receipt = tempfile::NamedTempFile::new_in(&root).unwrap();
        serde_json::to_writer_pretty(
            receipt.as_file_mut(),
            &json!({
                "key":plan.key,"label":case.label,"directory":directory,
                "comparison":"PASS","kind":"strict whole G-code",
                "requested_ids":case.ids,"requested_variants":case.variants,
                "expected_ids":case.expected_ids,"expected_variants":case.expected_variants
            }),
        )
        .unwrap();
        receipt
            .persist(root.join(format!("print-extruder-id-{}-comparison.json", case.label)))
            .unwrap();
    }
    super::OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, None)
}

#[tokio::test]
async fn print_process_identity_materializes_reversed_complete_map_without_master_changes() {
    compare_case("reversed").await;
}

#[tokio::test]
async fn print_process_identity_complete_physical_order_is_preserved() {
    compare_case("ordered").await;
}

#[tokio::test]
async fn print_process_identity_default_singleton_exports_complete_output() {
    compare_case("default").await;
}

#[tokio::test]
async fn print_process_identity_repeated_ids_select_matching_variants() {
    compare_case("repeated").await;
}

#[tokio::test]
async fn print_process_identity_short_ids_use_generated_group_identity() {
    compare_case("short").await;
}

#[tokio::test]
async fn print_process_identity_unmatched_variant_uses_native_materialized_view() {
    compare_case("unmatched").await;
}
