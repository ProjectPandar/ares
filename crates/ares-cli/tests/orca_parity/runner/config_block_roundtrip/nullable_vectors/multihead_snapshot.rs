//! `GCode.cpp:6451–6455,9831–9853`: runtime photo head1 maps to firmware label0.

use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::fs;

fn compare(enabled: bool) {
    let source = fs::read(repo_root().join("tests/parity/nozzle-flush-dataset/D0.3mf")).unwrap();
    assert_eq!(
        sha(&source),
        "6d576ce9d4835dfae9ec5edacf9c215d7d6893ce097606f06f987db98261ff34"
    );
    let mapped = super::mapped_nozzle::mapped_project(&source, 2);
    let project = super::input::settings_variant(
        &mapped,
        json!({
            "farthest_point_timelapse": if enabled { "1" } else { "0" },
            "timelapse_type":"0", "printer_structure":"corexy",
            "physical_extruder_map":["1","0"], "extruder_offset":["0x0","0x0"],
            "retraction_length":["0.4","0.4"], "retraction_speed":["30","30"],
            "filament_retraction_length":["3","3"], "filament_retraction_speed":["30","30"],
            "deretraction_speed":["30","30"],
            "machine_max_acceleration_e":["5000","5000","5000","5000"],
            "machine_max_speed_e":["120","120","120","120"]
        })
        .as_object()
        .unwrap(),
    );
    let case = if enabled { "head1-offset" } else { "disabled" };
    let (reference, _) = native_stream_case::qualify(
        (
            "multihead-inline-snapshot",
            &["GCode.cpp:6451-6455,9831-9853;GCode.cpp:6541-6547"],
        ),
        case,
        &project,
        &sha(&project),
        false,
    );
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ));
    fs::write(
        reference.root.join("multihead-api-result.json"),
        serde_json::to_vec_pretty(&json!({
            "case":case,"input_sha256":sha(&project),"runtime_head":1,
            "firmware_label":0,"nozzle_offset_mm":[0,0],
            "api_error":result.as_ref().err().map(|e|format!("{e:?}")),
            "whole_output_parity_claimed":false
        }))
        .unwrap(),
    )
    .unwrap();
    let api = result.unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    if enabled {
        crate::runner::first_travel_acceleration_ordering::assert_multi_nozzle_prefix(
            &reference.root,
            case,
            reference.native.as_bytes(),
            &api,
            "M9711 M0 E0 Z0.6 S11 C10 O0 T3000",
        );
    } else {
        let placement = |text: &str| {
            let lines = text.lines().collect::<Vec<_>>();
            let photo = lines
                .iter()
                .position(|s| s.starts_with("M622.1 S1"))
                .unwrap();
            let bottom = lines
                .iter()
                .position(|s| *s == "; FEATURE: Bottom surface")
                .unwrap();
            (photo > bottom, photo, bottom)
        };
        let native = placement(&reference.native);
        let actual = placement(std::str::from_utf8(&api).unwrap());
        fs::write(
            reference.root.join("multihead-disabled-section.json"),
            serde_json::to_vec_pretty(&json!({
                "case":case,"native_photo_after_bottom_surface":native.0,
                "ares_photo_after_bottom_surface":actual.0,"native_lines":native,
                "ares_lines":actual,"passed":native.0&&actual.0,
                "whole_output_parity_claimed":false
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(native.0, "Native disabled placement: {:?}", reference.root);
        assert!(
            actual.0,
            "disabled subsystem must not inline: {:?}",
            reference.root
        );
    }
}

#[test]
fn nonzero_photo_head_with_nozzle_offset_inlines_like_native() {
    compare(true);
}

#[test]
fn disabled_farthest_point_keeps_noninline_placement() {
    compare(false);
}
