use crate::options::{
    Nullable, OrcaBool, OrcaFloat, OrcaInt, ProjectSettings,
    project_config_views::{ProjectConfigViews, resolve_project_config_views},
};

use super::support::{bools, clear_nullable_retract_overrides, floats, ints};

#[test]
fn project_config_views_long_retraction_gate_two_applies_concrete_overrides() {
    let views = resolve_with_long_retraction_gate(2);

    assert_eq!(
        views.runtime.printer.gcode.long_retractions_when_cut,
        bools(&[true, true, false])
    );
    assert_eq!(
        views.runtime.printer.gcode.retraction_distances_when_cut,
        floats(&[41.0, 42.0, 43.0])
    );
    assert_eq!(
        views.runtime.project.gcode.retraction_length,
        floats(&[2.0, 9.0, 2.0])
    );
}

#[test]
fn project_config_views_long_retraction_gate_zero_maps_nil_bools_and_preserves_distances() {
    let views = resolve_with_long_retraction_gate(0);

    assert_eq!(
        views.runtime.printer.gcode.long_retractions_when_cut,
        bools(&[true, false, true])
    );
    assert_eq!(
        views.runtime.printer.gcode.retraction_distances_when_cut,
        floats(&[18.0, 17.0])
    );
    assert_eq!(
        views.runtime.project.gcode.retraction_length,
        floats(&[2.0, 9.0, 2.0])
    );
}

#[test]
fn project_config_views_long_retraction_gate_one_maps_nil_bools_and_preserves_distances() {
    let views = resolve_with_long_retraction_gate(1);

    assert_eq!(
        views.runtime.printer.gcode.long_retractions_when_cut,
        bools(&[true, false, true])
    );
    assert_eq!(
        views.runtime.printer.gcode.retraction_distances_when_cut,
        floats(&[18.0, 17.0])
    );
    assert_eq!(
        views.runtime.project.gcode.retraction_length,
        floats(&[2.0, 9.0, 2.0])
    );
}

fn resolve_with_long_retraction_gate(gate: i32) -> ProjectConfigViews {
    let mut full = ProjectSettings::default();
    clear_nullable_retract_overrides(&mut full.filament.retract_overrides);
    full.printer.gcode.enable_long_retraction_when_cut = OrcaInt(gate);
    full.project.gcode.filament_map = ints(&[2, 1, 2]);
    full.printer.gcode.long_retractions_when_cut = bools(&[false, true]);
    full.printer.gcode.retraction_distances_when_cut = floats(&[18.0, 17.0]);
    full.project.gcode.retraction_length = floats(&[0.8, 2.0]);

    let overrides = &mut full.filament.retract_overrides;
    overrides.filament_long_retractions_when_cut = vec![
        Nullable::Value(OrcaBool(true)),
        Nullable::Value(OrcaBool(true)),
        Nullable::Value(OrcaBool(false)),
    ];
    overrides.filament_retraction_distances_when_cut = vec![
        Nullable::Value(OrcaFloat(41.0)),
        Nullable::Value(OrcaFloat(42.0)),
        Nullable::Value(OrcaFloat(43.0)),
    ];
    overrides.filament_retraction_length = vec![
        Nullable::Nil,
        Nullable::Value(OrcaFloat(9.0)),
        Nullable::Nil,
    ];

    resolve_project_config_views(full).unwrap()
}
