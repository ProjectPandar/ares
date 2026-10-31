//! Print.cpp:1524-1556,1654-1671: unchanged stock minimum-width rejection inputs.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::json;

use crate::runner::{OrcaRunner, stages};

const ERROR: &str = "Too small line width";

macro_rules! stock_case {
    ($name:ident, $fixture:literal, $hash:literal, $label:literal) => {
        #[test]
        fn $name() {
            verify(
                $label,
                include_bytes!(concat!("../../../../../", $fixture)),
                $hash,
            );
        }
    };
}

stock_case!(
    o4_cr10se_0_2,
    "tests/parity/validation/minimum-width-cr10se-0.2.3mf",
    "dcaf990d1563fb43f91eecdd1e97dd9212da4a37d734cef6a551219e49350c0c",
    "stock/Creality/Creality CR-10 SE 0.2 nozzle"
);
stock_case!(
    o4_ender3_0_2,
    "tests/parity/validation/minimum-width-ender3-0.2.3mf",
    "741badc6fd0024dea8473f81f48beb4940c590d4c6e9d221341f9f7423ac16d6",
    "stock/Creality/Creality Ender-3 0.2 nozzle"
);
stock_case!(
    o4_ender3pro_0_2,
    "tests/parity/validation/minimum-width-ender3pro-0.2.3mf",
    "e8d7c006d42f6ae5079e1e093836f96c263d854f4e3d9660cc603e67da025751",
    "stock/Creality/Creality Ender-3 Pro 0.2 nozzle"
);
stock_case!(
    o4_ender3s1plus_0_2,
    "tests/parity/validation/minimum-width-ender3s1plus-0.2.3mf",
    "edb48192a609ede526e8e09e805b845ff99494c0d2d2b31a8655ca9b0ac89110",
    "stock/Creality/Creality Ender-3 S1 Plus 0.2 nozzle"
);
stock_case!(
    o4_ender3v3ke_0_2,
    "tests/parity/validation/minimum-width-ender3v3ke-0.2.3mf",
    "e2d577d48b05b2238f00d7fdb3fdd707efdac05580ae71d37be70e6e9c11a3d8",
    "stock/Creality/Creality Ender-3 V3 KE 0.2 nozzle"
);
stock_case!(
    o4_ender3v3se_0_2,
    "tests/parity/validation/minimum-width-ender3v3se-0.2.3mf",
    "4da4853ebdbfeeb2770552341dbb756ec0d1fdd95c8f0c9bf2fe80f1dbe232a2",
    "stock/Creality/Creality Ender-3 V3 SE 0.2 nozzle"
);
stock_case!(
    o4_lk10_0_2,
    "tests/parity/validation/minimum-width-lk10-0.2.3mf",
    "93a1a1639f38a3e4c419472c8be00c06004d2e73e6254fbf34834f4d2acb68c7",
    "stock/LONGER/LONGER LK10 (0.2 nozzle)"
);
stock_case!(
    o4_lk10plus_0_2,
    "tests/parity/validation/minimum-width-lk10plus-0.2.3mf",
    "36021e556f2c003fcbf965bd8802c57bbe2e8c39e63d9370edf2023e4603d699",
    "stock/LONGER/LONGER LK10 Plus (0.2 nozzle)"
);

fn verify(label: &str, input: &[u8], expected_hash: &str) {
    assert_eq!(super::sha256(input), expected_hash);
    let directory = super::evidence_directory("o4-stock-");
    let path = directory.join("input.3mf");
    fs::write(&path, input).unwrap();
    let runner = OrcaRunner::from_env().unwrap();
    let oracle = runner.as_ref().map(|runner| {
        let output = directory.join("orca");
        fs::create_dir(&output).unwrap();
        let error = stages::run(
            runner.bin(),
            [
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                path.to_str().unwrap().to_owned(),
            ],
            stages::Stage::Slice,
            &directory,
        )
        .unwrap_err();
        let captured = error.captured.as_ref().unwrap();
        assert_eq!(
            captured.code,
            Some(if cfg!(unix) { 205 } else { -51 }),
            "{error}"
        );
        assert_eq!(captured.stderr, b"Too small line width\n", "{error}");
        assert!(!output.join("plate_1.gcode").exists());
        json!({"code":captured.code, "stderr":"Too small line width\n", "evidence":error.evidence})
    });
    let overrides = ConfigOverrides::default();
    let validation = ares_core::validate_project_plate(input, None, &overrides);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let sliced = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.block_on(ares_core::slice_project_plate_validated(
            input,
            GenerationMetadata::default(),
            None,
            &overrides,
            SlicerContext::new(0),
        ))
    }));
    let slice_report = match &sliced {
        Ok(Ok(gcode)) => {
            fs::write(directory.join("ares.gcode"), gcode).unwrap();
            json!({"status":"GENERATED", "bytes":gcode.len(), "sha256":super::sha256(gcode)})
        }
        Ok(Err(error)) => json!({"status":"ERROR", "error":error.to_string()}),
        Err(payload) => {
            json!({"status":"PANICKED", "error":payload.downcast_ref::<&str>().copied().or_else(|| payload.downcast_ref::<String>().map(String::as_str))})
        }
    };
    let validation_report = match &validation {
        Ok(()) => json!({"status":"VALIDATED"}),
        Err(error) => json!({"status":"ERROR", "error":error.to_string()}),
    };
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "printer":label, "input_sha256":expected_hash, "expected_error":ERROR,
        "oracle":oracle, "ares_validation":validation_report, "ares_validated_slice":slice_report,
    })).unwrap()).unwrap();
    assert!(
        matches!(validation, Err(SliceError::Validate { message: ref text, .. }) if text == ERROR),
        "independent Print validation must match native; evidence: {}",
        directory.display()
    );
    assert!(
        matches!(sliced, Ok(Err(SliceError::Validate { message: ref text, .. })) if text == ERROR),
        "validated composition must match native; evidence: {}",
        directory.display()
    );
    assert!(!directory.join("ares.gcode").exists());
}
