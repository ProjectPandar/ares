//! `PerimeterGenerator::process_no_bridge` chbBridges and `Layer::make_perimeters`.

use super::{OrcaRunner, cache_identity, profile_entry};
use ares_core::{ConfigOverrides, SliceError};
use serde_json::{Value, json};
use std::fs;

const MACHINE: &str = "WEMAKE3D PhoenixProV1 0.4mm nozzle";
const PROCESS: &str = "0.2mm Standard @WEMAKE3D PhoenixProV1 0.4mm nozzle";
const FILAMENT: &str = "Generic PLA @System";

pub(super) fn check(name: &str, filename: &str, none: bool) {
    if !super::environment::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = super::repo_root();
    let fixtures = root.join("tests/parity/counterbore-partiallybridge");
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("geometry-manifest.json")).unwrap())
            .unwrap();
    let expected = manifest["geometry"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == filename)
        .unwrap();
    let input = fs::read(fixtures.join(format!("{filename}.stl"))).unwrap();
    assert_eq!(cache_identity::raw_digest(&input), expected["sha256"]);
    let provenance: Value =
        serde_json::from_slice(&fs::read(fixtures.join("profile-provenance.json")).unwrap())
            .unwrap();
    for kind in ["machine", "process", "filament"] {
        for source in provenance[kind]["parent_chain"].as_array().unwrap() {
            let bytes = fs::read(profiles_root.join(source["path"].as_str().unwrap())).unwrap();
            assert_eq!(cache_identity::raw_digest(&bytes), source["sha256"]);
        }
    }
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "WEMAKE3D").unwrap();
    assert!(profiles.process_is_compatible(PROCESS, MACHINE));
    assert!(profiles.filament_is_compatible(FILAMENT, MACHINE));
    let catalog = json!({
        "vendors": [profile_entry::vendor(&profiles_root, "OrcaFilamentLibrary"),
                    profile_entry::vendor(&profiles_root, "WEMAKE3D")],
        "imports": [],
    });
    let selection = json!({"machine": MACHINE, "process": PROCESS, "filaments": [FILAMENT]});
    for (kind, mut fields) in [
        ("machine", profiles.machine(MACHINE).unwrap()),
        ("process", profiles.process(PROCESS).unwrap()),
        ("filament", profiles.filament(FILAMENT).unwrap()),
    ] {
        if kind == "process" {
            assert_eq!(fields["counterbore_hole_bridging"], "partiallybridge");
            assert_eq!(fields["wall_generator"], "classic");
            if none {
                fields.insert("counterbore_hole_bridging".to_owned(), json!("none"));
            }
        }
        profile_entry::write_json(&runner.work.join(format!("{kind}.json")), &json!(fields));
    }
    profile_entry::write_json(&runner.work.join("catalog.json"), &catalog);
    profile_entry::write_json(&runner.work.join("selection.json"), &selection);
    let overrides_json = if none {
        json!({"counterbore_hole_bridging": "none"})
    } else {
        json!({})
    };
    profile_entry::write_json(&runner.work.join("overrides.json"), &overrides_json);
    let overrides =
        ConfigOverrides::from_json(&serde_json::to_vec(&overrides_json).unwrap()).unwrap();
    fs::write(runner.work.join("input.stl"), &input).unwrap();
    let reference = runner
        .slice_stl_with_settings(
            [
                &runner.work.join("machine.json"),
                &runner.work.join("process.json"),
                &runner.work.join("filament.json"),
            ],
            &runner.work.join("input.stl"),
        )
        .unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &input,
            ares_core::ProfileSliceRequest {
                filename: "input.stl",
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
                if matches!(error, SliceError::UnsupportedProjectFeature(_)) {
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
fn counterbore_partiallybridge_primary_complete_output() {
    check("primary", "stepped-hole", false);
}

#[test]
fn counterbore_none_control_complete_output() {
    check("none-control", "stepped-hole", true);
}

#[test]
fn counterbore_first_layer_control_complete_output() {
    check("first-layer-control", "first-layer-hole", false);
}

#[test]
fn counterbore_straight_hole_control_complete_output() {
    check("straight-hole-control", "straight-hole", false);
}

#[test]
fn counterbore_solid_control_complete_output() {
    check("solid-control", "solid-plate", false);
}

#[test]
fn counterbore_two_islands_complete_output() {
    check("two-islands", "two-islands", false);
}
