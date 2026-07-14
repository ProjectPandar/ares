use crate::options::{
    FilamentRetractOverrideOptions, GCodeOptions, Nullable, OrcaBool, OrcaBools, OrcaFloat,
    OrcaFloats, OrcaInt, OrcaInts, OrcaPercents, Percent, ProjectSettings, RetractLiftEnforce,
    RetractLiftEnforces, ZHopType, ZHopTypes,
};

pub(super) fn clear_nullable_retract_overrides(
    overrides: &mut FilamentRetractOverrideOptions,
) {
    overrides.filament_retraction_length.clear();
    overrides.filament_z_hop.clear();
    overrides.filament_z_hop_types.clear();
    overrides.filament_retract_lift_above.clear();
    overrides.filament_retract_lift_below.clear();
    overrides.filament_retract_lift_enforce.clear();
    overrides.filament_retraction_speed.clear();
    overrides.filament_deretraction_speed.clear();
    overrides.filament_retract_restart_extra.clear();
    overrides.filament_retraction_minimum_travel.clear();
    overrides.filament_wipe_distance.clear();
    overrides.filament_retract_when_changing_layer.clear();
    overrides.filament_wipe.clear();
    overrides.filament_retract_before_wipe.clear();
    overrides.filament_long_retractions_when_cut.clear();
    overrides
        .filament_retraction_distances_when_cut
        .clear();
}

pub(super) fn assert_all_sixteen_runtime_fields(
    actual: &ProjectSettings,
    expected: &ProjectSettings,
) {
    assert_eq!(
        actual.project.gcode.deretraction_speed,
        expected.project.gcode.deretraction_speed
    );
    assert_eq!(
        actual.printer.gcode.long_retractions_when_cut,
        expected.printer.gcode.long_retractions_when_cut
    );
    assert_eq!(
        actual.project.gcode.retract_before_wipe,
        expected.project.gcode.retract_before_wipe
    );
    assert_eq!(
        actual.project.gcode.retract_lift_above,
        expected.project.gcode.retract_lift_above
    );
    assert_eq!(
        actual.project.gcode.retract_lift_below,
        expected.project.gcode.retract_lift_below
    );
    assert_eq!(
        actual.printer.gcode.retract_lift_enforce,
        expected.printer.gcode.retract_lift_enforce
    );
    assert_eq!(
        actual.project.gcode.retract_restart_extra,
        expected.project.gcode.retract_restart_extra
    );
    assert_eq!(
        actual.project.print.retract_when_changing_layer,
        expected.project.print.retract_when_changing_layer
    );
    assert_eq!(
        actual.printer.gcode.retraction_distances_when_cut,
        expected.printer.gcode.retraction_distances_when_cut
    );
    assert_eq!(
        actual.project.gcode.retraction_length,
        expected.project.gcode.retraction_length
    );
    assert_eq!(
        actual.project.print.retraction_minimum_travel,
        expected.project.print.retraction_minimum_travel
    );
    assert_eq!(
        actual.project.gcode.retraction_speed,
        expected.project.gcode.retraction_speed
    );
    assert_eq!(actual.project.print.wipe, expected.project.print.wipe);
    assert_eq!(
        actual.project.print.wipe_distance,
        expected.project.print.wipe_distance
    );
    assert_eq!(actual.project.gcode.z_hop, expected.project.gcode.z_hop);
    assert_eq!(
        actual.printer.gcode.z_hop_types,
        expected.printer.gcode.z_hop_types
    );
}

pub(super) fn assert_all_twelve_runtime_gcode_fields(
    runtime_gcode: &GCodeOptions,
    runtime: &ProjectSettings,
) {
    assert_eq!(
        runtime_gcode.deretraction_speed,
        runtime.project.gcode.deretraction_speed
    );
    assert_eq!(
        runtime_gcode.long_retractions_when_cut,
        runtime.printer.gcode.long_retractions_when_cut
    );
    assert_eq!(
        runtime_gcode.retract_before_wipe,
        runtime.project.gcode.retract_before_wipe
    );
    assert_eq!(
        runtime_gcode.retract_lift_above,
        runtime.project.gcode.retract_lift_above
    );
    assert_eq!(
        runtime_gcode.retract_lift_below,
        runtime.project.gcode.retract_lift_below
    );
    assert_eq!(
        runtime_gcode.retract_lift_enforce,
        runtime.printer.gcode.retract_lift_enforce
    );
    assert_eq!(
        runtime_gcode.retract_restart_extra,
        runtime.project.gcode.retract_restart_extra
    );
    assert_eq!(
        runtime_gcode.retraction_distances_when_cut,
        runtime.printer.gcode.retraction_distances_when_cut
    );
    assert_eq!(
        runtime_gcode.retraction_length,
        runtime.project.gcode.retraction_length
    );
    assert_eq!(
        runtime_gcode.retraction_speed,
        runtime.project.gcode.retraction_speed
    );
    assert_eq!(runtime_gcode.z_hop, runtime.project.gcode.z_hop);
    assert_eq!(
        runtime_gcode.z_hop_types,
        runtime.printer.gcode.z_hop_types
    );
}

pub(super) fn nullable<T>(values: [Option<T>; 3]) -> Vec<Nullable<T>> {
    values
        .into_iter()
        .map(|value| value.map_or(Nullable::Nil, Nullable::Value))
        .collect()
}

pub(super) fn bools(values: &[bool]) -> OrcaBools {
    OrcaBools(values.iter().copied().map(OrcaBool).collect())
}

pub(super) fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}

pub(super) fn ints(values: &[i32]) -> OrcaInts {
    OrcaInts(values.iter().copied().map(OrcaInt).collect())
}

pub(super) fn percents(values: &[f64]) -> OrcaPercents {
    OrcaPercents(values.iter().copied().map(Percent).collect())
}

pub(super) fn retract_lift_enforces(values: &[RetractLiftEnforce]) -> RetractLiftEnforces {
    RetractLiftEnforces(values.to_vec())
}

pub(super) fn z_hop_types(values: &[ZHopType]) -> ZHopTypes {
    ZHopTypes(values.to_vec())
}
