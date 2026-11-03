//! PrintConfig.cpp:10354-10358,10450-10454: diameter and ratio validation.
use super::*;
use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use runner::stages::{FailureKind, Stage};

#[test]
fn original_orca_and_ares_reject_invalid_filament_ratio_and_diameter() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let mut proofs = Vec::new();
    for (key, valid, invalid) in [
        ("filament_flow_ratio", "1", "0"),
        ("filament_diameter", "1.75", "0.99999"),
    ] {
        let baseline = input::export(
            &runner,
            key,
            valid,
            &format!("bounded-filament/{key}/{valid}"),
        )
        .unwrap();
        let error = input::export(
            &runner,
            key,
            invalid,
            &format!("bounded-filament/{key}/{invalid}"),
        )
        .and_then(|exported| runner.slice_case(&exported))
        .err()
        .expect("invalid filament value must be rejected");
        assert_eq!(error.stage, Stage::Export, "{key}: {error}");
        assert_eq!(error.kind, FailureKind::Process);
        let captured = error.captured.as_ref().unwrap();
        assert_eq!(
            captured.code,
            Some(if cfg!(unix) { 238 } else { -18 }),
            "{key}: {error}"
        );
        let diagnostic = format!("{key}: invalid value {invalid}");
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&captured.stdout),
            String::from_utf8_lossy(&captured.stderr)
        );
        assert!(text.contains(&diagnostic), "{key}: {text}");
        assert!(
            error
                .evidence
                .as_ref()
                .unwrap()
                .join("status.json")
                .is_file()
        );
        let overrides = ConfigOverrides::from_json(
            serde_json::to_vec(&json!({key:[invalid]}))
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let validation = ares_core::validate_project_plate(&baseline.project, None, &overrides);
        assert_rejected(validation, &diagnostic);
        let sliced = runtime.block_on(crate::runner::application::slice_project_plate_validated(
            &baseline.project,
            GenerationMetadata::default(),
            None,
            &overrides,
            SlicerContext::new(0),
        ));
        assert_rejected(sliced.map(|_| ()), &diagnostic);
        proofs.push(json!({"key":key,"requested":[invalid],"stage":"Export",
            "native_code":captured.code,"diagnostic":diagnostic,"evidence":error.evidence,
            "public_validation":"same rejection","validated_slice":"same rejection"}));
    }
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &root.join("filament-ratio-diameter-rejections.json"),
        &serde_json::to_vec_pretty(&proofs).unwrap(),
    )
    .unwrap();
}

fn assert_rejected(result: Result<(), SliceError>, diagnostic: &str) {
    match result {
        Err(SliceError::InvalidInput(message)) => assert_eq!(message, diagnostic),
        other => panic!("public filament rejection disagrees: {other:?}"),
    }
}
