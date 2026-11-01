//! Public Arachne counterbore parity through original profiles and complete Native output.

use super::{OrcaRunner, cache_identity, profile_entry};
use ares_core::ConfigOverrides;
use serde_json::{Value, json};
use std::fs;

const MACHINE: &str = "WEMAKE3D PhoenixProV1 0.4mm nozzle";
const PROCESS: &str = "0.2mm Standard @WEMAKE3D PhoenixProV1 0.4mm nozzle";
const FILAMENT: &str = "Generic PLA @System";

#[test]
fn counterbore_n_split_arachne_filled_complete_output() {
    if !super::environment::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = super::repo_root();
    let fixtures = root.join("tests/parity/counterbore-n-split");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    let input = fs::read(fixtures.join("n-split.stl")).unwrap();
    assert_eq!(cache_identity::raw_digest(&input), manifest["input_sha256"]);
    assert_eq!(
        cache_identity::raw_digest(&fs::read(fixtures.join("n-split-shape.json")).unwrap()),
        manifest["shape_sha256"]
    );
    let provenance_bytes =
        fs::read(root.join(manifest["profile_provenance"].as_str().unwrap())).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&provenance_bytes),
        manifest["profile_provenance_sha256"]
    );
    let provenance: Value = serde_json::from_slice(&provenance_bytes).unwrap();
    let mode = "sacrificiallayer";
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
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
            fields.insert("counterbore_hole_bridging".to_owned(), json!(mode));
            fields.insert("wall_generator".to_owned(), json!("arachne"));
        }
        profile_entry::write_json(&runner.work.join(format!("{kind}.json")), &json!(fields));
    }
    profile_entry::write_json(&runner.work.join("catalog.json"), &catalog);
    profile_entry::write_json(&runner.work.join("selection.json"), &selection);
    let wall_generator = "arachne";
    let overrides_json =
        json!({"wall_generator": wall_generator, "counterbore_hole_bridging": mode});
    profile_entry::write_json(&runner.work.join("overrides.json"), &overrides_json);
    let overrides =
        ConfigOverrides::from_json(&serde_json::to_vec(&overrides_json).unwrap()).unwrap();
    fs::write(runner.work.join("n-split.stl"), &input).unwrap();
    let reference = runner
        .slice_stl_with_settings(
            [
                &runner.work.join("machine.json"),
                &runner.work.join("process.json"),
                &runner.work.join("filament.json"),
            ],
            &runner.work.join("n-split.stl"),
        )
        .unwrap();
    fs::write(runner.work.join("native-counterbore.gcode"), &reference).unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &input,
            ares_core::ProfileSliceRequest {
                filename: "n-split.stl",
                catalog_json: &catalog.to_string(),
                selection_json: &selection.to_string(),
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    let compared = crate::golden::compare_ordered_bytes_generator_only(&reference, &actual);
    profile_entry::write_json(
        &runner.work.join("counterbore-result.json"),
        &json!({
            "case": manifest["case"], "input_sha256": cache_identity::raw_digest(&input),
            "native_sha256": cache_identity::raw_digest(&reference),
            "ares_sha256": cache_identity::raw_digest(&actual),
            "catalog_sha256": cache_identity::raw_digest(catalog.to_string().as_bytes()),
            "selection": selection, "overrides": overrides_json,
            "status": if compared.is_ok() { "PASS" } else { "DIVERGENT" },
            "detail": compared.as_ref().err(),
            "rust_entry": "ares_core::slice_with_profiles", "evidence": crate::artifacts::EVIDENCE,
            "comparison": "complete bytes; generated-by identity only; movement/M73/stats/footer retained",
        }),
    );
    assert!(compared.is_ok(), "{:?}; {:?}", compared.err(), runner.work);
}
