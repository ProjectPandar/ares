//! `Print::process` -> `ConflictChecker`: an active fake Type2 tower is
//! included in native path-conflict rejection, not just the preview.
use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::json;

#[test]
fn smooth_type2_tower_path_conflict() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("wipe_tower_type".into(), json!("type2"));
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("enable_prime_tower", json!("1")),
        ("timelapse_type", json!("1")),
        ("wipe_tower_wall_type", json!("rectangle")),
        ("prime_tower_width", json!("40")),
        ("wipe_tower_x", json!(["105"])),
        ("wipe_tower_y", json!(["105"])),
    ] {
        process.insert(key.into(), value);
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "smooth-type2/path-conflict",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    for key in [
        "enable_prime_tower",
        "wipe_tower_x",
        "wipe_tower_y",
        "wipe_tower_wall_type",
    ] {
        runner::application::verify_value(&exported.project, key, &process[key]).unwrap();
    }
    let native = runner
        .slice_case(&exported)
        .err()
        .expect("native must reject the path conflict");
    let evidence = tempfile::Builder::new()
        .prefix("tower-conflict-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    std::fs::write(evidence.join("input.3mf"), &exported.project).unwrap();
    std::fs::write(evidence.join("native-error.txt"), native.to_string()).unwrap();
    let captured = native.captured.as_ref().unwrap();
    std::fs::write(evidence.join("native-stdout.txt"), &captured.stdout).unwrap();
    std::fs::write(evidence.join("native-stderr.txt"), &captured.stderr).unwrap();
    // Utils.hpp: CLI_GCODE_PATH_CONFLICTS; Unix retains its low byte.
    assert_eq!(captured.code, Some(if cfg!(unix) { 155 } else { -101 }));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let actual = runtime.block_on(crate::runner::application::slice_project(
        &exported.project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    ));
    match actual {
        Ok(bytes) => {
            std::fs::write(evidence.join("ares.gcode"), bytes).unwrap();
            panic!("Ares accepted a native-rejected tower path conflict; evidence {evidence:?}");
        }
        Err(error) => {
            std::fs::write(evidence.join("ares-error.txt"), error.to_string()).unwrap();
            assert!(error.to_string().contains("path conflicts"), "{error}");
        }
    }
}
