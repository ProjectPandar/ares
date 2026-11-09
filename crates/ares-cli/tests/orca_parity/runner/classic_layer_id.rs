//! PerimeterGenerator.cpp:1463–1471 physical-layer sandwich gate public REDs.

use super::{OrcaRunner, cache_identity, profile_entry};
use ares_core::{ConfigOverrides, SliceError};
use serde_json::{Value, json};
use std::fs;

const MACHINE: &str = "WEMAKE3D PhoenixProV1 0.4mm nozzle";
const PROCESS: &str = "0.2mm Standard @WEMAKE3D PhoenixProV1 0.4mm nozzle";
const FILAMENT: &str = "Generic PLA @System";

fn check(name: &str) {
    if !super::environment::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = super::repo_root();
    let fixtures = root.join("tests/parity/classic-layer-id");
    let matrix_bytes = fs::read(fixtures.join("matrix.json")).unwrap();
    let matrix: Value = serde_json::from_slice(&matrix_bytes).unwrap();
    let case = matrix["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == name)
        .unwrap();
    let filename = case["geometry"].as_str().unwrap();
    let mode = case["counterbore_hole_bridging"].as_str().unwrap();
    let wall_generator = case["wall_generator"].as_str().unwrap();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&fs::read(fixtures.join("manifest.json")).unwrap()),
        matrix["geometry_manifest_sha256"]
    );
    assert_eq!(
        cache_identity::raw_digest(&fs::read(fixtures.join("profile-provenance.json")).unwrap()),
        matrix["profile_provenance_sha256"]
    );
    let expected = manifest["geometry"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == filename)
        .unwrap();
    let input_filename = case["input"].as_str().unwrap();
    let input = fs::read(fixtures.join(input_filename)).unwrap();
    assert_eq!(cache_identity::raw_digest(&input), expected["sha256"]);
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "WEMAKE3D").unwrap();
    assert!(profiles.process_is_compatible(PROCESS, MACHINE));
    assert!(profiles.filament_is_compatible(FILAMENT, MACHINE));
    let catalog = json!({
        "vendors": [profile_entry::vendor(&profiles_root, "OrcaFilamentLibrary"),
                    profile_entry::vendor(&profiles_root, "WEMAKE3D")],
        "imports": [],
    });
    let selection = json!({"machine": MACHINE, "process": PROCESS, "filaments": [FILAMENT]});
    // These files are flattened, so no `inherits`: with one, Native would have to
    // match them to a vendor source (`OrcaSlicer.cpp:2122-2123`, `PresetBundle.cpp:650`).
    for (kind, mut fields) in [
        ("machine", profiles.machine(MACHINE).unwrap()),
        ("process", profiles.process(PROCESS).unwrap()),
        ("filament", profiles.filament(FILAMENT).unwrap()),
    ] {
        if kind == "process" {
            assert_eq!(fields["counterbore_hole_bridging"], "partiallybridge");
            assert_eq!(fields["wall_generator"], "classic");
            fields.insert("counterbore_hole_bridging".to_owned(), json!(mode));
            fields.insert("wall_generator".to_owned(), json!(wall_generator));
        }
        profile_entry::write_json(&runner.work.join(format!("{kind}.json")), &json!(fields));
    }
    profile_entry::write_json(&runner.work.join("catalog.json"), &catalog);
    profile_entry::write_json(&runner.work.join("selection.json"), &selection);

    let overrides_json =
        json!({"wall_generator": wall_generator, "counterbore_hole_bridging": mode});
    profile_entry::write_json(&runner.work.join("overrides.json"), &overrides_json);
    let overrides =
        ConfigOverrides::from_json(&serde_json::to_vec(&overrides_json).unwrap()).unwrap();
    fs::write(runner.work.join(input_filename), &input).unwrap();
    let reference = runner
        .slice_stl_with_settings(
            [
                &runner.work.join("machine.json"),
                &runner.work.join("process.json"),
                &runner.work.join("filament.json"),
            ],
            &runner.work.join(input_filename),
        )
        .unwrap();
    fs::write(runner.work.join("native-counterbore.gcode"), &reference).unwrap();
    let native_text = std::str::from_utf8(&reference).unwrap();
    assert!(
        native_text
            .lines()
            .any(|l| l == format!("; wall_generator = {wall_generator}")),
        "Native effective wall_generator mismatch"
    );
    assert!(
        native_text
            .lines()
            .any(|l| l == format!("; counterbore_hole_bridging = {mode}")),
        "Native effective counterbore mode mismatch"
    );
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &input,
            ares_core::ProfileSliceRequest {
                filename: input_filename,
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ));
    let (status, detail, actual_sha256) = match &actual {
        Ok(bytes) => {
            fs::write(runner.work.join("ares.gcode"), bytes).unwrap();
            let compared = crate::golden::compare_ordered_bytes_generator_only(&reference, bytes);
            (
                if compared.is_ok() {
                    "PASS"
                } else {
                    "DIVERGENT"
                },
                compared.err(),
                Some(cache_identity::raw_digest(bytes)),
            )
        }
        Err(error) => {
            profile_entry::write_json(
                &runner.work.join("ares-error.json"),
                &json!({"debug": format!("{error:?}"), "display": error.to_string()}),
            );
            (
                if matches!(error, SliceError::UnsupportedProjectFeature(feature)
                    if *feature == "counterbore_hole_bridging")
                {
                    "GUARD_REJECTED"
                } else {
                    "ARES_ERROR"
                },
                Some(format!("{error:?}: {error}")),
                None,
            )
        }
    };
    profile_entry::write_json(
        &runner.work.join("counterbore-result.json"),
        &json!({
            "matrix_sha256": cache_identity::raw_digest(&matrix_bytes),
            "case": name, "input_sha256": cache_identity::raw_digest(&input),
            "catalog_sha256": cache_identity::raw_digest(catalog.to_string().as_bytes()),
            "selection": selection, "overrides": overrides_json,
            "native_sha256": cache_identity::raw_digest(&reference),
            "ares_sha256": actual_sha256, "status": status, "detail": detail,
            "rust_entry": "ares_core::slice_with_profiles", "evidence": crate::artifacts::EVIDENCE,
            "comparison": "complete bytes; generated-by identity only; movement/M73/stats/footer retained",
        }),
    );
    assert_eq!(status, "PASS", "{name}: {detail:?}; {:?}", runner.work);
}

#[test]
fn classic_later_owner_filled_actual_layer_order() {
    check("m-later-owner-classic-sacrificiallayer");
}

#[test]
fn classic_later_owner_none_actual_layer_order() {
    check("m-later-owner-classic-none");
}
