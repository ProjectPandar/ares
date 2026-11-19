//! Print.hpp:1086 and Print.cpp:3444-3541: stock BBL Type1 tower parity.
//! The width60 witness pins the `prime_tower_skip_points` waypoint travels
//! (`GCode.cpp:1002-1046` → `generate_path_to_wipe_tower`, `GCode.cpp:594-710`):
//! the polyline `Point`s flow straight into `travel_to(p)` / `point_to_gcode`
//! (`GCode.cpp:8124-8127`), and the post-toolchange slope lift
//! (`GCodeWriter.cpp:685-770`) reads the gridded writer position — at
//! `prime_tower_width` 60 the slope top lands on the 3-decimal rounding
//! boundary (Y221.158 vs Y221.159, 13 occurrences).

const UPSTREAM: &[&str] = &[
    "Print.hpp:1086",
    "Print.cpp:3290-3301,3390-3541",
    "GCode/WipeTower.cpp:2449-2510,3516-3806,3864",
    "GCode.cpp:712-1117,2926-2939",
];

const LAST_CHANGE_HEIGHT: &[&str] =
    &["GCode/WipeTower.cpp:4496-4507,4861-4862,5057-5093,5528-5533"];

fn framework_project(source: &[u8], updates: serde_json::Value) -> Vec<u8> {
    use std::io::{Cursor, Write};

    let mut archive = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let entry = archive.by_index(i).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            output
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            let mut settings: serde_json::Value = serde_json::from_reader(entry).unwrap();
            assert_eq!(settings["prime_tower_enable_framework"], "0");
            settings["prime_tower_enable_framework"] = serde_json::json!("1");
            settings
                .as_object_mut()
                .unwrap()
                .extend(updates.as_object().unwrap().clone());
            output
                .write_all(&serde_json::to_vec(&settings).unwrap())
                .unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    output.finish().unwrap().into_inner()
}

#[test]
fn bbl_type1_prime_tower_waypoints_enabled() {
    crate::runner::native_stream_case::check(
        ("bbl-type1-tower", UPSTREAM),
        "enabled",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/enabled.3mf"),
        "ab00b9040c3ccc1c10490a220880b87108aed7edd3a19926b8e7967f1287e506",
        (false, |_, _| {}),
    );
}

#[test]
fn bbl_type1_framework_skips_finish_block_above_last_change_height() {
    let source = include_bytes!("../../../../../tests/parity/bbl-type1-tower/enabled.3mf");
    let project = framework_project(source, serde_json::json!({}));
    crate::runner::native_stream_case::check(
        ("bbl-type1-tower", LAST_CHANGE_HEIGHT),
        "framework-above-last-change-height",
        &project,
        &crate::runner::native_api_cli::sha(&project),
        (false, |_, _| {}),
    );
}

#[test]
fn bbl_type1_framework_keeps_finish_block_inside_two_heads_heights() {
    use crate::runner::{application, native_api_cli::sha, native_stream_case};
    use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};

    let source = include_bytes!(
        "../../../../../tests/parity/filament-group-flush-restriction-pairs/x2d_pla_both_heads_reach_models.3mf"
    );
    let project = framework_project(
        source,
        serde_json::json!({
            "enable_prime_tower": "1",
            "filament_map": ["1", "2", "1"],
            "filament_map_mode": "Manual",
        }),
    );
    let (reference, _) = native_stream_case::qualify(
        ("bbl-type1-tower", LAST_CHANGE_HEIGHT),
        "framework-two-head-heights",
        &project,
        &sha(&project),
        true,
    );
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            &project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = reference.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    assert_eq!(api, std::fs::read(cli_path).unwrap());

    // Earlier tower geometry differs; this control checks the final block is admitted.
    for text in [&reference.native, std::str::from_utf8(&api).unwrap()] {
        assert!(text.lines().any(|line| line == "; filament_map = 1,2,1"));
        assert!(
            text.lines()
                .any(|line| line == "; extruder_printable_height = 261,256")
        );
        let (_, last_layer) = text.rsplit_once("; CHANGE_LAYER\n").unwrap();
        assert!(last_layer.starts_with("; Z_HEIGHT: 1.2\n"));
        assert!(last_layer.lines().any(|line| line == "T0 H-1"));
        assert!(last_layer.lines().any(|line| line == "T1 H-1"));
        assert!(last_layer.contains("; CP EMPTY GRID START\n; layer #7\n"));
        assert_eq!(last_layer.matches("; CP EMPTY GRID END\n").count(), 1);
    }
}

#[test]
fn bbl_type1_prime_tower_preserves_filament_start_escapes() {
    crate::runner::native_stream_case::check(
        (
            "bbl-type1-tower",
            &[
                "GCode.cpp:1593-1613,1624-1631",
                "PlaceholderParser.cpp:1064-1067",
                "Config.cpp:76-96,150-170",
            ],
        ),
        "filament-start-escapes",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/filament-start-escapes.3mf"),
        "04eaea81b8553416ea0d6418a5aa5f948a365998a37197acb78451cdd6de038e",
        (false, |native, _| {
            assert!(
                native.native.lines().any(|line| {
                    line == r#"; filament start literal=\n quote="double" slash=\q"#
                })
            );
        }),
    );
}

#[test]
fn bbl_type1_prime_tower_waypoints_width60_skip_points_slope_boundary() {
    crate::runner::native_stream_case::check(
        (
            "bbl-type1-tower",
            &[
                "GCode.cpp:594-710,1002-1046",
                "GCodeWriter.cpp:685-770",
                "GCode.cpp:8124-8127",
            ],
        ),
        "width60",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/width60.3mf"),
        "8bea13f0c36e384615a1c9dea4960ba62a5395be43d8c15e15acca21cee85563",
        (true, |_, _| {}),
    );
}

#[test]
fn bbl_type1_prime_tower_waypoints_rib3_segment_unscale() {
    crate::runner::native_stream_case::check(
        (
            "bbl-type1-tower",
            &["GCode/WipeTower.cpp:883-894,1102-1153", "Point.hpp:688-705"],
        ),
        "rib3",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/rib3.3mf"),
        "986e7ea38d1f60ff26e086c0ae8cdb4ba68336ba5d442689276deba603768f5d",
        (false, |_, _| {}),
    );
}

#[test]
fn bbl_type1_prime_tower_waypoints_disabled_control() {
    crate::runner::native_stream_case::check(
        ("bbl-type1-tower", UPSTREAM),
        "disabled",
        include_bytes!("../../../../../tests/parity/bbl-type1-tower/disabled.3mf"),
        "e78d0feba53df6318deb96929eb0620d5392f16a88832fb4ffd75969b853ed28",
        (false, |_, _| {}),
    );
}
