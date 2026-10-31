use super::*;
use serde_json::json;

#[path = "safc_observation.rs"]
mod observation;

#[path = "safc_authority_tests.rs"]
mod authority_tests;

const ENABLE: &str = "small_area_infill_flow_compensation";
const MODEL: &str = "small_area_infill_flow_compensation_model";

fn stock_model() -> Value {
    json!([
        "0,0",
        "\n0.2,0.4444",
        "\n0.4,0.6145",
        "\n0.6,0.7059",
        "\n0.8,0.7619",
        "\n1.5,0.8571",
        "\n2,0.8889",
        "\n3,0.9231",
        "\n5,0.9520",
        "\n10,1"
    ])
}

#[test]
fn safc_coupled_models_preserve_whole_output_and_segment_consumers() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = parity::artifacts::root_from_env().unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let fixed = [
        ("layer_height", "0.2"),
        ("initial_layer_print_height", "0.2"),
        ("wall_generator", "classic"),
        ("wall_loops", "2"),
        ("sparse_infill_density", "40%"),
        ("sparse_infill_pattern", "rectilinear"),
        ("bottom_shell_layers", "3"),
        ("top_shell_layers", "3"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("skirt_loops", "0"),
        ("enable_prime_tower", "0"),
        ("enable_arc_fitting", "0"),
        ("spiral_mode", "0"),
    ];
    for (key, value) in fixed {
        process.insert(key.into(), json!(value));
    }
    let custom = json!(["0,0", "5,0.5", "20,1"]);
    let cases = [
        ("off-stock", "0", stock_model(), "rectilinear"),
        ("on-stock", "1", stock_model(), "rectilinear"),
        ("on-custom", "1", custom.clone(), "rectilinear"),
        ("off-custom", "0", custom.clone(), "rectilinear"),
        ("on-empty", "1", json!([]), "rectilinear"),
        ("on-concentric", "1", custom.clone(), "concentric"),
        ("off-concentric", "0", custom, "concentric"),
    ];
    let mut proofs = Vec::new();
    let mut observed_native = Vec::new();
    let mut observed_ares = Vec::new();
    let mut failures = Vec::new();
    let mut exported_cases = Vec::new();
    for (label, enabled, model, pattern) in cases {
        process.insert(ENABLE.into(), json!(enabled));
        process.insert(MODEL.into(), model.clone());
        for key in [
            "bottom_surface_pattern",
            "top_surface_pattern",
            "internal_solid_infill_pattern",
        ] {
            process.insert(key.into(), json!(pattern));
        }
        let label = format!("safc/{label}");
        let exported = runner
            .export_case(
                &runner::CaseInputs {
                    label: &label,
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        runner::application::verify_value(&exported.project, ENABLE, &json!(enabled)).unwrap();
        runner::application::verify_value(&exported.project, MODEL, &model).unwrap();
        for (key, value) in fixed {
            runner::application::verify_value(&exported.project, key, &json!(value)).unwrap();
        }
        for key in [
            "bottom_surface_pattern",
            "top_surface_pattern",
            "internal_solid_infill_pattern",
        ] {
            runner::application::verify_value(&exported.project, key, &json!(pattern)).unwrap();
        }
        exported_cases.push((label, enabled, model, pattern, exported));
    }
    let concentric_cases = exported_cases
        .iter()
        .skip(5)
        .map(|(_, _, _, _, exported)| exported)
        .collect::<Vec<_>>();
    let Some(initialized) = runner::safc_reference::initialized_pair(&concentric_cases, &root)
    else {
        return;
    };
    let caller_producer = runner::safc_reference::producer(&runner);
    for (index, (label, enabled, model, pattern, exported)) in
        exported_cases.into_iter().enumerate()
    {
        let (sliced, native_producer) = if index < 5 {
            (runner.slice_case(&exported).unwrap(), &caller_producer)
        } else {
            (
                runner::ParityCase {
                    label: label.clone(),
                    project: exported.project,
                    reference: std::fs::read(&initialized.paths[index - 5]).unwrap(),
                },
                &initialized.producer,
            )
        };
        let native = observation::observe(&sliced.reference);
        observation::require_roles(&native);
        let outcome = parity::compare_case(&sliced);
        let ares = outcome
            .artifacts
            .as_ref()
            .and_then(|path| std::fs::read(path.join("ares.gcode")).ok())
            .map(|bytes| observation::observe(&bytes));
        if let Some(ares) = &ares {
            observation::require_roles(ares);
        }
        proofs.push(json!({"label":label,"enabled":enabled,"model":model,"pattern":pattern,
            "native_producer":native_producer,
            "native_role_segments":observation::role_counts(&native),
            "ares_role_segments":ares.as_ref().map(|s| observation::role_counts(s)),
            "full_output_status":outcome.status,"detail":outcome.detail,"artifacts":outcome.artifacts}));
        parity::artifacts::overwrite(
            &root.join("safc-cases.json"),
            &serde_json::to_vec_pretty(&proofs).unwrap(),
        )
        .unwrap();
        if outcome.status != "PASS" {
            failures.push(format!("{label}: {}", outcome.detail));
        }
        observed_native.push(native);
        observed_ares.push(ares);
    }
    let pairs = [
        (0, 1, true),
        (0, 2, true),
        (0, 3, false),
        (0, 4, false),
        (6, 5, false),
    ];
    let mut consumers = Vec::new();
    for (off, on, active) in pairs {
        let native_reduced =
            observation::verify_pair(&observed_native[off], &observed_native[on], active);
        let ares_reduced = match (&observed_ares[off], &observed_ares[on]) {
            (Some(off), Some(on)) => Some(observation::verify_pair(off, on, active)),
            _ => None,
        };
        consumers.push(
            json!({"off":proofs[off]["label"],"on":proofs[on]["label"],"active":active,
            "native_reduced_segments":native_reduced,"ares_reduced_segments":ares_reduced}),
        );
        parity::artifacts::overwrite(
            &root.join("safc-consumers.json"),
            &serde_json::to_vec_pretty(&consumers).unwrap(),
        )
        .unwrap();
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
