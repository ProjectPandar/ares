use crate::{
    ProjectBedType, load_project,
    options::config_export::{
        collector::{ConfigEntry, collect_config_entries},
        write_config_block,
    },
    project::effective_config::resolve_bounded_project_config,
};

use super::assignment_lines;

const FULL_SENTINELS: [&str; 6] = [
    "deretraction_speed",
    "retraction_distances_when_cut",
    "retraction_length",
    "retraction_speed",
    "wipe_distance",
    "z_hop_types",
];

#[test]
fn config_export_fixture_keeps_resolved_task19b3_views_self_consistent() {
    let project = load_project(crate::test_fixtures::ksr_project()).unwrap();
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

fn entry_token<'a>(entries: &'a [ConfigEntry], key: &str) -> &'a str {
    entries
        .iter()
        .find(|entry| entry.key == key)
        .unwrap()
        .token
        .as_str()
}
