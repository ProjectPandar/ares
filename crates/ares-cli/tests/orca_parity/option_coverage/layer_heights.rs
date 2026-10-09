//! Print.cpp:1639-1694 height admission, isolated from nonbridge width checks.
use super::*;
use crate::runner::CaseInputs;
use serde_json::json;

#[test]
fn orca_parity_normal_layer_height_includes_nozzle_equality() {
    compare_height("layer_height");
}

#[test]
fn orca_parity_initial_layer_height_includes_nozzle_equality() {
    compare_height("initial_layer_print_height");
}

fn compare_height(key: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == key)
        .unwrap();
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.value.as_ref().unwrap().as_str().unwrap())
            .collect::<Vec<_>>(),
        ["0.05", "0.2", "0.36", "0.39999", "0.4"]
    );
    let mut proofs = Vec::new();
    for case in &plan.cases {
        let wire = case.value.as_ref().unwrap().as_str().unwrap();
        let exported = export_height(&runner, key, wire).unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        let schedule = std::str::from_utf8(&sliced.reference)
            .unwrap()
            .lines()
            .filter(|line| line.starts_with(";Z:") || line.starts_with(";HEIGHT:"))
            .collect::<Vec<_>>();
        assert!(schedule.len() > 2, "{key}={wire}: missing layer schedule");
        let height = wire.parse::<f64>().unwrap();
        let heights = schedule
            .iter()
            .filter_map(|line| line.strip_prefix(";HEIGHT:"))
            .map(|value| value.parse::<f64>().unwrap())
            .collect::<Vec<_>>();
        let observed = if key == "initial_layer_print_height" {
            heights[0]
        } else {
            heights[1]
        };
        // GCode.cpp:4626 emits %g; each finite request fits its precision.
        assert_eq!(observed, height, "{key}={wire}: {heights:?}");
        let outcome = parity::compare_case(&sliced);
        proofs.push(json!({"requested":wire,"schedule":schedule,
            "full_output_status":outcome.status,"detail":outcome.detail}));
        let root = parity::artifacts::root_from_env().unwrap();
        parity::artifacts::overwrite(
            &root.join(format!("{key}-bounded.json")),
            &serde_json::to_vec_pretty(&proofs).unwrap(),
        )
        .unwrap();
        assert_eq!(outcome.status, "PASS", "{key}={wire}: {}", outcome.detail);
    }
    let outcome = OptionOutcome::executed(&plan, proofs.len(), Vec::new(), 0, None);
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &root.join(format!("{key}-bounded.json")),
        &serde_json::to_vec_pretty(&json!({"domain":height_context::metadata(),
            "cases":proofs,"classification":outcome.status,"detail":outcome.detail}))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome.status, "PASS", "{key}: {}", outcome.detail);
}

fn export_height(
    runner: &OrcaRunner,
    key: &str,
    wire: &str,
) -> Result<runner::ExportedCase, runner::stages::StageError> {
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    machine.insert("nozzle_diameter".into(), json!(["0.4"]));
    machine.insert("use_relative_e_distances".into(), json!("1"));
    for field in ["layer_height", "initial_layer_print_height"] {
        process.insert(field.into(), json!("0.2"));
    }
    for field in [
        "line_width",
        "initial_layer_line_width",
        "inner_wall_line_width",
        "outer_wall_line_width",
        "sparse_infill_line_width",
        "internal_solid_infill_line_width",
        "top_surface_line_width",
        "skin_infill_line_width",
        "skeleton_infill_line_width",
    ] {
        process.insert(field.into(), json!("0.6"));
    }
    for (field, value) in [
        ("bridge_line_width", "0.4"),
        ("thick_bridges", "1"),
        ("thick_internal_bridges", "1"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
    ] {
        process.insert(field.into(), json!(value));
    }
    process.insert(key.into(), json!(wire));
    let exported = runner.export_case(
        &CaseInputs {
            label: &format!("bounded-height/{key}/{wire}"),
            machine: &machine,
            process: &process,
            filaments: &filaments,
        },
        &parity::smoke_overrides(),
        &runner::repo_root().join("tests/parity/cube10.stl"),
    )?;
    for field in [
        "layer_height",
        "initial_layer_print_height",
        "line_width",
        "initial_layer_line_width",
        "inner_wall_line_width",
        "outer_wall_line_width",
        "sparse_infill_line_width",
        "internal_solid_infill_line_width",
        "top_surface_line_width",
        "skin_infill_line_width",
        "skeleton_infill_line_width",
        "bridge_line_width",
        "thick_bridges",
        "thick_internal_bridges",
        "enable_support",
        "raft_layers",
    ] {
        runner::application::verify_value(&exported.project, field, &process[field]).unwrap();
    }
    for field in ["nozzle_diameter", "use_relative_e_distances"] {
        runner::application::verify_value(&exported.project, field, &machine[field]).unwrap();
    }
    Ok(exported)
}

#[test]
fn original_orca_and_ares_reject_layer_height_boundaries() {
    use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
    use runner::stages::{FailureKind, Stage};
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let mut proofs = Vec::new();
    for key in ["layer_height", "initial_layer_print_height"] {
        let baseline = export_height(&runner, key, "0.2").unwrap();
        for (wire, stage, code, diagnostic) in [
            (
                "0",
                Stage::Export,
                -18_i32,
                format!("{key}: invalid value 0.000000"),
            ),
            (
                "0.40001",
                Stage::Slice,
                -51,
                "Layer height cannot exceed nozzle diameter.".into(),
            ),
        ] {
            let error = export_height(&runner, key, wire)
                .and_then(|exported| runner.slice_case(&exported))
                .err()
                .expect("out-of-bound height must fail");
            assert_eq!(error.stage, stage, "{key}={wire}: {error}");
            assert_eq!(error.kind, FailureKind::Process);
            let captured = error.captured.as_ref().unwrap();
            let expected_code = if cfg!(unix) {
                code.rem_euclid(256)
            } else {
                code
            };
            assert_eq!(captured.code, Some(expected_code), "{key}={wire}: {error}");
            let text = format!(
                "{}\n{}",
                String::from_utf8_lossy(&captured.stdout),
                String::from_utf8_lossy(&captured.stderr)
            );
            assert!(text.contains(&diagnostic), "{key}={wire}: {text}");
            assert!(
                error
                    .evidence
                    .as_ref()
                    .unwrap()
                    .join("status.json")
                    .is_file()
            );
            let overrides = ConfigOverrides::from_json(
                serde_json::to_string(&json!({key:wire}))
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap();
            let validation = ares_core::validate_project_plate(&baseline.project, None, &overrides);
            let sliced = runtime.block_on(ares_core::slice_project_plate_validated(
                &baseline.project,
                GenerationMetadata::default(),
                None,
                &overrides,
                SlicerContext::new(0),
            ));
            assert_public_rejection(validation, key, wire, &diagnostic);
            assert_public_rejection(sliced.map(|_| ()), key, wire, &diagnostic);
            proofs.push(
                json!({"key":key,"requested":wire,"native_stage":format!("{stage:?}"),
                "native_code":captured.code,"diagnostic":diagnostic,"evidence":error.evidence,
                "public_validation":"same rejection","validated_slice":"same rejection"}),
            );
        }
    }
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &root.join("layer-height-rejections.json"),
        &serde_json::to_vec_pretty(&proofs).unwrap(),
    )
    .unwrap();
}

fn assert_public_rejection(
    result: Result<(), ares_core::SliceError>,
    key: &str,
    wire: &str,
    diagnostic: &str,
) {
    use ares_core::SliceError;
    match result {
        Err(SliceError::InvalidInput(message)) if wire == "0" => assert_eq!(message, diagnostic),
        Err(SliceError::Validate { message, .. }) if wire == "0.40001" => {
            assert_eq!(message, diagnostic)
        }
        other => panic!("{key}={wire}: public rejection disagrees: {other:?}"),
    }
}
