//! Public Arachne counterbore parity through original profiles and complete Native output.

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
    let fixtures = root.join("tests/parity/counterbore-partiallybridge");
    let matrix_bytes =
        fs::read(root.join("tests/parity/arachne-counterbore-none/matrix.json")).unwrap();
    let matrix: Value = serde_json::from_slice(&matrix_bytes).unwrap();
    let case = matrix["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == name)
        .unwrap();
    let filename = case["geometry"].as_str().unwrap();
    let mode = case["counterbore_hole_bridging"].as_str().unwrap();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("geometry-manifest.json")).unwrap())
            .unwrap();
    assert_eq!(
        cache_identity::raw_digest(&fs::read(fixtures.join("geometry-manifest.json")).unwrap()),
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
    assert_eq!(
        profiles.process(PROCESS).unwrap()["counterbore_hole_bridging"],
        "partiallybridge"
    );
    assert_eq!(
        profiles.process(PROCESS).unwrap()["wall_generator"],
        "classic"
    );
    profile_entry::write_json(&runner.work.join("catalog.json"), &catalog);
    profile_entry::write_json(&runner.work.join("selection.json"), &selection);
    let wall_generator = matrix["wall_generator"].as_str().unwrap();
    assert_eq!(wall_generator, "arachne");
    let mut activation_verified = true;
    let overrides_json =
        json!({"wall_generator": wall_generator, "counterbore_hole_bridging": mode});
    profile_entry::write_json(&runner.work.join("overrides.json"), &overrides_json);
    let overrides =
        ConfigOverrides::from_json(&serde_json::to_vec(&overrides_json).unwrap()).unwrap();
    fs::write(runner.work.join("input.stl"), &input).unwrap();
    let native_presets = profile_entry::native_user_presets::resolved(
        &runner.work,
        &profiles_root,
        &catalog,
        &selection,
    );
    let reference = profile_entry::native_user_presets::slice_stl(
        &runner,
        &native_presets,
        &runner.work.join("input.stl"),
        "native",
        &overrides_json,
    );
    fs::write(runner.work.join("native-counterbore.gcode"), &reference).unwrap();
    if filename == "stepped-hole" && mode != "none" {
        let mut none = overrides_json.clone();
        none["counterbore_hole_bridging"] = json!("none");
        let control = profile_entry::native_user_presets::slice_stl(
            &runner,
            &native_presets,
            &runner.work.join("input.stl"),
            "native-none",
            &none,
        );
        fs::write(runner.work.join("native-none.gcode"), &control).unwrap();
        let none_normalized = crate::golden::normalize_one_generator_line(
            &control,
            crate::golden::GeneratorKind::Orca,
        )
        .unwrap();
        let counterbore_normalized = crate::golden::normalize_one_generator_line(
            &reference,
            crate::golden::GeneratorKind::Orca,
        )
        .unwrap();
        let difference = crate::golden::first_difference(&none_normalized, &counterbore_normalized)
            .map(|difference| difference.to_string());
        // Diagnostic Native-only activation witness; Ares acceptance below compares all bytes.
        let motion = |bytes: &[u8]| {
            bytes
                .split(|byte| *byte == b'\n')
                .filter(|line| {
                    [b"G0 ".as_slice(), b"G1 ", b"G2 ", b"G3 "]
                        .iter()
                        .any(|prefix| line.starts_with(prefix))
                })
                .flat_map(|line| line.iter().copied().chain(std::iter::once(b'\n')))
                .collect::<Vec<u8>>()
        };
        let motion_difference = crate::golden::first_difference(
            &motion(&none_normalized),
            &motion(&counterbore_normalized),
        )
        .map(|difference| difference.to_string());
        profile_entry::write_json(
            &runner.work.join("native-counterbore-reachability.json"),
            &json!({
                "none_sha256": cache_identity::raw_digest(&control),
                "counterbore_sha256": cache_identity::raw_digest(&reference),
                "difference": difference, "motion_difference": motion_difference,
                "purpose": "independent Native counterbore versus None output witness, not branch coverage",
            }),
        );
        activation_verified = difference.is_some() && motion_difference.is_some();
    }
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
    assert!(
        activation_verified,
        "primary must change Native motion versus None, not only config footer; {:?}",
        runner.work
    );
    assert_eq!(status, "PASS", "{name}: {detail:?}; {:?}", runner.work);
}

#[test]
fn arachne_counterbore_none_primary_complete_output() {
    check("none-primary");
}
