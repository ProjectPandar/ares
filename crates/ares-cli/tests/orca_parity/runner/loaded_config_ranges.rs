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

/// Writes a standalone `{kind}.json` for the native `--load-settings` /
/// `--load-filaments` path, mirroring `runner.rs::write_preset` (398-404):
/// injects `type`, `instantiation:"true"`, and normalizes `from` to the
/// lowercase `"system"` the native CLI accepts (the stock SeeMeCNC files
/// carry `"from":"System"`, which native rejects).
fn write_native_preset(path: &Path, kind: &str, fields: serde_json::Map<String, Value>) {
    let mut preset = fields;
    preset.insert("type".to_owned(), json!(kind));
    preset.insert("from".to_owned(), json!("system"));
    preset.insert("instantiation".to_owned(), json!("true"));
    write_json(path, &Value::Object(preset));
}

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
/// surface the same `SliceError::InvalidValuesInProject` text via
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
        write_native_preset(&directory.join(format!("{kind}.json")), kind, fields);
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
        matches!(validated, Err(SliceError::InvalidValuesInProject(text)) if text == case.error_lines),
        "validated must surface the same key:text lines; evidence {directory:?}"
    );
    assert!(
        matches!(sliced, Err(SliceError::InvalidValuesInProject(text)) if text == case.error_lines),
        "sliced must surface the same key:text lines; evidence {directory:?}"
    );
    assert!(!directory.join("ares.gcode").exists());
}

// --- O3 single-key out-of-range cases (original matrix vendor data) ---

#[tokio::test]
async fn retraction_when_cut_28_loaded_config() {
    original_profile(&RangeCase {
        vendor_name: "Creality",
        machine: "Creality SPARKX i7 0.4 nozzle",
        process: "0.20mm Standard @Creality SPARKX i7 0.4 nozzle",
        filament: "Generic PLA @SPARKX i7-all",
        error_lines: "retraction_distances_when_cut: 28 not in range [10.000000,18.000000]",
    })
    .await;
}

/// Creality K2 carries `retraction_distances_when_cut: "30"` (above
/// [10,18]). The `solid_infill_filament`/`internal_solid_filament_id`
/// legacy-alias collision is now resolved by upstream overwrite
/// semantics (`f5c869df`), so the O3 range loop reaches and rejects the
/// out-of-range retraction with the native text.
#[tokio::test]
async fn retraction_when_cut_30_loaded_config() {
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
async fn extruder_printable_height_2100_loaded_config() {
    original_profile(&RangeCase {
        vendor_name: "SeeMeCNC",
        machine: "SeeMeCNC BOSSdelta 500 0521 0.4 nozzle",
        process: "0.20mm Standard @SeeMeCNC BOSSdelta 500 0521 0.4",
        filament: "SeeMeCNC PLA",
        error_lines: "extruder_printable_height: 2100 not in range [0.000000,1000.000000]",
    })
    .await;
}

/// Anycubic Kobra S1 Max carries `retraction_distances_when_cut: ["0"]`
/// (below [10,18]) plus `filament_flush_temp: ["nil"]` — the nil `coInts`
/// element is the `INT_MAX` sentinel (Config.hpp:1013), which
/// `is_value_valid` reports out-of-range (max 1500). Two sorted lines.
#[tokio::test]
async fn nil_flush_temp_and_retraction_zero_loaded_config() {
    original_profile(&RangeCase {
        vendor_name: "Anycubic",
        machine: "Anycubic Kobra S1 Max 0.6 nozzle",
        process: "0.30mm Standard @Anycubic Kobra S1 Max 0.6 nozzle",
        filament: "Anycubic PLA @Anycubic Kobra S1 Max 0.6 nozzle",
        error_lines: "filament_flush_temp: nil not in range [0.000000,1500.000000]\nretraction_distances_when_cut: 0 not in range [10.000000,18.000000]",
    })
    .await;
}

// --- Override-driven cases: catalog + standalone files share the patch ---

struct OverrideCase {
    vendor_name: &'static str,
    machine: &'static str,
    process: &'static str,
    filament: &'static str,
    /// Which preset kind carries the injected field: machine|process|filament.
    kind: &'static str,
    key: &'static str,
    value: Value,
    error_lines: &'static str,
    prefix: &'static str,
}

/// Runs one case where a named field on `kind` is overridden in BOTH the
/// catalog (source of truth for Ares profile composition) and the standalone
/// `{kind}.json` fed to native. `error_lines` is the exact key:sorted text.
async fn run_override_case(case: &OverrideCase) {
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

/// `bridge_line_width` > min_nozzle_diameter — the separate
/// `Slic3r::validate` line-width bound (PrintConfig.cpp:10504-10517), not
/// the generic `is_value_valid` loop.
#[tokio::test]
async fn bridge_line_width_over_nozzle_loaded_config() {
    run_override_case(&OverrideCase {
        vendor_name: "Creality",
        machine: "Creality Ender-3 0.4 nozzle",
        process: "0.20mm Standard @Creality Ender3 0.4",
        filament: "Creality Generic PLA",
        kind: "process",
        key: "bridge_line_width",
        value: json!("1.0"),
        error_lines:
            "bridge_line_width: Bridge line width must not exceed nozzle diameter: 1.000000",
        prefix: "o3-bridge-",
    })
    .await;
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
        error_lines: "retraction_distances_when_cut: 28 not in range [10.000000,18.000000]\nuse_firmware_retraction: --use-firmware-retraction is not compatible with --wipe",
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
#[tokio::test]
async fn matrix_sweep_loaded_config() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    // K2 0.4 now resolves its `solid_infill_filament`/
    // `internal_solid_filament_id` legacy-alias collision via upstream
    // overwrite semantics (`f5c869df`), so every machine below reaches the
    // O3 range loop and is held to strict parity.
    const CASES: &[(&str, &str)] = &[
        ("Creality", "Creality K2 0.4 nozzle"),
        ("Creality", "Creality SPARKX i7 0.2 nozzle"),
        ("Creality", "Creality SPARKX i7 0.4 nozzle"),
        ("Creality", "Creality SPARKX i7 0.6 nozzle"),
        ("Creality", "Creality SPARKX i7 0.8 nozzle"),
        ("Anycubic", "Anycubic Kobra S1 Max 0.25 nozzle"),
        ("Anycubic", "Anycubic Kobra S1 Max 0.4 nozzle"),
        ("Anycubic", "Anycubic Kobra S1 Max 0.6 nozzle"),
        ("Anycubic", "Anycubic Kobra S1 Max 0.8 nozzle"),
        ("Anycubic", "Anycubic Kobra X 0.4 nozzle"),
        ("SeeMeCNC", "SeeMeCNC BOSSdelta 500 0521 0.4 nozzle"),
        ("SeeMeCNC", "SeeMeCNC BOSSdelta 500 0521 0.5 nozzle"),
        ("SeeMeCNC", "SeeMeCNC BOSSdelta 500 0521 0.7 nozzle"),
        ("SeeMeCNC", "SeeMeCNC BOSSdelta 500 0521 1.0 nozzle"),
    ];
    let root = super::super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let input = fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    let mut report = Vec::new();
    let mut failures = Vec::new();
    for (vendor_name, machine) in CASES {
        let profiles = crate::presets::VendorProfiles::load(&profiles_root, vendor_name).unwrap();
        let selection = crate::select_printer(&profiles, vendor_name, machine)
            .unwrap_or_else(|error| panic!("{vendor_name}/{machine}: {error}"));
        let catalog = json!({"vendors": [
            vendor(&profiles_root, "OrcaFilamentLibrary"),
            vendor(&profiles_root, vendor_name),
        ], "imports": []});
        let sel_json = json!({
            "machine": machine,
            "process": selection.process,
            "filaments": selection.filaments,
        });
        let directory = tempfile::Builder::new()
            .prefix(&format!("o3-sweep-{}-", machine.replace(' ', "_")))
            .tempdir()
            .unwrap()
            .keep();
        for (kind, fields) in [
            ("machine", profiles.machine(machine).unwrap()),
            ("process", profiles.process(&selection.process).unwrap()),
            (
                "filament",
                profiles.filament(&selection.filaments[0]).unwrap(),
            ),
        ] {
            write_native_preset(&directory.join(format!("{kind}.json")), kind, fields);
        }
        fs::write(directory.join("cube10.stl"), &input).unwrap();
        let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
        let native = stages::run(
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
        );
        let native_err = native
            .as_ref()
            .err()
            .and_then(|failure| failure.captured.as_ref())
            .map(|captured| String::from_utf8_lossy(&captured.stderr).into_owned());
        let request = ProfileSliceRequest {
            filename: "cube10.stl",
            catalog_json: &serde_json::to_string(&catalog).unwrap(),
            selection_json: &sel_json.to_string(),
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
        };
        let validated = ares_core::validate_with_profiles(&input, request);
        report.push(json!({
            "machine": machine,
            "process": selection.process,
            "native_stderr": native_err,
            "ares_validated": validated.as_ref().map_err(ToString::to_string),
        }));
        // Ares must reject with the identical `key: text` lines (the body
        // after the `Param values in 3mf/config error: ` header).
        let native_text = native_err
            .as_deref()
            .and_then(|text| text.split("Param values in 3mf/config error: \n").nth(1))
            .map(|body| body.trim_end().to_owned());
        let matched = matches!(
            (&validated, native_text.as_deref()),
            (Err(SliceError::InvalidValuesInProject(text)), Some(native)) if text == native
        );
        report.last_mut().unwrap()["native_text"] = json!(native_text);
        report.last_mut().unwrap()["matched"] = json!(matched);
        if !matched {
            failures.push(format!(
                "{vendor_name}/{machine}: ares={validated:?} native={native_text:?}"
            ));
        }
    }
    write_json(
        &runner.work.join("o3-sweep-report.json"),
        &Value::Array(report),
    );
    assert!(
        failures.is_empty(),
        "O3 sweep divergences:\n{}",
        failures.join("\n")
    );
}
