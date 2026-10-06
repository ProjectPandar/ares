//! OrcaSlicer.cpp:3647-3653 / PrintConfig.cpp:10519-10559 (is_value_valid
//! out-of-range loop) + 10504-10517 (line-width nozzle bound), original
//! O3 matrix inputs. Shares the single `Slic3r::validate` error map with O2.
use std::fs;

use ares_core::{
    ConfigOverrides, GenerationMetadata, Placement, ProfileSliceRequest, SliceError, SlicerContext,
};
use serde_json::{Value, json};

use super::{OrcaRunner, stages, vendor, write_json};

struct RangeCase {
    /// Vendor directory under `OrcaSlicer/resources/profiles`.
    vendor_name: &'static str,
    machine: &'static str,
    /// Process preset filename stem (loaded via `profiles.process`).
    process: &'static str,
    filament: &'static str,
    /// Exact `key: text` lines after the `Param values in 3mf/config
    /// error: ` header, in the native (key-sorted) order.
    error_lines: &'static str,
}

/// One O3 case: native `--load-settings` rejects with exit 238 and the
/// exact `key: <serialize()> not in range [min,max]` text; Ares must
/// surface the same `SliceError::InvalidInput` text via
/// `validate_with_profiles` / `slice_with_profiles_validated`.
async fn original_profile(case: &RangeCase) {
    let root = super::super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, case.vendor_name).unwrap();
    let machine_fields = profiles.machine(case.machine).unwrap();
    let process_fields = profiles.process(case.process).unwrap();
    let filament_fields = profiles.filament(case.filament).unwrap();
    let catalog = json!({"vendors": [
        vendor(&profiles_root, "OrcaFilamentLibrary"),
        vendor(&profiles_root, case.vendor_name),
    ], "imports": []});
    let selection = json!({
        "machine": case.machine,
        "process": case.process,
        "filaments": [case.filament],
    });
    let directory = tempfile::Builder::new()
        .prefix(&format!("o3-{}-", case.machine.replace(' ', "_")))
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    let input = fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    fs::write(directory.join("cube10.stl"), &input).unwrap();
    for (kind, fields) in [
        ("machine", machine_fields),
        ("process", process_fields),
        ("filament", filament_fields),
    ] {
        write_json(
            &directory.join(format!("{kind}.json")),
            &Value::Object(fields),
        );
    }
    let expected_stderr = format!("Param values in 3mf/config error: \n{}\n", case.error_lines);
    let native = OrcaRunner::from_env().unwrap().map(|runner| {
        let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
        let failure = stages::run(
            runner.bin(),
            [
                "--load-settings".to_owned(),
                format!("{};{}", path("machine.json"), path("process.json")),
                "--load-filaments".to_owned(),
                path("filament.json"),
                "--arrange".to_owned(),
                "1".to_owned(),
                "--export-3mf".to_owned(),
                path("native.3mf"),
                path("cube10.stl"),
            ],
            stages::Stage::Export,
            &directory,
        )
        .unwrap_err();
        let captured = failure.captured.as_ref().unwrap();
        assert_eq!(captured.code, Some(if cfg!(unix) { 238 } else { -18 }));
        assert_eq!(
            std::str::from_utf8(&captured.stderr).unwrap(),
            expected_stderr,
            "native diagnostic; evidence {directory:?}"
        );
        assert!(!directory.join("native.3mf").exists());
        json!({"code": captured.code, "stderr": expected_stderr, "evidence": failure.evidence})
    });
    let catalog_json = serde_json::to_string(&catalog).unwrap();
    let selection_json = serde_json::to_string(&selection).unwrap();
    let overrides = ConfigOverrides::default();
    let request = ProfileSliceRequest {
        filename: "cube10.stl",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: Placement::None,
    };
    let loaded = ares_core::profile_settings(&input, request).await;
    let placed = ares_core::profile_placed_project(&input, request).await;
    assert!(
        placed.is_ok(),
        "placed scene must allow editing invalid config; evidence {directory:?}"
    );
    let validated = ares_core::validate_with_profiles(&input, request);
    let sliced = ares_core::slice_with_profiles_validated(
        &input,
        request,
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await;
    if let Ok(gcode) = &sliced {
        fs::write(directory.join("ares.gcode"), gcode).unwrap();
    }
    write_json(
        &directory.join("result.json"),
        &json!({
            "native": native,
            "expected": case.error_lines,
            "loaded": loaded.as_ref().map(|view| json!(view)).map_err(ToString::to_string),
            "validated": validated.as_ref().map_err(ToString::to_string),
            "sliced": sliced.as_ref().map(|_| "GCODE").map_err(ToString::to_string),
        }),
    );
    loaded.unwrap();
    assert!(
        matches!(validated, Err(SliceError::InvalidInput(text)) if text == case.error_lines),
        "validated must surface the same key:text lines; evidence {directory:?}"
    );
    assert!(
        matches!(sliced, Err(SliceError::InvalidInput(text)) if text == case.error_lines),
        "sliced must surface the same key:text lines; evidence {directory:?}"
    );
    assert!(!directory.join("ares.gcode").exists());
}

// --- O3 single-key out-of-range cases (original matrix vendor data) ---

#[tokio::test]
async fn orca_parity_o3_retraction_when_cut_28_loaded_config() {
    original_profile(&RangeCase {
        vendor_name: "Creality",
        machine: "Creality SPARKX i7 0.4 nozzle",
        process: "0.20mm Standard @Creality SPARKX i7 0.4 nozzle",
        filament: "Generic PLA @SPARKX i7-all",
        error_lines: "retraction_distances_when_cut: 28 not in range [10.000000,18.000000]",
    })
    .await;
}

#[tokio::test]
async fn orca_parity_o3_retraction_when_cut_30_loaded_config() {
    original_profile(&RangeCase {
        vendor_name: "Creality",
        machine: "Creality K2 0.4 nozzle",
        process: "0.20mm Standard @Creality K2 0.4 nozzle",
        filament: "Creality Generic PLA @K2-all",
        error_lines: "retraction_distances_when_cut: 30 not in range [10.000000,18.000000]",
    })
    .await;
}

#[tokio::test]
async fn orca_parity_o3_extruder_printable_height_2100_loaded_config() {
    original_profile(&RangeCase {
        vendor_name: "SeeMeCNC",
        machine: "SeeMeCNC BOSSdelta 500 0521 0.4 nozzle",
        process: "0.20mm Standard @SeeMeCNC BOSSdelta 500 0521 0.4",
        filament: "SeeMeCNC PLA",
        error_lines: "extruder_printable_height: 2100 not in range [0.000000,1000.000000]",
    })
    .await;
}

// --- O2+O3 simultaneous violation: shared ordered map ---

/// Firmware-retraction (O2) and retraction-distances-when-cut (O3) are both
/// violated; native prints both lines sorted by key, after the shared
/// `Param values in 3mf/config error: ` header.
#[tokio::test]
async fn orca_parity_o3_firmware_and_range_loaded_config() {
    let root = super::super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "Creality").unwrap();
    let machine = "Creality SPARKX i7 0.4 nozzle";
    let process = "0.20mm Standard @Creality SPARKX i7 0.4 nozzle";
    let filament = "Generic PLA @SPARKX i7-all";
    let mut machine_fields = profiles.machine(machine).unwrap();
    // Force the O2 violation alongside the machine's O3 (28) violation;
    // the machine already carries `wipe: "1"` (printer scope, Bools).
    machine_fields.insert("use_firmware_retraction".to_owned(), json!("1"));
    let catalog = json!({"vendors": [
        vendor(&profiles_root, "OrcaFilamentLibrary"),
        vendor(&profiles_root, "Creality"),
    ], "imports": []});
    let selection = json!({
        "machine": machine, "process": process, "filaments": [filament],
    });
    let directory = tempfile::Builder::new()
        .prefix("o3-combo-")
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    let input = fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    fs::write(directory.join("cube10.stl"), &input).unwrap();
    write_json(
        &directory.join("machine.json"),
        &Value::Object(machine_fields),
    );
    write_json(
        &directory.join("process.json"),
        &Value::Object(profiles.process(process).unwrap()),
    );
    write_json(
        &directory.join("filament.json"),
        &Value::Object(profiles.filament(filament).unwrap()),
    );
    const EXPECTED: &str = "retraction_distances_when_cut: 28 not in range [10.000000,18.000000]\nuse_firmware_retraction: --use-firmware-retraction is not compatible with --wipe";
    let native = OrcaRunner::from_env().unwrap().map(|runner| {
        let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
        let failure = stages::run(
            runner.bin(),
            [
                "--load-settings".to_owned(),
                format!("{};{}", path("machine.json"), path("process.json")),
                "--load-filaments".to_owned(),
                path("filament.json"),
                "--arrange".to_owned(),
                "1".to_owned(),
                "--export-3mf".to_owned(),
                path("native.3mf"),
                path("cube10.stl"),
            ],
            stages::Stage::Export,
            &directory,
        )
        .unwrap_err();
        let captured = failure.captured.as_ref().unwrap();
        assert_eq!(captured.code, Some(if cfg!(unix) { 238 } else { -18 }));
        assert_eq!(
            std::str::from_utf8(&captured.stderr).unwrap(),
            format!("Param values in 3mf/config error: \n{EXPECTED}\n"),
            "native ordered multi-error; evidence {directory:?}"
        );
        json!({"code": captured.code, "stderr": EXPECTED, "evidence": failure.evidence})
    });
    let catalog_json = serde_json::to_string(&catalog).unwrap();
    let selection_json = serde_json::to_string(&selection).unwrap();
    let overrides = ConfigOverrides::default();
    let request = ProfileSliceRequest {
        filename: "cube10.stl",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: Placement::None,
    };
    let validated = ares_core::validate_with_profiles(&input, request);
    let sliced = ares_core::slice_with_profiles_validated(
        &input,
        request,
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await;
    write_json(
        &directory.join("result.json"),
        &json!({
            "native": native,
            "expected": EXPECTED,
            "validated": validated.as_ref().map_err(ToString::to_string),
            "sliced": sliced.as_ref().map(|_| "GCODE").map_err(ToString::to_string),
        }),
    );
    assert!(
        matches!(validated, Err(SliceError::InvalidInput(text)) if text == EXPECTED),
        "validated must emit both lines in key order; evidence {directory:?}"
    );
    assert!(
        matches!(sliced, Err(SliceError::InvalidInput(text)) if text == EXPECTED),
        "sliced must emit both lines in key order; evidence {directory:?}"
    );
}
