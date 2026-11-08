//! Direct-STL overrides retain the CLI non-BBL bookkeeping boundary.

use super::{OrcaRunner, cache_identity, profile_entry};
use ares_core::{ConfigOverrides, SliceError};
use serde_json::{Value, json};
use std::fs;

const MACHINE: &str = "WEMAKE3D PhoenixProV1 0.4mm nozzle";
const PROCESS: &str = "0.2mm Standard @WEMAKE3D PhoenixProV1 0.4mm nozzle";
const FILAMENT: &str = "Generic PLA @System";

fn check(name: &str, filename: &str, none: bool) {
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
        &runner.work.join("direct-stl-override-result.json"),
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

fn check_applied_machine_option(key: &str, value: Value, expected: &str) {
    let root = super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let mut machine_vendor = profile_entry::vendor(&profiles_root, "WEMAKE3D");
    let machine = machine_vendor["files"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .find(|profile| profile["type"] == "machine" && profile["name"] == MACHINE)
        .unwrap();
    machine[key] = value;
    let catalog = json!({
        "vendors": [profile_entry::vendor(&profiles_root, "OrcaFilamentLibrary"), machine_vendor],
        "imports": [],
    });
    let selection = json!({"machine": MACHINE, "process": PROCESS, "filaments": [FILAMENT]});
    let input =
        fs::read(root.join("tests/parity/counterbore-partiallybridge/solid-plate.stl")).unwrap();
    let artifacts = tempfile::Builder::new()
        .prefix("applied-machine-option-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(artifacts.join("input.stl"), &input).unwrap();
    profile_entry::write_json(&artifacts.join("catalog.json"), &catalog);
    profile_entry::write_json(&artifacts.join("selection.json"), &selection);
    let output = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &input,
            ares_core::ProfileSliceRequest {
                filename: "input.stl",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &ConfigOverrides::default(),
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(artifacts.join("ares.gcode"), &output).unwrap();
    let text = std::str::from_utf8(&output).unwrap();
    let block = text.split_once("; CONFIG_BLOCK_START\n").unwrap().1;
    let block = block.split_once("; CONFIG_BLOCK_END").unwrap().0;
    let lines = block
        .lines()
        .filter(|line| line.starts_with(&format!("; {key} = ")))
        .collect::<Vec<_>>();
    assert_eq!(lines, [format!("; {key} = {expected}")]);
}

#[test]
fn direct_stl_applied_print_host_true_is_emitted() {
    check_applied_machine_option("bbl_use_printhost", json!("1"), "1");
}

#[test]
fn direct_stl_applied_print_host_false_is_emitted() {
    check_applied_machine_option("bbl_use_printhost", json!("0"), "0");
}

#[test]
fn direct_stl_applied_default_bed_type_is_emitted() {
    check_applied_machine_option(
        "default_bed_type",
        json!("Textured PEI Plate"),
        "Textured PEI Plate",
    );
}

#[test]
fn direct_stl_applied_default_nozzle_volume_type_is_emitted() {
    check_applied_machine_option(
        "default_nozzle_volume_type",
        json!(["High Flow"]),
        "High Flow",
    );
}

#[test]
fn direct_stl_none_override_bookkeeping_complete_output() {
    check("direct-stl-none-override", "solid-plate", true);
}

#[test]
fn direct_stl_no_override_bookkeeping_control_complete_output() {
    check("direct-stl-no-override-control", "solid-plate", false);
}
