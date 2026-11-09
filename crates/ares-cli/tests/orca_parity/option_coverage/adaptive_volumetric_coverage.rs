use super::*;
use serde_json::json;

#[path = "adaptive_volumetric_observation.rs"]
mod observation;
#[path = "adaptive_volumetric_rejection.rs"]
mod rejection;

const ENABLE: &str = "filament_adaptive_volumetric_speed";
const MODEL: &str = "volumetric_speed_coefficients";

#[test]
fn adaptive_volumetric_coupled_models_preserve_whole_output_and_segment_consumers() {
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
    for filament in &mut filaments {
        for (key, value) in [
            ("filament_max_volumetric_speed", "12"),
            ("slow_down_for_layer_cooling", "0"),
        ] {
            filament.insert(key.into(), json!([value]));
        }
    }
    let speed_values = [
        ("outer_wall_speed", json!(["100"])),
        ("inner_wall_speed", json!("100")),
        ("sparse_infill_speed", json!("100")),
        ("internal_solid_infill_speed", json!("100")),
        ("top_surface_speed", json!("100")),
    ];
    let cases = [
        ("off-empty", "0", ""),
        ("on-empty", "1", ""),
        ("off-low", "0", "0 0 0 0 0 1"),
        ("on-low", "1", "0 0 0 0 0 1"),
        ("on-high", "1", "0 0 0 0 0 100"),
        ("on-zero", "1", "0 0 0 0 0 0"),
        ("on-short", "1", "0 0 0 0 1"),
        ("on-polynomial", "1", "1 1 1 1 1 0.5"),
        ("explicit-off-low", "0", "0 0 0 0 0 1"),
        ("explicit-on-low", "1", "0 0 0 0 0 1"),
    ];
    let mut rejection_project = Vec::new();
    let mut proofs = Vec::new();
    let mut observed_native = Vec::new();
    let mut observed_ares = Vec::new();
    let mut failures = Vec::new();
    for (label, enabled, coefficients) in cases {
        for (key, value) in &speed_values {
            process.insert((*key).into(), value.clone());
        }
        let model = json!([coefficients]);
        for filament in &mut filaments {
            filament.insert(ENABLE.into(), json!([enabled]));
            filament.insert(MODEL.into(), model.clone());
        }
        let label = format!("adaptive_volumetric/{label}");
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
        runner::application::verify_value(&exported.project, ENABLE, &json!([enabled])).unwrap();
        runner::application::verify_value(&exported.project, MODEL, &model).unwrap();
        for (key, value) in fixed {
            runner::application::verify_value(&exported.project, key, &json!(value)).unwrap();
        }
        // Config.cpp:1573–1585 emits coFloats options as JSON arrays.
        for (key, value) in &speed_values {
            let expected = if value.is_array() {
                value.clone()
            } else {
                json!([value])
            };
            runner::application::verify_value(&exported.project, key, &expected).unwrap();
        }
        rejection_project.clone_from(&exported.project);
        let sliced = runner.slice_case(&exported).unwrap();
        let native = observation::observe(&sliced.reference);

        let outcome = parity::compare_case(&sliced);
        let ares = outcome
            .artifacts
            .as_ref()
            .and_then(|path| std::fs::read(path.join("ares.gcode")).ok())
            .map(|bytes| observation::observe(&bytes));

        proofs.push(json!({"label":label,"enabled":enabled,"model":model,
            "native_role_segments":observation::role_counts(&native),
            "ares_role_segments":ares.as_ref().map(|s| observation::role_counts(s)),
            "full_output_status":outcome.status,"detail":outcome.detail,"artifacts":outcome.artifacts}));
        parity::artifacts::overwrite(
            &root.join("adaptive_volumetric-cases.json"),
            &serde_json::to_vec_pretty(&proofs).unwrap(),
        )
        .unwrap();
        if outcome.status != "PASS" {
            failures.push(format!("{label}: {}", outcome.detail));
        }
        observed_native.push(native);
        observed_ares.push(ares);
    }
    let speed_keys = speed_values.map(|(key, _)| key);
    rejection::verify(&runner, &root, &rejection_project, &speed_keys);
    let pairs = [
        (0, 1, false),
        (0, 2, false),
        (2, 3, false),
        (0, 4, false),
        (0, 5, false),
        (0, 6, false),
        (0, 7, false),
        (8, 9, false),
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
            &root.join("adaptive_volumetric-consumers.json"),
            &serde_json::to_vec_pretty(&consumers).unwrap(),
        )
        .unwrap();
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
