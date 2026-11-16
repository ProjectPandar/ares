use super::producer;
use crate::runner::{
    cache_identity,
    command::{self, Limits},
    repo_root,
    stages::Stage,
};
use serde_json::json;
use std::{fs, time::Duration};

#[test]
fn direct_ksr_retained_missing_options_match_native_effective_config() {
    let Some(runner) = producer::runner(
        "runner::stock_project_native_stream::retained_config::direct_ksr_retained_missing_options_match_native_effective_config",
    ) else {
        return;
    };
    let project =
        fs::read(repo_root().join("tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf")).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&project),
        "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9"
    );
    let source = runner.work.join("ksr.3mf");
    fs::write(&source, &project).unwrap();
    let supply = crate::runner::application::project_supply()
        .unwrap()
        .unwrap();
    let native_path = runner.work.join("native-settings.json");
    command::run(
        runner.bin(),
        [
            "--datadir".to_owned(),
            supply.native_dir.to_string_lossy().into_owned(),
            "--export-settings".to_owned(),
            native_path.to_string_lossy().into_owned(),
            source.to_string_lossy().into_owned(),
        ],
        &[("LC_ALL", "C")],
        Limits {
            stage: Stage::Export,
            work: &runner.work,
            timeout: Duration::from_secs(120),
        },
    )
    .unwrap();
    let native: serde_json::Value =
        serde_json::from_slice(&fs::read(native_path).unwrap()).unwrap();
    let keys = [
        "farthest_point_timelapse",
        "enable_pre_heating",
        "group_algo_with_time",
        "bottom_color_penetration_layers",
        "top_color_penetration_layers",
        "extruder_clearance_dist_to_rod",
    ];
    let overrides = ares_core::ConfigOverrides::default();
    let catalog = ares_core::PresetCatalogHandle::from_json(&supply.catalog).unwrap();
    let output = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_with_catalog(
            &project,
            ares_core::GenerationMetadata::default(),
            (None, &overrides),
            Some(&catalog),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(runner.work.join("api.gcode"), &output).unwrap();
    let output = String::from_utf8(output).unwrap();
    let values = keys.map(|key| {
        let actual = if key == "farthest_point_timelapse" {
            let snapshot = output
                .lines()
                .find(|line| line.starts_with("M9711 "))
                .unwrap();
            Some(if snapshot.contains(" X") { "0" } else { "1" })
        } else {
            let prefix = format!("; {key} = ");
            output.lines().find_map(|line| line.strip_prefix(&prefix))
        };
        json!({"key": key, "native": native[key], "api": actual})
    });
    fs::write(
        runner.work.join("effective-config-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "catalog_sha256": supply.digest,
            "values": values,
        }))
        .unwrap(),
    )
    .unwrap();
    for value in values {
        assert_eq!(value["api"], value["native"], "{value}; {:?}", runner.work);
    }
}
