use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use crate::{
    ProjectBedType, load_project,
    options::config_export::{
        collector::{ConfigEntry, collect_config_entries},
        write_config_block,
    },
    project::effective_config::resolve_bounded_project_config,
};

use super::assignment_lines;

const PROJECT: &[u8] = include_bytes!(
    "../../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf"
);
const REFERENCE: &[u8] =
    include_bytes!("../../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode");
const START: &[u8] = b"; CONFIG_BLOCK_START\n";
const END: &[u8] = b"; CONFIG_BLOCK_END\n\n";
const BLOCK_SHA256: &str = "6a14052054bfd46b7ea0268917246f8ce0c6388936351816fc4b41a5a6cd6706";

const NIL_OMISSIONS: [&str; 15] = [
    "filament_deretraction_speed",
    "filament_ironing_flow",
    "filament_ironing_inset",
    "filament_ironing_spacing",
    "filament_ironing_speed",
    "filament_long_retractions_when_cut",
    "filament_retract_before_wipe",
    "filament_retract_lift_above",
    "filament_retract_lift_below",
    "filament_retract_lift_enforce",
    "filament_retract_restart_extra",
    "filament_retract_when_changing_layer",
    "filament_retraction_minimum_travel",
    "filament_retraction_speed",
    "filament_z_hop",
];

const RETAINED_EMPTY: [&str; 5] = [
    "bed_exclude_area",
    "head_wrap_detect_zone",
    "parallel_printheads_bed_exclude_areas",
    "post_process",
    "wrapping_exclude_area",
];

const FULL_SENTINELS: [&str; 6] = [
    "deretraction_speed",
    "retraction_distances_when_cut",
    "retraction_length",
    "retraction_speed",
    "wipe_distance",
    "z_hop_types",
];

#[test]
fn config_export_fixture_freezes_the_committed_reference_block_contract() {
    let block = reference_block();
    assert_eq!(block.len(), 49_005);
    assert_eq!(sha256(block), BLOCK_SHA256);
    assert!(block.starts_with(START));
    assert!(block.ends_with(END));
    assert!(!block.contains(&b'\r'));

    let assignments = assignments(block);
    assert_eq!(assignments.len(), 639);
    assert_eq!(
        assignments
            .iter()
            .map(|(key, _)| *key)
            .collect::<BTreeSet<_>>()
            .len(),
        637
    );
    assert_eq!(
        assignment_lines(block, "wipe_tower_x"),
        ["; wipe_tower_x = 165.000", "; wipe_tower_x = 165"]
    );
    assert_eq!(
        assignment_lines(block, "wipe_tower_y"),
        ["; wipe_tower_y = 220.096", "; wipe_tower_y = 220.096"]
    );
    for key in ["from", "name", "version"].into_iter().chain(NIL_OMISSIONS) {
        assert!(assignments.iter().all(|(actual, _)| *actual != key), "{key}");
    }
    for key in RETAINED_EMPTY {
        assert!(
            assignments
                .iter()
                .any(|(actual, value)| *actual == key && value.is_empty()),
            "{key}"
        );
    }
    assert_eq!(
        assignments[assignments.len() - 2..],
        [
            ("first_layer_bed_temperature", "55"),
            ("first_layer_temperature", "220"),
        ]
    );
}

#[test]
fn config_export_fixture_matches_resolved_task19b3_views_byte_for_byte() {
    let project = load_project(PROJECT).unwrap();
    let resolved = resolve_bounded_project_config(&project).unwrap();
    let full_entries = collect_config_entries(&resolved.views.full).unwrap();
    let runtime_entries = collect_config_entries(&resolved.views.runtime).unwrap();
    let mut actual = Vec::new();

    write_config_block(
        &resolved.views,
        &project.documents().project_settings_raw,
        0,
        &mut actual,
    )
    .unwrap();

    let expected = reference_block();
    if actual != expected {
        let index = actual
            .iter()
            .zip(expected)
            .position(|(actual, expected)| actual != expected)
            .unwrap_or(actual.len().min(expected.len()));
        panic!(
            "config block differs at byte {index}: expected {} bytes, got {} bytes\nexpected: {:?}\nactual: {:?}",
            expected.len(),
            actual.len(),
            bounded_text(expected, index),
            bounded_text(&actual, index),
        );
    }
    for key in FULL_SENTINELS {
        let full = entry_token(&full_entries, key);
        let runtime = entry_token(&runtime_entries, key);
        assert_ne!(full, runtime, "fixture sentinel {key} must distinguish views");
        assert_eq!(assignment_lines(&actual, key), [format!("; {key} = {full}")]);
    }
    let runtime = &resolved.views.runtime;
    let bed = match runtime.project.print.curr_bed_type {
        ProjectBedType::SupertackPlate => &runtime.filament.print.supertack_plate_temp_initial_layer,
        ProjectBedType::CoolPlate => &runtime.filament.print.cool_plate_temp_initial_layer,
        ProjectBedType::TexturedCoolPlate => {
            &runtime.filament.print.textured_cool_plate_temp_initial_layer
        }
        ProjectBedType::EngineeringPlate => &runtime.filament.print.eng_plate_temp_initial_layer,
        ProjectBedType::HighTempPlate => &runtime.filament.print.hot_plate_temp_initial_layer,
        ProjectBedType::TexturedPeiPlate => &runtime.filament.print.textured_plate_temp_initial_layer,
        ProjectBedType::DefaultPlate => panic!("fixture uses Default Plate"),
    };
    assert_eq!(bed.0[0].0, 55);
    assert_eq!(runtime.filament.print.nozzle_temperature_initial_layer.0[0].0, 220);
}

fn reference_block() -> &'static [u8] {
    let start = find_once(REFERENCE, START);
    let end = start + find_once(&REFERENCE[start..], END) + END.len();
    &REFERENCE[start..end]
}

fn find_once(haystack: &[u8], needle: &[u8]) -> usize {
    let matches = haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, window)| (window == needle).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "marker occurrence count");
    matches[0]
}

fn assignments(block: &[u8]) -> Vec<(&str, &str)> {
    std::str::from_utf8(block)
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix("; ")?.split_once(" = "))
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn bounded_text(bytes: &[u8], index: usize) -> String {
    let start = index.saturating_sub(80);
    let end = (index + 80).min(bytes.len());
    String::from_utf8_lossy(&bytes[start..end]).into_owned()
}

fn entry_token<'a>(entries: &'a [ConfigEntry], key: &str) -> &'a str {
    entries
        .iter()
        .find(|entry| entry.key == key)
        .unwrap()
        .token
        .as_str()
}
