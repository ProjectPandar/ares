use std::io::{Cursor, Read, Write};

use super::{ExportedCase, OrcaRunner};

#[test]
fn seeded_template_flashforge_04() {
    verify("flashforge-04", false);
}

#[test]
fn seeded_template_flashforge_06() {
    verify("flashforge-06", false);
}

#[test]
fn seeded_template_flashforge_08() {
    verify("flashforge-08", false);
}

#[test]
fn seeded_template_raise3d_dual() {
    verify("raise3d-dual", false);
}

#[test]
fn seeded_template_raise3d_left() {
    verify("raise3d-left", false);
}

#[test]
fn seeded_template_raise3d_right() {
    verify("raise3d-right", false);
}

#[test]
fn seeded_template_distributions_and_shared_stream() {
    verify("raise3d-left", true);
}

#[test]
fn seeded_template_file_start_has_no_rng_context() {
    if !crate::runner::seeded_reference_enabled(None) {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("seeded native runner");
    let input = super::repo_root().join("tests/parity/template-rng/raise3d-left.3mf");
    let project = rewrite_settings(&std::fs::read(input).unwrap(), |settings| {
        settings["file_start_gcode"] = "; RNG no context {random(2,8)}\n".into();
    });
    let project_path = runner.work.join("input.3mf");
    std::fs::write(&project_path, &project).unwrap();
    let output_dir = runner.work.join("reference");
    std::fs::create_dir(&output_dir).unwrap();
    let exported = ExportedCase {
        label: "template-rng/no-context".to_owned(),
        project,
        project_path,
        output_dir,
    };
    assert!(runner.slice_case(&exported).is_err());
    let error = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            &exported.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap_err();
    std::fs::write(runner.work.join("ares-error.txt"), error.to_string()).unwrap();
    assert!(
        error
            .to_string()
            .contains("Random number generator not available in this context.")
    );
}

fn verify(fixture: &str, probe: bool) {
    if !crate::runner::seeded_reference_enabled(None) {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("seeded template E2E requires the native runner");
    let seed = std::env::var("ORCA_REPLAY_SEED")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let input = super::repo_root()
        .join("tests/parity/template-rng")
        .join(format!("{fixture}.3mf"));
    let mut project = std::fs::read(input).unwrap();
    if probe {
        project = rewrite_settings(&project, append_probes);
    }
    let settings = super::application::settings(&project).unwrap();
    assert!(
        settings["machine_start_gcode"]
            .as_str()
            .unwrap()
            .contains("random(")
    );
    let output_dir = runner.work.join("reference");
    std::fs::create_dir(&output_dir).unwrap();
    let project_path = runner.work.join("input.3mf");
    std::fs::write(&project_path, &project).unwrap();
    let exported = ExportedCase {
        label: format!("template-rng/{fixture}/probe-{probe}/seed-{seed}"),
        project,
        project_path,
        output_dir,
    };
    let first = runner.slice_case(&exported).unwrap();
    if probe {
        let text = std::str::from_utf8(&first.reference).unwrap();
        for marker in [
            "integral-real",
            "mixed",
            "full-int",
            "rejection",
            "integer-division",
            "equal",
            "skipped",
            "layer",
            "end",
        ] {
            let prefix = format!("; RNG {marker}");
            assert!(
                text.lines().any(|line| line.starts_with(&prefix)),
                "missing {marker} probe"
            );
        }
    }
    std::fs::write(runner.work.join("reference-first.gcode"), &first.reference).unwrap();
    let second = runner.rerun_reference(&exported).unwrap();
    std::fs::write(
        runner.work.join("reference-repeat.gcode"),
        &second.reference,
    )
    .unwrap();
    let normalize = |bytes: &[u8]| {
        crate::golden::normalize_one_generator_line(bytes, crate::golden::GeneratorKind::Orca)
            .unwrap()
    };
    if let Some(difference) =
        crate::golden::first_difference(&normalize(&first.reference), &normalize(&second.reference))
    {
        panic!("native repeat differs: {difference}");
    }
    let actual = slice(&exported.project, seed);
    std::fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    let comparison = crate::golden::compare_ordered_bytes_generator_only(&first.reference, &actual);
    std::fs::write(
        runner.work.join("result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "label": exported.label,
            "seed": seed,
            "cpu": std::env::var("ORCA_REPLAY_CPU").unwrap(),
            "binary": std::env::var("ORCA_REPLAY_BIN").unwrap(),
            "evidence": crate::artifacts::EVIDENCE,
            "native_repeat": "PASS",
            "status": if comparison.is_ok() { "PASS" } else { "DIVERGENT" },
        }))
        .unwrap(),
    )
    .unwrap();
    comparison.unwrap();
    verify_isolation(&runner, &exported.project, seed, &actual);
}

fn slice(project: &[u8], seed: u32) -> Vec<u8> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(seed),
        ))
        .unwrap()
}

fn verify_isolation(runner: &OrcaRunner, project: &[u8], seed: u32, actual: &[u8]) {
    let other = slice(project, seed ^ 1);
    std::fs::write(runner.work.join("ares-other-seed.gcode"), &other).unwrap();
    assert_ne!(actual, other);
    let repeat = slice(project, seed);
    std::fs::write(runner.work.join("ares-repeat.gcode"), &repeat).unwrap();
    assert_eq!(actual, repeat);
    std::thread::scope(|scope| {
        let left = scope.spawn(|| slice(project, seed));
        let right = scope.spawn(|| slice(project, seed ^ 1));
        assert_eq!(left.join().unwrap(), actual);
        assert_eq!(right.join().unwrap(), other);
    });
}

fn rewrite_settings(project: &[u8], update: fn(&mut serde_json::Value)) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            update(&mut settings);
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

fn append_probes(settings: &mut serde_json::Value) {
    for (key, suffix) in [
        (
            "machine_start_gcode",
            "\n; RNG integral-real {digits(random(2.0,8.0),0,20)}\n; RNG mixed {digits(random(2,8.25),0,20)}\n; RNG full-int {random(-2147483647-1,2147483647)}\n; RNG rejection {random(-1073741824,1073741824)}\n; RNG integer-division {random(2,8)/2}\n; RNG equal {random(3,3)}\n; RNG skipped {if false}{random(2,8)}{endif}\n",
        ),
        (
            "before_layer_change_gcode",
            "\n{if layer_num == 0}; RNG layer {digits(random(2.0,8.0),0,20)} {random(-1073741824,1073741824)}{endif}\n",
        ),
        (
            "machine_end_gcode",
            "\n; RNG end {random(-2147483647-1,2147483647)} {digits(random(2,8.25),0,20)}\n",
        ),
    ] {
        let template = settings[key].as_str().unwrap();
        settings[key] = serde_json::Value::String(format!("{template}{suffix}"));
    }
}
