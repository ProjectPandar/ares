use super::*;

fn layer(print_z: f64, tools: &[usize], has_object: bool) -> LayerTools {
    LayerTools {
        print_z,
        extruders: tools.to_vec(),
        has_object,
        ..Default::default()
    }
}

#[test]
fn upper_tool_changes_support_lower_tower_partitions() {
    let mut layers = vec![
        layer(0.2, &[0], true),
        layer(0.4, &[0, 1], true),
        layer(0.6, &[1], true),
    ];
    fill(&mut layers, &ProjectSettings::default(), 0.0, 0.3);
    assert_eq!(
        layers
            .iter()
            .map(|l| l.wipe_tower_partitions)
            .collect::<Vec<_>>(),
        [1, 1, 0]
    );
    assert_eq!(
        layers.iter().map(|l| l.has_wipe_tower).collect::<Vec<_>>(),
        [true, true, false]
    );
    assert_eq!(
        layers
            .iter()
            .map(|l| l.wipe_tower_layer_height)
            .collect::<Vec<_>>(),
        [0.2, 0.2, 0.0]
    );
}

#[test]
fn smooth_same_tool_layers_have_no_change_partitions() {
    let mut settings = ProjectSettings::default();
    settings.process.print.timelapse_type = crate::ProcessTimelapseType::Smooth;
    let mut layers = vec![layer(0.2, &[3], true), layer(0.4, &[3], true)];
    fill(&mut layers, &settings, 0.0, 0.3);
    assert!(
        layers
            .iter()
            .all(|l| l.has_wipe_tower && l.wipe_tower_partitions == 0)
    );
    assert_eq!(layers[1].wipe_tower_layer_height, 0.2);
}

#[test]
fn raft_gap_inserts_midpoint_with_the_following_tool() {
    let mut settings = ProjectSettings::default();
    settings.process.print.timelapse_type = crate::ProcessTimelapseType::Smooth;
    let mut layers = vec![layer(0.2, &[2], false), layer(1.0, &[2], true)];
    fill(&mut layers, &settings, 0.8, 0.3);
    assert_eq!(layers.len(), 3);
    assert_eq!(layers[1].print_z, 0.6);
    assert_eq!(layers[1].extruders, [2]);
    assert!(!layers[1].has_object);
    assert!(layers.iter().all(|l| l.has_wipe_tower));
    assert!((layers[1].wipe_tower_layer_height - 0.4).abs() < 1e-12);
}

#[test]
fn raft_gap_marks_an_existing_midpoint_instead_of_duplicating_it() {
    let mut settings = ProjectSettings::default();
    settings.process.print.timelapse_type = crate::ProcessTimelapseType::Smooth;
    let mut layers = vec![
        layer(0.2, &[2], false),
        layer(0.8, &[2], false),
        layer(1.4, &[2], true),
    ];
    fill(&mut layers, &settings, 0.75, 0.7);
    assert_eq!(layers.len(), 3);
    assert!(layers[1].has_wipe_tower);
    assert!((layers[1].wipe_tower_layer_height - 0.6).abs() < 1e-12);
}

#[test]
fn support_only_tool_change_requires_a_tower() {
    let mut layers = vec![layer(0.2, &[0], true), layer(0.4, &[1], false)];
    fill(&mut layers, &ProjectSettings::default(), 0.0, 0.3);
    assert!(layers[1].has_wipe_tower);
    assert_eq!(layers[1].wipe_tower_partitions, 1);
}

#[test]
fn tower_gap_is_installed_in_the_first_objects_support_schedule() {
    use super::super::{InstalledSupportLayer, ToolOrdering};
    let mut settings = ProjectSettings::default();
    settings.process.print.timelapse_type = crate::ProcessTimelapseType::Smooth;
    let mut layers = vec![layer(0.2, &[2], false), layer(1.0, &[2], true)];
    layers[0].has_support = true;
    fill(&mut layers, &settings, 0.8, 0.3);
    let mut ordering = ToolOrdering {
        layers,
        all_extruders: vec![2],
        first_extruder: Some(2),
        tower_type: Some(crate::WipeTowerType::Type2),
    };
    let mut raft = InstalledSupportLayer::for_wipe_tower(0.2, 0.2);
    raft.id = 0;
    let mut support = vec![vec![raft], Vec::new()];
    ordering.install_tower_support_layers(&mut support);
    assert_eq!(support[0].len(), 2);
    assert_eq!(support[0][1].id, usize::MAX);
    assert_eq!(support[0][1].print_z, 0.6);
    assert!((support[0][1].height - 0.4).abs() < 1e-12);
    assert!(support[0][1].support_fills.is_empty());
    assert!(support[1].is_empty());
    assert!(ordering.layers[1].has_support);
}

#[test]
fn selected_tower_filament_is_added_only_to_partitioned_layers() {
    let mut settings = ProjectSettings::default();
    settings.process.print.enable_prime_tower.0 = true;
    settings.process.print.wipe_tower_filament.0 = 3;
    let mut layers = vec![layer(0.2, &[0, 1], true), layer(0.4, &[1], true)];
    fill(&mut layers, &settings, 0.0, 0.3);
    assert!(insert_extruder(&mut layers, &settings));
    assert_eq!(layers[0].extruders, [0, 1, 2]);
    assert_eq!(layers[1].extruders, [1]);
    assert!(!insert_extruder(&mut layers, &settings));
}
