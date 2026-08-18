use super::motion::MotionOptions;
use crate::{Nullable, OrcaFloat};

#[test]
fn ksr_motion_options_resolve_from_typed_project_settings() {
    let prepared = crate::project_slice::perimeters::prepare_post_classic_traversal(
        crate::project_slice::tests::support::ksr_project(),
    )
    .unwrap();

    let options = MotionOptions::from_traversal(&prepared);

    assert_eq!(options.travel_feedrate, 60_000.0);
    assert_eq!(options.first_layer_travel_feedrate, 60_000.0);
    assert_eq!(
        options.filament_area,
        std::f64::consts::PI * 1.75_f64.powi(2) * 0.25
    );
    assert_eq!(options.filament_flow_ratio, 0.98);
    assert_eq!(options.max_volumetric_speed, 21.0);
    assert_eq!(options.initial_layer_speed, 50.0);
    assert_eq!(options.initial_layer_infill_speed, 105.0);
    assert_eq!(options.inner_wall_speed, 300.0);
    assert_eq!(options.outer_wall_speed, 200.0);
    assert_eq!(options.bridge_speed, 50.0);
    assert_eq!(options.internal_bridge_speed, 75.0);
    assert_eq!(options.sparse_infill_speed, 270.0);
    assert_eq!(options.internal_solid_infill_speed, 250.0);
    assert_eq!(options.top_surface_speed, 200.0);
    assert_eq!(options.gap_infill_speed, 250.0);
    assert_eq!(options.initial_layer_acceleration, 500);
    assert_eq!(options.default_acceleration, 10_000);
    assert_eq!(options.outer_wall_acceleration, 5_000);
    assert_eq!(options.bridge_acceleration, 2_500);
    assert_eq!(options.top_surface_acceleration, 2_000);
    assert_eq!(options.initial_layer_travel_acceleration, 6_000);
    assert_eq!(options.travel_acceleration, 10_000);
    assert_eq!(options.retraction_length, 0.4);
    assert_eq!(options.deretraction_feedrate, 1_800.0);
    assert_eq!(options.z_hop, 0.4);
    assert_eq!(options.retraction_feedrate, 1_800.0);
    assert!(options.wipe);
    assert_eq!(options.wipe_distance, 1.0);
    assert_eq!(options.retraction_minimum_travel, 1.0);
    assert!(options.reduce_infill_retraction);
    assert_eq!(options.retract_before_wipe, 0.0);
    assert!(options.role_based_wipe_speed);
    assert!(options.spiral_lift);
    assert_eq!(options.travel_slope_radians, 3.0_f64.to_radians());
    assert_eq!(options.seam_gap, 0.04);
}

#[test]
fn nullable_filament_flow_ratio_uses_selected_value() {
    assert_eq!(
        super::motion::first_nullable_float(
            &[Nullable::Nil, Nullable::Value(OrcaFloat(0.97)),],
            1.0
        ),
        0.97,
    );
}

#[tokio::test]
async fn ksr_project_motion_is_finite_and_uses_configured_first_layer_rates() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(!output.contains("Einf"));
    assert!(!output.contains("Enan"));
    assert!(!output.contains(" F0\n"));
    assert!(output.contains("G1 X144.504 Y100.092 E.63582\n"));
    assert!(output.contains("; FEATURE: Inner wall\n; LINE_WIDTH: 0.5\nG1 F3000\n"));
    assert!(output.contains("; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 F15791.926\n"));
}

#[tokio::test]
async fn task22o143_ksr_seam_gap_clips_before_path_simplification() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let feature = lines
        .iter()
        .position(|line| *line == "; FEATURE: Inner wall")
        .unwrap();
    let travel = lines[..feature]
        .iter()
        .rposition(|line| line.starts_with("G1 X") && line.ends_with(" F60000"))
        .unwrap();
    let travel_xy = lines[travel]
        .split_ascii_whitespace()
        .skip(1)
        .take(2)
        .collect::<Vec<_>>();
    let next_travel = lines[feature + 1..]
        .iter()
        .position(|line| line.starts_with("G1 X") && line.ends_with(" F60000"))
        .map(|offset| feature + 1 + offset)
        .unwrap();

    assert_eq!(lines[next_travel - 2], "G1 X140.174 Y102.761 E.02841");
    assert!(!lines[feature + 1..next_travel].iter().any(|line| {
        line.split_ascii_whitespace()
            .skip(1)
            .take(2)
            .eq(travel_xy.iter().copied())
    }));
}

#[tokio::test]
async fn ksr_machine_start_uses_skirt_expanded_first_layer_hull() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.contains("G29 A1 X93.5392 Y79.8921 I78.1 J73.1 R\n"));
}
#[tokio::test]
async fn ksr_first_object_travel_uses_configured_acceleration_lift_and_deretraction() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let label = lines
        .iter()
        .position(|line| *line == "M624 AQAAAAAAAAA=")
        .unwrap();

    assert_eq!(lines[label - 4], "M204 S500");
    assert!(lines[label - 3].starts_with("; printing object "));
    assert_eq!(lines[label - 2], "M204 S6000");
    assert_eq!(
        lines[label - 1],
        "; start printing object, unique label id: 133"
    );
    assert!(lines[label + 1].starts_with("G1 X"));
    assert!(lines[label + 1].ends_with(" F60000"));
    assert_eq!(lines[label + 2], "G1 Z.6");
    assert_eq!(lines[label + 3], "G1 Z.2");
    assert_eq!(lines[label + 4], "G1 E.4 F1800");
}

#[tokio::test]
async fn ksr_project_emits_3mf_object_labels_per_layer() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert_eq!(
        output
            .matches("; printing object ksr_fdmtest_v4.drc id:2 copy 0\n")
            .count(),
        460
    );
    assert_eq!(
        output
            .matches("; start printing object, unique label id: 133\nM624 AQAAAAAAAAA=\n")
            .count(),
        460
    );
    assert_eq!(
        output
            .matches("; stop printing object, unique label id: 133\nM625\n")
            .count(),
        460
    );
}

#[tokio::test]
async fn gcode_label_objects_false_suppresses_project_object_labels() {
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"gcode_label_objects\": \"1\"",
        "\"gcode_label_objects\": \"0\"",
    );

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(!output.contains("; printing object "));
    assert!(!output.contains("; start printing object, unique label id:"));
    assert!(!output.contains("; stop printing object "));
    assert!(!output.contains("; stop printing object, unique label id:"));
}

#[tokio::test]
async fn disable_m73_true_suppresses_project_progress_lines() {
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"disable_m73\": \"0\"",
        "\"disable_m73\": \"1\"",
    );

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(!output.lines().any(|line| line.starts_with("M73 P")));
}

#[tokio::test]
async fn ksr_layer_change_appends_fan_speed_marker_after_custom_gcode() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert_eq!(
        output.matches(";_SET_FAN_SPEED_CHANGING_LAYER\n").count(),
        460
    );
    assert!(output.contains(
        "M991 S0 P0 ;notify layer change\n\n;_SET_FAN_SPEED_CHANGING_LAYER\nM204 S500\n"
    ));
    assert!(output.contains(
        ";===== 2025/04/08 =====\n\n\n\n\n\n\n\n    M106 P2 S102\n\tM106 P10 S102\n\n;not reset fan\n"
    ));
    assert!(output.contains("M106 S255\nM106 P2 S178\n; CHANGE_LAYER\n"));
}

#[tokio::test]
async fn ksr_project_renders_timelapse_template_after_every_layer() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert_eq!(output.matches("\nM9711 M0 E1 X133 Y193 Z").count(), 460);
    assert!(output.contains("M9711 M0 E1 X133 Y193 Z0.6 S11 C10 O0 T3000\n"));
    assert!(output.contains("M9711 M0 E1 X133 Y193 Z92.4 S11 C10 O0 T3000\n"));
    assert_eq!(
        output
            .matches(";======== X2D timelapse gcode ========\n")
            .count(),
        460
    );
}

#[tokio::test]
async fn ksr_project_retracts_wipes_and_lifts_before_timelapse() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();
    let invalid_numeric_word = output.lines().find(|line| {
        line.starts_with('G')
            && line
                .split_whitespace()
                .any(|word| matches!(word, "I" | "J" | "E" | "F"))
    });
    assert_eq!(invalid_numeric_word, None);

    assert!(output.contains(
        "; stop printing object ksr_fdmtest_v4.drc id:2 copy 0\nG1 E-.11429 F1800\n; WIPE_START\nG1 F6300\nG1 X109.036 Y94.518 E-.28571\n; WIPE_END\nG17\nG3 Z.6 I1.217 J0 P1  F60000\n; stop printing object, unique label id: 133\nM625\n;======== X2D timelapse gcode ========\n"
    ));
}

#[tokio::test]
async fn ksr_layer_metadata_uses_orca_float_precision() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.contains("; Z_HEIGHT: 8.6\n; LAYER_HEIGHT: 0.200001\n"));
    assert!(output.contains("; Z_HEIGHT: 91.4\n; LAYER_HEIGHT: 0.200005\n"));
    assert_eq!(output.matches("; CHANGE_LAYER\n; Z_HEIGHT:").count(), 460);
    assert!(!output.contains("; Z_HEIGHT: 0.6000000000000001\n"));
}

#[tokio::test]
async fn ksr_internal_bridges_keep_their_processor_role() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.contains("; FEATURE: Bridge\n"));
    assert!(output.contains("; FEATURE: Internal Bridge\n"));
    assert!(output.contains(
        "M204 S2500\n; FEATURE: Internal Bridge\n; LINE_WIDTH: 0.4\n; LAYER_HEIGHT: 0.4\nG1 F4500\n"
    ));
}
#[tokio::test]
async fn ksr_project_renders_end_templates_and_closes_executable_block() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.contains(";======== X2D timelapse gcode ========\n"));
    assert!(output.contains("; filament end gcode \n"));
    assert!(output.contains(";======== X2D end gcode ==========\n"));
    assert!(output.contains("M73 P100 R0\n; EXECUTABLE_BLOCK_END\n\n; filament used [mm] = "));
    assert!(
        output
            .lines()
            .last()
            .unwrap()
            .starts_with("; filament cost = ")
    );
    assert!(!output.ends_with("M2\n"));
}

#[tokio::test]
async fn task22o131_lifted_next_layer_travel_keeps_current_z() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let second_label = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == "M624 AQAAAAAAAAA=")
        .nth(1)
        .unwrap()
        .0;

    assert!(lines[second_label + 1].ends_with(" F60000"));
    assert!(!lines[second_label + 1].contains(" Z"));
    assert_eq!(lines[second_label + 2], "G1 Z.4");
}

#[tokio::test]
async fn task22o162_project_emits_filament_statistics() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    let footer = output.rsplit_once("; EXECUTABLE_BLOCK_END\n").unwrap().1;
    let values = footer
        .lines()
        .filter(|line| line.starts_with("; filament"))
        .map(|line| {
            line.split_once(" = ")
                .unwrap()
                .1
                .split_once(',')
                .unwrap()
                .0
                .parse::<f64>()
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 4);
    assert!(
        footer
            .lines()
            .filter(|line| line.starts_with("; filament"))
            .all(|line| line.ends_with(", 0.00"))
    );
    let filament_area = std::f64::consts::PI * (1.75_f64 * 0.5).powi(2);
    assert!((values[1] - values[0] * filament_area / 1_000.0).abs() <= 0.01);
    assert!((values[2] - values[1] * 1.26).abs() <= 0.02);
    assert!((values[3] - values[2] * 25.0 / 1_000.0).abs() <= 0.01);
}
