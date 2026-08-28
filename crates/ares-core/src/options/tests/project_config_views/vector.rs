use crate::{
    options::{
        Nullable, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, ProjectSettings,
        project_config_views::resolve_project_config_views,
    },
};

use super::support::clear_nullable_retract_overrides;

#[test]
fn project_config_views_mixed_deretraction_speed_preserves_full_and_owns_runtime_views() {
    let mut full = ProjectSettings::default();
    clear_nullable_retract_overrides(&mut full.filament.retract_overrides);
    full.project.gcode.deretraction_speed = floats(&[10.0, 20.0]);
    full.project.gcode.filament_map = ints(&[2, 1, 2]);
    full.filament
        .retract_overrides
        .filament_deretraction_speed = vec![
        Nullable::Nil,
        Nullable::Value(OrcaFloat(99.0)),
        Nullable::Nil,
    ];
    let expected_full = full.clone();

    let views = resolve_project_config_views(full).unwrap();

    assert_eq!(views.full, expected_full);
    assert_eq!(
        views.runtime.project.gcode.deretraction_speed,
        floats(&[20.0, 99.0, 20.0])
    );
    assert_eq!(
        views.runtime_gcode.deretraction_speed,
        views.runtime.project.gcode.deretraction_speed
    );
}

#[test]
fn project_config_views_value_deretraction_entries_ignore_invalid_maps_and_nil_is_one_based() {
    let mut full = ProjectSettings::default();
    clear_nullable_retract_overrides(&mut full.filament.retract_overrides);
    full.project.gcode.deretraction_speed = floats(&[10.0, 20.0]);
    full.project.gcode.filament_map = ints(&[0, -1, 99, 2]);
    full.filament
        .retract_overrides
        .filament_deretraction_speed = vec![
        Nullable::Value(OrcaFloat(91.0)),
        Nullable::Value(OrcaFloat(92.0)),
        Nullable::Value(OrcaFloat(93.0)),
        Nullable::Nil,
    ];

    let views = resolve_project_config_views(full).unwrap();

    assert_eq!(
        views.runtime.project.gcode.deretraction_speed,
        floats(&[91.0, 92.0, 93.0, 20.0])
    );
}

#[test]
fn project_config_views_invalid_nil_deretraction_maps_inherit_first_machine_value() {
    let mut full = ProjectSettings::default();
    clear_nullable_retract_overrides(&mut full.filament.retract_overrides);
    full.project.gcode.deretraction_speed = floats(&[10.0, 20.0]);
    full.project.gcode.filament_map = ints(&[0, -1, 99]);
    full.filament
        .retract_overrides
        .filament_deretraction_speed = vec![Nullable::Nil, Nullable::Nil, Nullable::Nil];

    let views = resolve_project_config_views(full).unwrap();

    assert_eq!(
        views.runtime.project.gcode.deretraction_speed,
        floats(&[10.0, 10.0, 10.0])
    );
}

#[test]
fn project_config_views_empty_machine_deretraction_speed_stays_empty() {
    let mut full = ProjectSettings::default();
    clear_nullable_retract_overrides(&mut full.filament.retract_overrides);
    full.project.gcode.deretraction_speed = OrcaFloats::default();
    full.project.gcode.filament_map = ints(&[1]);
    full.filament
        .retract_overrides
        .filament_deretraction_speed = vec![Nullable::Value(OrcaFloat(99.0))];

    let views = resolve_project_config_views(full).unwrap();

    assert_eq!(
        views.runtime.project.gcode.deretraction_speed,
        OrcaFloats::default()
    );
}

#[test]
fn project_config_views_empty_filament_deretraction_speed_preserves_machine_cardinality() {
    let mut full = ProjectSettings::default();
    clear_nullable_retract_overrides(&mut full.filament.retract_overrides);
    full.project.gcode.deretraction_speed = floats(&[10.0, 20.0]);
    full.project.gcode.filament_map = OrcaInts::default();

    let views = resolve_project_config_views(full).unwrap();

    assert_eq!(
        views.runtime.project.gcode.deretraction_speed,
        floats(&[10.0, 20.0])
    );
}

#[test]
fn project_config_views_nullable_override_cardinality_broadcasts() {
    let mut full = ProjectSettings::default();
    clear_nullable_retract_overrides(&mut full.filament.retract_overrides);
    full.project.gcode.deretraction_speed = floats(&[10.0, 20.0]);
    full.project.gcode.filament_map = ints(&[1, 2, 1]);
    full.filament
        .retract_overrides
        .filament_deretraction_speed = vec![Nullable::Value(OrcaFloat(30.0))];

    let views = resolve_project_config_views(full).unwrap();

    assert_eq!(
        views.runtime.project.gcode.deretraction_speed,
        floats(&[30.0, 30.0, 30.0])
    );
}

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}

fn ints(values: &[i32]) -> OrcaInts {
    OrcaInts(values.iter().copied().map(OrcaInt).collect())
}
