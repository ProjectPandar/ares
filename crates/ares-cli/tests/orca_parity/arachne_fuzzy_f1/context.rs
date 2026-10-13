//! Public Context lifecycle is witnessed before any F1 implementation.
use super::{native, public, sha, witness};
use crate::runner::OrcaRunner;
use serde_json::json;
use std::fs;

#[test]
fn f1_context_seed_repeat_and_job_isolation() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("F1 Context Native oracle not configured");
        return;
    };
    let project =
        include_bytes!("../../../../../tests/parity/arachne-fuzzy-f1/classic-displacement.3mf");
    let root = tempfile::Builder::new()
        .prefix("fuzzy-f1-context-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let zero_native = native::prepare(
        &runner,
        "classic-displacement",
        project,
        0,
        &root.join("native-seed0"),
    );
    let seeded = std::env::var("ARES_PARITY_SEEDED_REFERENCE").as_deref() == Ok("1");
    let one_native = seeded.then(|| {
        native::prepare(
            &runner,
            "classic-displacement",
            project,
            1,
            &root.join("native-seed1"),
        )
    });
    let zero = public::slice(project, 0);
    let one = public::slice(project, 1);
    let repeat = public::slice(project, 0);
    let (parallel_zero, parallel_one) = std::thread::scope(|scope| {
        let left = scope.spawn(|| public::slice(project, 0));
        let right = scope.spawn(|| public::slice(project, 1));
        (left.join().unwrap(), right.join().unwrap())
    });
    let mut observations = Vec::new();
    for (label, result) in [
        ("seed0", &zero),
        ("seed1", &one),
        ("seed0-after1", &repeat),
        ("parallel0", &parallel_zero),
        ("parallel1", &parallel_one),
    ] {
        if let Ok(bytes) = result {
            fs::write(root.join(format!("{label}.gcode")), bytes).unwrap();
        }
        observations.push(json!({"label":label,"admitted":result.is_ok(),
            "error":result.as_ref().err().map(|e|format!("{e:?}")),
            "sha256":result.as_ref().ok().map(|bytes|sha(bytes))}));
    }
    let compare = |reference: &[u8], result: &Result<Vec<u8>, ares_core::SliceError>| {
        result
            .as_ref()
            .map_err(|e| format!("{e:?}"))
            .and_then(|g| crate::golden::compare_ordered_bytes_generator_only(reference, g))
    };
    let zero_comparison = seeded.then(|| compare(&zero_native.reference, &zero));
    let one_comparison = one_native.as_ref().map(|n| compare(&n.reference, &one));
    fs::write(
        root.join("context-evidence.json"),
        serde_json::to_vec_pretty(&json!({
            "observations":observations,"input_sha256":sha(project),
            "seeded_native_strict_parity_required":seeded,
            "seeded_native_strict_parity_performed":seeded && zero.is_ok() && one.is_ok(),
            "official_contract":"one Native stochastic diagnostic; no seed identity parity claimed",
            "seed0_comparison":zero_comparison,"seed1_comparison":one_comparison,
            "cli_scope":"CLI seed0 covered in corpus; explicit seed1 uses the public Context API"
        }))
        .unwrap(),
    )
    .unwrap();
    let zero = zero.unwrap_or_else(|e| panic!("public Context admission RED: {root:?}: {e:?}"));
    let one = one.unwrap();
    assert_eq!(repeat.unwrap(), zero, "fresh seed0 after seed1");
    assert_eq!(parallel_zero.unwrap(), zero, "independent job0");
    assert_eq!(parallel_one.unwrap(), one, "independent job1");
    assert_ne!(
        zero, one,
        "real source random stream must consume the task seed"
    );
    if let Some(result) = zero_comparison {
        result.unwrap();
    }
    if let Some(result) = one_comparison {
        result.unwrap();
    }
    if !seeded {
        witness::verify_diagnostic(&witness::capture(&zero), &zero_native.contract);
    }
}
