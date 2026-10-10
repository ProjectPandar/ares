//! PrintConfig.cpp:779-787: preserve the existing 50/400 interior requests.
//! A complete stream alone cannot certify the printable-height consumer scope.

use super::super::{OptionOutcome, domains, inject_case};
use crate::runner::{self, CaseInputs, OrcaRunner};
use serde_json::json;

#[path = "printable_height/admission.rs"]
pub(in crate::option_coverage) mod admission;
#[path = "printable_height/loaded.rs"]
mod loaded;
#[path = "printable_height/scene.rs"]
mod scene;

// PrintConfig.cpp:779-787 accepts zero; BuildVolume.cpp:383-384 disables
// only its scene ceiling. Print.cpp:1378-1396 still validates generated layers.
#[test]
fn orca_parity_printable_height_zero_endpoint_validated_cli() {
    if crate::native_environment::binary("ARES_ORCA_NATIVE_BIN").is_none()
        || crate::native_environment::value("ARES_ORCA_RESOURCES_DIR").is_none()
    {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = crate::presets::VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("printable_height".to_owned(), json!("0"));
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "printable-height/zero-endpoint",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    runner::application::verify_value(&exported.project, "printable_height", &json!("0")).unwrap();
    runner::application::verify_value(&exported.project, "print_sequence", &json!("by layer"))
        .unwrap();
    let loaded = ares_core::load_project(&exported.project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    let object = &loaded.objects()[0];
    assert_eq!(object.instances().len(), 1);
    assert_eq!(object.volumes().len(), 1);
    let volume = &object.volumes()[0];
    let transform = object.instances()[0].transform().then(volume.transform());
    let points = volume
        .mesh()
        .vertices()
        .iter()
        .map(|point| transform.transform_point(*point))
        .collect::<Vec<_>>();
    let min_z = points.iter().map(|p| p.z).fold(f64::INFINITY, f64::min);
    let max_z = points.iter().map(|p| p.z).fold(f64::NEG_INFINITY, f64::max);
    assert_eq!([min_z, max_z], [0., 10.]);
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("printable-height-zero-")
        .tempdir_in(&root)
        .unwrap()
        .keep();
    let input = directory.join("input.3mf");
    crate::artifacts::write(&input, &exported.project).unwrap();
    let native = runner
        .early_cli_reference(&input, runner::EarlyExit::Validation)
        .unwrap();
    for (name, bytes) in [
        ("native.stdout", &native.stdout),
        ("native.stderr", &native.stderr),
        ("native.result.json", &native.result_json),
    ] {
        crate::artifacts::write(&directory.join(name), bytes).unwrap();
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let actual = runtime.block_on(ares_core::slice_project_plate_validated(
        &exported.project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    let public_matches_native = matches!(
        &actual,
        Err(ares_core::SliceError::Validate { message, .. })
            if message.as_bytes() == native.stderr.strip_suffix(b"\n").unwrap()
    );
    let public_outcome = match actual {
        Ok(gcode) => {
            crate::artifacts::write(&directory.join("ares.validated.gcode"), &gcode).unwrap();
            json!({"accepted_bytes":gcode.len()})
        }
        Err(error) => json!({"error":format!("{error:?}")}),
    };
    let output = directory.join("plate_1.gcode");
    let cli = std::process::Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&output)
        .arg(&input)
        .output()
        .unwrap();
    for (name, bytes) in [("ares.stdout", &cli.stdout), ("ares.stderr", &cli.stderr)] {
        crate::artifacts::write(&directory.join(name), bytes).unwrap();
    }
    let result = std::fs::read(directory.join("result.json")).unwrap();
    crate::artifacts::overwrite(
        &root.join("printable-height-zero-endpoint.json"),
        &serde_json::to_vec_pretty(&json!({
            "requested":"0","min_z":min_z,"max_z":max_z,"input":input,"Native_receipt":native.directory.join("receipt.json"),
            "Native_result":serde_json::from_slice::<serde_json::Value>(&native.result_json).unwrap(),
            "Ares_public":public_outcome,"Ares_CLI_code":cli.status.code(),
            "Ares_CLI_result":serde_json::from_slice::<serde_json::Value>(&result).unwrap(),
            "Ares_output_exists":output.exists(),"artifacts":directory
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        public_matches_native,
        "validated API must match the Native Validate diagnostic"
    );
    assert_eq!(cli.status.code(), Some(if cfg!(unix) { 205 } else { -51 }));
    assert_eq!(cli.stdout, native.stdout);
    assert_eq!(cli.stderr, native.stderr);
    assert_eq!(
        crate::native_wall_clock::result_comparison_bytes(&result),
        crate::native_wall_clock::result_comparison_bytes(&native.result_json)
    );
    assert!(!output.exists());
}

#[test]
fn orca_parity_printable_height_canonical_complete_output_red() {
    if crate::native_environment::binary("ARES_ORCA_NATIVE_BIN").is_none()
        || crate::native_environment::value("ARES_ORCA_RESOURCES_DIR").is_none()
    {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "printable_height")
        .unwrap();
    let profiles = crate::presets::VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let mut proofs = Vec::new();
    let mut observed = plan.clone();
    observed.cases.clear();
    for wire in ["50", "400", "214700"] {
        let case = domains::OptionCase {
            label: format!("canonical-{wire}"),
            value: Some(json!(wire)),
        };
        let mut case_machine = machine.clone();
        let mut case_process = process.clone();
        let mut case_filaments = filaments.clone();
        inject_case(
            &plan,
            &case,
            &mut case_machine,
            &mut case_process,
            &mut case_filaments,
        );
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("printable-height/{}", case.label),
                    machine: &case_machine,
                    process: &case_process,
                    filaments: &case_filaments,
                },
                &crate::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        runner::application::verify_value(&exported.project, "printable_height", &json!(wire))
            .unwrap();
        runner::application::verify_value(&exported.project, "print_sequence", &json!("by layer"))
            .unwrap();
        runner::application::verify_value(&exported.project, "nozzle_diameter", &json!(["0.4"]))
            .unwrap();
        let loaded = ares_core::load_project(&exported.project).unwrap();
        assert_eq!(loaded.objects().len(), 1);
        let object = &loaded.objects()[0];
        assert_eq!(object.instances().len(), 1);
        assert_eq!(object.volumes().len(), 1);
        let volume = &object.volumes()[0];
        let transform = object.instances()[0].transform().then(volume.transform());
        let vertices = volume
            .mesh()
            .vertices()
            .iter()
            .map(|point| transform.transform_point(*point))
            .collect::<Vec<_>>();
        let min_z = vertices.iter().map(|p| p.z).fold(f64::INFINITY, f64::min);
        let max_z = vertices
            .iter()
            .map(|p| p.z)
            .fold(f64::NEG_INFINITY, f64::max);
        assert_eq!([min_z, max_z], [0., 10.]);
        let sliced = runner.slice_case(&exported).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let validated = runtime
            .block_on(ares_core::slice_project_plate_validated(
                &exported.project,
                ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
                None,
                &ares_core::ConfigOverrides::default(),
                ares_core::SlicerContext::new(0),
            ))
            .unwrap();
        crate::golden::compare_ordered_bytes_generator_only(&sliced.reference, &validated).unwrap();
        let outcome = crate::compare_case(&sliced);
        assert_eq!(outcome.status, "PASS", "{wire}: {}", outcome.detail);
        proofs.push(json!({"requested":wire,"min_z":min_z,"max_z":max_z,"status":outcome.status,"artifacts":outcome.artifacts}));
        observed.cases.push(case);
    }
    let classification = OptionOutcome::executed(&observed, 3, Vec::new(), 0, None);
    let root = crate::artifacts::root_from_env().unwrap();
    crate::artifacts::overwrite(
        &root.join("printable-height-canonical-red.json"),
        &serde_json::to_vec_pretty(&json!({
            "source":"PrintConfig.cpp:779-787",
            "scope":"one cube10 at Z0..10, by layer, one0.4 nozzle; consumer witnesses still required",
            "proofs":proofs,"classification":classification.status,"detail":classification.detail
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(classification.status, "PASS", "{}", classification.detail);
}
