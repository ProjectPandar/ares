use crate::options::{
    GCodeOptions, OrcaBool, OrcaFloat, OrcaInt, OrcaString, Percent, ProjectSettings,
    RetractLiftEnforce, ZHopType, project_config_views::resolve_project_config_views,
};

use super::support::{
    assert_all_sixteen_runtime_fields, assert_all_twelve_runtime_gcode_fields, bools,
    clear_nullable_retract_overrides, floats, ints, nullable, percents, retract_lift_enforces,
    z_hop_types,
};

#[test]
fn project_config_views_all_sixteen_fields_overlay_and_preserve_full() {
    let mut full = ProjectSettings::default();
    clear_nullable_retract_overrides(&mut full.filament.retract_overrides);
    full.printer.gcode.enable_long_retraction_when_cut = OrcaInt(2);
    full.project.gcode.filament_map = ints(&[2, 1, 2]);
    full.printer.gcode.machine_end_gcode = OrcaString("unchanged sentinel".to_owned());
    full.printer.gcode.travel_slope = floats(&[51.0, 52.0]);

    full.project.gcode.deretraction_speed = floats(&[10.0, 20.0]);
    full.printer.gcode.long_retractions_when_cut = bools(&[false, true]);
    full.project.gcode.retract_before_wipe = percents(&[11.0, 21.0]);
    full.project.gcode.retract_lift_above = floats(&[12.0, 22.0]);
    full.project.gcode.retract_lift_below = floats(&[13.0, 23.0]);
    full.printer.gcode.retract_lift_enforce = retract_lift_enforces(&[
        RetractLiftEnforce::AllSurfaces,
        RetractLiftEnforce::TopOnly,
    ]);
    full.project.gcode.retract_restart_extra = floats(&[14.0, 24.0]);
    full.project.print.retract_when_changing_layer = bools(&[true, false]);
    full.printer.gcode.retraction_distances_when_cut = floats(&[15.0, 25.0]);
    full.project.gcode.retraction_length = floats(&[16.0, 26.0]);
    full.project.print.retraction_minimum_travel = floats(&[17.0, 27.0]);
    full.project.gcode.retraction_speed = floats(&[18.0, 28.0]);
    full.project.print.wipe = bools(&[true, false]);
    full.project.print.wipe_distance = floats(&[19.0, 29.0]);
    full.project.gcode.z_hop = floats(&[20.0, 30.0]);
    full.printer.gcode.z_hop_types = z_hop_types(&[ZHopType::Auto, ZHopType::Normal]);

    let overrides = &mut full.filament.retract_overrides;
    overrides.filament_deretraction_speed = nullable([
        None,
        Some(OrcaFloat(101.0)),
        Some(OrcaFloat(102.0)),
    ]);
    overrides.filament_long_retractions_when_cut = nullable([
        None,
        Some(OrcaBool(false)),
        Some(OrcaBool(true)),
    ]);
    overrides.filament_retract_before_wipe = nullable([
        None,
        Some(Percent(103.0)),
        Some(Percent(104.0)),
    ]);
    overrides.filament_retract_lift_above = nullable([
        None,
        Some(OrcaFloat(105.0)),
        Some(OrcaFloat(106.0)),
    ]);
    overrides.filament_retract_lift_below = nullable([
        None,
        Some(OrcaFloat(107.0)),
        Some(OrcaFloat(108.0)),
    ]);
    overrides.filament_retract_lift_enforce = nullable([
        None,
        Some(RetractLiftEnforce::BottomOnly),
        Some(RetractLiftEnforce::TopAndBottom),
    ]);
    overrides.filament_retract_restart_extra = nullable([
        None,
        Some(OrcaFloat(109.0)),
        Some(OrcaFloat(110.0)),
    ]);
    overrides.filament_retract_when_changing_layer = nullable([
        None,
        Some(OrcaBool(true)),
        Some(OrcaBool(false)),
    ]);
    overrides.filament_retraction_distances_when_cut = nullable([
        None,
        Some(OrcaFloat(111.0)),
        Some(OrcaFloat(112.0)),
    ]);
    overrides.filament_retraction_length = nullable([
        None,
        Some(OrcaFloat(113.0)),
        Some(OrcaFloat(114.0)),
    ]);
    overrides.filament_retraction_minimum_travel = nullable([
        None,
        Some(OrcaFloat(115.0)),
        Some(OrcaFloat(116.0)),
    ]);
    overrides.filament_retraction_speed = nullable([
        None,
        Some(OrcaFloat(117.0)),
        Some(OrcaFloat(118.0)),
    ]);
    overrides.filament_wipe = nullable([
        Some(OrcaBool(false)),
        None,
        Some(OrcaBool(true)),
    ]);
    overrides.filament_wipe_distance = nullable([
        None,
        Some(OrcaFloat(119.0)),
        Some(OrcaFloat(120.0)),
    ]);
    overrides.filament_z_hop = nullable([
        None,
        Some(OrcaFloat(121.0)),
        Some(OrcaFloat(122.0)),
    ]);
    overrides.filament_z_hop_types = nullable([
        None,
        Some(ZHopType::Spiral),
        Some(ZHopType::Slope),
    ]);

    let expected_full = full.clone();
    let unchanged_sentinel = full.printer.gcode.machine_end_gcode.clone();
    let mut expected_runtime = full.clone();
    expected_runtime.project.gcode.deretraction_speed = floats(&[20.0, 101.0, 102.0]);
    expected_runtime.printer.gcode.long_retractions_when_cut = bools(&[true, false, true]);
    expected_runtime.project.gcode.retract_before_wipe = percents(&[21.0, 103.0, 104.0]);
    expected_runtime.project.gcode.retract_lift_above = floats(&[22.0, 105.0, 106.0]);
    expected_runtime.project.gcode.retract_lift_below = floats(&[23.0, 107.0, 108.0]);
    expected_runtime.printer.gcode.retract_lift_enforce = retract_lift_enforces(&[
        RetractLiftEnforce::TopOnly,
        RetractLiftEnforce::BottomOnly,
        RetractLiftEnforce::TopAndBottom,
    ]);
    expected_runtime.project.gcode.retract_restart_extra = floats(&[24.0, 109.0, 110.0]);
    expected_runtime.project.print.retract_when_changing_layer = bools(&[false, true, false]);
    expected_runtime.printer.gcode.retraction_distances_when_cut =
        floats(&[25.0, 111.0, 112.0]);
    expected_runtime.project.gcode.retraction_length = floats(&[26.0, 113.0, 114.0]);
    expected_runtime.project.print.retraction_minimum_travel = floats(&[27.0, 115.0, 116.0]);
    expected_runtime.project.gcode.retraction_speed = floats(&[28.0, 117.0, 118.0]);
    expected_runtime.project.print.wipe = bools(&[false, true, true]);
    expected_runtime.project.print.wipe_distance = floats(&[29.0, 119.0, 120.0]);
    expected_runtime.project.gcode.z_hop = floats(&[30.0, 121.0, 122.0]);
    expected_runtime.printer.gcode.z_hop_types =
        z_hop_types(&[ZHopType::Normal, ZHopType::Spiral, ZHopType::Slope]);

    let views = resolve_project_config_views(full).unwrap();

    assert_eq!(views.full, expected_full);
    assert_all_sixteen_runtime_fields(&views.runtime, &expected_runtime);
    assert_all_twelve_runtime_gcode_fields(&views.runtime_gcode, &views.runtime);
    assert_eq!(
        views.runtime.filament.retract_overrides,
        expected_full.filament.retract_overrides
    );
    assert_eq!(
        views.runtime.printer.gcode.travel_slope,
        expected_full.printer.gcode.travel_slope
    );
    assert_eq!(
        views.runtime.printer.gcode.machine_end_gcode,
        unchanged_sentinel
    );
}

#[test]
fn project_config_views_inventory_contains_twelve_runtime_gcode_fields_and_omits_print_only() {
    let metadata = GCodeOptions::FIELD_METADATA;

    assert!(metadata.iter().any(|(_, key, _)| *key == "deretraction_speed"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "long_retractions_when_cut"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "retract_before_wipe"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "retract_lift_above"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "retract_lift_below"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "retract_lift_enforce"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "retract_restart_extra"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "retraction_distances_when_cut"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "retraction_length"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "retraction_speed"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "z_hop"));
    assert!(metadata.iter().any(|(_, key, _)| *key == "z_hop_types"));
    assert!(!metadata.iter().any(|(_, key, _)| *key == "retract_when_changing_layer"));
    assert!(!metadata.iter().any(|(_, key, _)| *key == "retraction_minimum_travel"));
    assert!(!metadata.iter().any(|(_, key, _)| *key == "wipe"));
    assert!(!metadata.iter().any(|(_, key, _)| *key == "wipe_distance"));
}
