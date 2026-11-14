use std::fs;

use crate::runner::{OrcaRunner, cache_identity, stages};
use serde_json::json;

#[test]
fn stl_override_preserves_vendor_default_thumbnails() {
    if !crate::runner::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let root = crate::runner::repo_root();
    let profiles = root.join("OrcaSlicer/resources/profiles");
    let vendors = ["OrcaFilamentLibrary", "WEMAKE3D"];
    let catalog = json!({
        "vendors": vendors.map(|name| super::vendor(&profiles, name)),
        "imports": [],
    });
    let selection = json!({
        "machine": "WEMAKE3D PhoenixProV1 0.4mm nozzle",
        "process": "0.2mm Standard @WEMAKE3D PhoenixProV1 0.4mm nozzle",
        "filaments": ["Generic PLA @System"],
    });
    let override_json = json!({"counterbore_hole_bridging": "sacrificiallayer"});
    let overrides =
        ares_core::ConfigOverrides::from_json(&serde_json::to_vec(&override_json).unwrap())
            .unwrap();
    let input =
        fs::read(root.join("tests/parity/counterbore-partiallybridge/stepped-hole.stl")).unwrap();
    fs::write(runner.work.join("input.stl"), &input).unwrap();
    super::write_json(&runner.work.join("catalog.json"), &catalog);
    super::write_json(&runner.work.join("selection.json"), &selection);
    super::write_json(&runner.work.join("overrides.json"), &override_json);

    // Native resolves the original manifest-owned leaves, not flattened configs.
    let datadir = super::native_user_presets::system(&runner.work, &profiles, &vendors);
    let leaves = [
        ("machine", selection["machine"].as_str().unwrap()),
        ("process", selection["process"].as_str().unwrap()),
        ("filament", selection["filaments"][0].as_str().unwrap()),
    ]
    .map(|(kind, name)| {
        let (vendor, relative, fields) = catalog["vendors"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|vendor| {
                vendor["files"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(move |(path, fields)| (vendor["id"].as_str().unwrap(), path, fields))
            })
            .find(|(_, _, fields)| fields["type"] == kind && fields["name"] == name)
            .unwrap();
        let path = datadir.join("system").join(vendor).join(relative);
        let mut fields = fields.clone();
        // File admission needs the system identity; option values stay authored.
        fields["from"] = json!("system");
        super::write_json(&path, &fields);
        path
    });
    let output = runner.work.join("native");
    fs::create_dir(&output).unwrap();
    stages::run(
        &runner.bin,
        [
            "--datadir".to_owned(),
            datadir.to_str().unwrap().to_owned(),
            "--load-settings".to_owned(),
            crate::runner::join_paths(leaves[..2].iter()),
            "--load-filaments".to_owned(),
            leaves[2].to_str().unwrap().to_owned(),
            "--arrange".to_owned(),
            "0".to_owned(),
            "--slice".to_owned(),
            "0".to_owned(),
            "--counterbore-hole-bridging".to_owned(),
            "sacrificiallayer".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            runner.work.join("input.stl").to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    let native = fs::read(output.join("plate_1.gcode")).unwrap();
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
        ))
        .unwrap();
    fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    let thumbnail_line = |bytes: &[u8]| {
        std::str::from_utf8(bytes)
            .unwrap()
            .lines()
            .find(|line| line.starts_with("; thumbnails = "))
            .map(str::to_owned)
    };
    let native_thumbnails = thumbnail_line(&native);
    let actual_thumbnails = thumbnail_line(&actual);
    let comparison = crate::golden::compare_ordered_bytes_generator_only(&native, &actual);
    super::write_json(
        &runner.work.join("result.json"),
        &json!({
            "input_sha256": cache_identity::raw_digest(&input),
            "native_sha256": cache_identity::raw_digest(&native),
            "ares_sha256": cache_identity::raw_digest(&actual),
            "native_thumbnails": native_thumbnails,
            "ares_thumbnails": actual_thumbnails,
            "selection": selection,
            "overrides": override_json,
            "rust_entry": "ares_core::slice_with_profiles",
            "evidence": crate::artifacts::EVIDENCE,
            "comparison": comparison.as_ref().err().map(ToString::to_string),
        }),
    );
    // PrintConfig.cpp:8603 supplies this default after source JSON composite handling.
    assert_eq!(
        native_thumbnails.as_deref(),
        Some("; thumbnails = 48x48/PNG,300x300/PNG")
    );
    assert_eq!(actual_thumbnails, native_thumbnails);
    comparison.unwrap();
}
