//! OrcaSlicer.cpp:3647-3653 / PrintConfig.cpp:10519-10559 (is_value_valid
//! out-of-range loop) + 10504-10517 (line-width nozzle bound), original
//! O3 matrix inputs. Shares the single `Slic3r::validate` error map with O2.
use std::fs;
use std::path::Path;

use ares_core::{
    ConfigOverrides, GenerationMetadata, Placement, ProfileSliceRequest, SliceError, SlicerContext,
};
use serde_json::{Value, json};

use super::{OrcaRunner, stages, vendor, write_json};

#[path = "loaded_config_ranges/stock_filament.rs"]
mod stock_filament;

/// Retains flattened input snapshots alongside the manifest-owned Native leaves.
fn write_native_preset(path: &Path, kind: &str, fields: serde_json::Map<String, Value>) {
    let mut preset = fields;
    preset.insert("type".to_owned(), json!(kind));
    preset.insert("from".to_owned(), json!("system"));
    preset.insert("instantiation".to_owned(), json!("true"));
    write_json(path, &Value::Object(preset));
}

struct RangeCase<'a> {
    /// Vendor directory under `OrcaSlicer/resources/profiles`.
    vendor_name: &'static str,
    machine: &'static str,
    /// Process preset filename stem (loaded via `profiles.process`).
    process: &'static str,
    filament: &'a str,
    /// Exact `key: text` lines after the `Param values in 3mf/config
    /// error: ` header, in the native (key-sorted) order.
    error_lines: &'static str,
}

#[path = "loaded_config_ranges/original.rs"]
mod original;
use original::original_profile;

// --- O3 single-key out-of-range cases (original matrix vendor data) ---

#[tokio::test]
async fn retraction_when_cut_28_loaded_config() {
    original_profile(&RangeCase {
        vendor_name: "Creality",
        machine: "Creality SPARKX i7 0.4 nozzle",
        process: "0.20mm Standard @Creality SPARKX i7 0.4 nozzle",
        filament: "Generic PLA @SPARKX i7-all",
        error_lines: "retraction_distances_when_cut: 28 not in range [10, 18]",
    })
    .await;
}

#[tokio::test]
async fn extruder_printable_height_2100_loaded_config() {
    original_profile(&RangeCase {
        vendor_name: "SeeMeCNC",
        machine: "SeeMeCNC BOSSdelta 500 0521 0.4 nozzle",
        process: "0.20mm Standard @SeeMeCNC BOSSdelta 500 0521 0.4",
        filament: "SeeMeCNC PLA",
        error_lines: "extruder_printable_height: 2100 not in range [0, 1000]",
    })
    .await;
}

/// Anycubic Kobra S1 Max carries `retraction_distances_when_cut: ["0"]`
/// (below [10,18]) plus `filament_flush_temp: ["nil"]`. Vendor loading completes
/// the whole nil vector from its parent/default before validation
/// (Preset.cpp:313-370; PresetBundle.cpp:7781-7848), retaining only the retraction error.
#[tokio::test]
async fn nil_flush_temp_and_retraction_zero_loaded_config() {
    original_profile(&RangeCase {
        vendor_name: "Anycubic",
        machine: "Anycubic Kobra S1 Max 0.6 nozzle",
        process: "0.30mm Standard @Anycubic Kobra S1 Max 0.6 nozzle",
        filament: "Anycubic PLA @Anycubic Kobra S1 Max 0.6 nozzle",
        error_lines: "retraction_distances_when_cut: 0 not in range [10, 18]",
    })
    .await;
}

// --- Override-driven cases: catalog + Native vendor leaves share the patch ---

struct OverrideCase<'a> {
    vendor_name: &'static str,
    machine: &'static str,
    process: &'static str,
    filament: &'a str,
    /// Which preset kind carries the injected field: machine|process|filament.
    kind: &'static str,
    key: &'static str,
    value: Value,
    error_lines: &'static str,
    prefix: &'static str,
}

/// The named catalog source edit reaches both Ares and Native's staged vendor leaf.
/// `error_lines` is the exact key-sorted diagnostic.
async fn run_override_case(case: &OverrideCase<'_>) {
    let OverrideCase {
        vendor_name,
        machine,
        process,
        filament,
        kind,
        key,
        value,
        error_lines,
        prefix,
    } = case;
    let root = super::super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, vendor_name).unwrap();
    let machine_fields = profiles.machine(machine).unwrap();
    let process_fields = profiles.process(process).unwrap();
    let filament_fields = profiles.filament(filament).unwrap();
    let mut catalog = json!({"vendors": [
        vendor(&profiles_root, "OrcaFilamentLibrary"),
        vendor(&profiles_root, vendor_name),
    ], "imports": []});
    // Apply the override to the catalog's named file so the Ares profile
    // composition path sees it (catalog, not the standalone json, drives it).
    // The catalog `name` is the preset name for that kind.
    let target_name = match *kind {
        "machine" => *machine,
        "process" => *process,
        "filament" => *filament,
        other => panic!("unknown preset kind {other}"),
    };
    for file in catalog["vendors"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .flat_map(|vendor| vendor["files"].as_object_mut().unwrap().values_mut())
    {
        if file["type"] == json!(kind) && file["name"] == json!(target_name) {
            file.as_object_mut()
                .unwrap()
                .insert(key.to_string(), value.clone());
        }
    }
    let selection = json!({
        "machine": machine, "process": process, "filaments": [filament],
    });
    let directory = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    let input = fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    fs::write(directory.join("cube10.stl"), &input).unwrap();
    for (k, mut fields) in [
        ("machine", machine_fields),
        ("process", process_fields),
        ("filament", filament_fields),
    ] {
        if k == *kind {
            fields.insert(key.to_string(), value.clone());
        }
        write_native_preset(&directory.join(format!("{k}.json")), k, fields);
    }
    let expected_stderr = format!("Param values in 3mf/config error: \n{error_lines}\n");
    let native = OrcaRunner::from_env().unwrap().map(|runner| {
        let presets =
            super::native_user_presets::resolved(&directory, &profiles_root, &catalog, &selection);
        let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
        let failure = stages::run(
            runner.bin(),
            [
                "--datadir".to_owned(),
                presets.datadir.to_str().unwrap().to_owned(),
                "--load-settings".to_owned(),
                super::super::join_paths(presets.settings.iter()),
                "--load-filaments".to_owned(),
                super::super::join_paths(presets.filaments.iter()),
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
            "expected": error_lines,
            "validated": validated.as_ref().map_err(ToString::to_string),
            "sliced": sliced.as_ref().map(|_| "GCODE").map_err(ToString::to_string),
        }),
    );
    assert!(
        matches!(validated, Err(SliceError::InvalidValuesInProject(text)) if text == *error_lines),
        "validated must emit the same key:text lines; evidence {directory:?}"
    );
    assert!(
        matches!(sliced, Err(SliceError::InvalidValuesInProject(text)) if text == *error_lines),
        "sliced must emit the same key:text lines; evidence {directory:?}"
    );
}

// --- O2+O3 simultaneous violation: shared ordered map ---

/// Firmware-retraction (O2) and retraction-distances-when-cut (O3) are both
/// violated; native prints both lines sorted by key, after the shared
/// `Param values in 3mf/config error: ` header. The machine already carries
/// `wipe: "1"`; forcing `use_firmware_retraction` adds the O2 violation.
#[tokio::test]
async fn firmware_and_range_loaded_config() {
    run_override_case(&OverrideCase {
        vendor_name: "Creality",
        machine: "Creality SPARKX i7 0.4 nozzle",
        process: "0.20mm Standard @Creality SPARKX i7 0.4 nozzle",
        filament: "Generic PLA @SPARKX i7-all",
        kind: "machine",
        key: "use_firmware_retraction",
        value: json!("1"),
        error_lines: "retraction_distances_when_cut: 28 not in range [10, 18]\nuse_firmware_retraction: --use-firmware-retraction is not compatible with --wipe",
        prefix: "o3-combo-",
    })
    .await;
}

// --- Full 15-case O3 sweep: native vs Ares same-text rejection ---

/// The 15 matrix O3 printers. Each resolves its default process + filament
/// via `select_printer` (matching the matrix harness), slices through native
/// `--load-settings` (exit 238, `Param values in 3mf/config error:`) and
/// asserts Ares `validate_with_profiles` rejects with the identical
/// `key: text` line set. The per-case text is captured from native at
/// runtime so the sweep is a self-documenting report of the shared
/// `Slic3r::validate` map.
#[path = "loaded_config_ranges/sweep.rs"]
mod sweep;

#[tokio::test]
async fn matrix_sweep_loaded_config() {
    sweep::run().await;
}
