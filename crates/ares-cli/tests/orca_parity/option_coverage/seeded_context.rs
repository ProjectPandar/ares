use super::*;
use crate::runner::CaseInputs;

#[test]
fn seeded_context_replays_active_fuzzy_output_and_isolates_jobs() {
    verify_context("cube10.stl", "0.3", "classic");
}

#[test]
fn seeded_context_matches_short_fuzzy_contour_padding() {
    verify_context("cube08.stl", "2", "classic");
}

#[test]
fn seeded_context_matches_coherent_random_spacing() {
    for noise in ["perlin", "billow", "ridgedmulti", "voronoi"] {
        verify_context("cube10.stl", "0.3", noise);
    }
}

fn verify_context(model: &str, point_distance: &str, noise: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("fuzzy_skin", "external"),
        ("fuzzy_skin_noise_type", noise),
        ("fuzzy_skin_thickness", "0.2"),
        ("fuzzy_skin_point_distance", point_distance),
        ("top_shell_layers", "0"),
        ("bottom_shell_layers", "0"),
        ("sparse_infill_density", "0%"),
    ] {
        process.insert(key.to_owned(), Value::String(value.to_owned()));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &format!("seeded-context/{noise}/{model}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &runner::repo_root().join("tests/parity").join(model),
        )
        .unwrap();
    let settings = runner::application::settings(&exported.project).unwrap();
    assert_eq!(settings["fuzzy_skin"], "external");
    assert_eq!(settings["fuzzy_skin_noise_type"], noise);
    assert_eq!(settings["wall_generator"], "classic");
    assert_eq!(settings["fuzzy_skin_point_distance"], point_distance);
    let artifacts = tempfile::Builder::new()
        .prefix("seeded-context-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    std::fs::write(artifacts.join("input.3mf"), &exported.project).unwrap();
    let output = |seed| {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime
            .block_on(ares_core::slice_project(
                &exported.project,
                ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
                ares_core::SlicerContext::new(seed),
            ))
            .unwrap()
    };
    let zero = output(0);
    std::fs::write(artifacts.join("seed-0.gcode"), &zero).unwrap();
    if std::env::var("ARES_PARITY_SEEDED_REFERENCE").as_deref() == Ok("1") {
        let seed = std::env::var("ORCA_REPLAY_SEED")
            .unwrap()
            .parse::<u32>()
            .unwrap();
        let reference = runner.slice_case(&exported).unwrap();
        let actual = output(seed);
        std::fs::write(artifacts.join("reference.gcode"), &reference.reference).unwrap();
        std::fs::write(artifacts.join("reference-ares.gcode"), &actual).unwrap();
        std::fs::write(
            artifacts.join("reference.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "seed": seed,
                "cpu": std::env::var("ORCA_REPLAY_CPU").unwrap(),
                "binary": std::env::var("ORCA_REPLAY_BIN").unwrap(),
                "evidence": crate::artifacts::EVIDENCE,
                "scope": "classic fuzzy walls without fill",
                "noise": noise,
            }))
            .unwrap(),
        )
        .unwrap();
        crate::golden::compare_ordered_bytes_generator_only(&reference.reference, &actual).unwrap();
    }
    let repeat = output(0);
    std::fs::write(artifacts.join("seed-0-repeat.gcode"), &repeat).unwrap();
    assert_eq!(repeat, zero);
    let one = output(1);
    std::fs::write(artifacts.join("seed-1.gcode"), &one).unwrap();
    assert_ne!(motion(&zero), motion(&one));
    let after_one = output(0);
    std::fs::write(artifacts.join("seed-0-after-1.gcode"), &after_one).unwrap();
    assert_eq!(after_one, zero);
    std::thread::scope(|scope| {
        let left = scope.spawn(|| output(0));
        let right = scope.spawn(|| output(1));
        let left = left.join().unwrap();
        let right = right.join().unwrap();
        std::fs::write(artifacts.join("seed-0-concurrent.gcode"), &left).unwrap();
        std::fs::write(artifacts.join("seed-1-concurrent.gcode"), &right).unwrap();
        assert_eq!(left, zero);
        assert_eq!(right, one);
    });
}

fn motion(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with("G1 "))
        .collect()
}
