use crate::{
    load_project,
    options::{
        Nullable, OrcaFloat, OrcaInts, ProjectSettings, ZHopType,
        project_config_views::{ProjectConfigViews, resolve_project_config_views},
        project_variants::materialize_project_variants,
    },
};

use super::support::{assert_all_twelve_runtime_gcode_fields, floats, ints, z_hop_types};

#[test]
fn project_config_views_fixture_preserves_full_and_resolves_runtime_retracts() {
    let project = load_project(include_bytes!(
        "../../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf"
    ))
    .unwrap();
    let raw = project.settings().clone();
    let original_raw = raw.clone();
    let full = materialize_project_variants(&raw, &raw.project.gcode.filament_map).unwrap();

    let views = resolve_project_config_views(full).unwrap();

    assert_eq!(raw, original_raw);
    assert_eq!(
        views.full.project.gcode.deretraction_speed,
        floats(&[30.0, 20.0])
    );
    assert_eq!(
        views.runtime.project.gcode.deretraction_speed,
        floats(&[30.0, 30.0])
    );
    assert_eq!(
        views.full.printer.gcode.retraction_distances_when_cut,
        floats(&[18.0, 18.0])
    );
    assert_eq!(
        views.runtime.printer.gcode.retraction_distances_when_cut,
        floats(&[10.0, 10.0])
    );
    assert_eq!(
        views.full.project.gcode.retraction_length,
        floats(&[0.8, 2.0])
    );
    assert_eq!(
        views.runtime.project.gcode.retraction_length,
        floats(&[0.4, 0.4])
    );
    assert_eq!(
        views.full.project.gcode.retraction_speed,
        floats(&[30.0, 20.0])
    );
    assert_eq!(
        views.runtime.project.gcode.retraction_speed,
        floats(&[30.0, 30.0])
    );
    assert_eq!(
        views.full.project.print.wipe_distance,
        floats(&[2.0, 2.0])
    );
    assert_eq!(
        views.runtime.project.print.wipe_distance,
        floats(&[1.0, 1.0])
    );
    assert_eq!(
        views.full.printer.gcode.z_hop_types,
        z_hop_types(&[ZHopType::Auto, ZHopType::Auto])
    );
    assert_eq!(
        views.runtime.printer.gcode.z_hop_types,
        z_hop_types(&[ZHopType::Spiral, ZHopType::Spiral])
    );
    assert_all_twelve_runtime_gcode_fields(&views.runtime_gcode, &views.runtime);
}

#[test]
fn project_config_views_rematerializes_each_map_from_original_raw_source() {
    let project = load_project(include_bytes!(
        "../../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf"
    ))
    .unwrap();
    let raw = project.settings().clone();
    let original_raw = raw.clone();

    let map_a = ints(&[1, 1]);
    let map_b = ints(&[2, 1]);
    let first_a = resolve_from_raw(&raw, &map_a);
    let repeated_a = resolve_from_raw(&raw, &map_a);
    let result_b = resolve_from_raw(&raw, &map_b);

    assert_eq!(raw, original_raw);
    assert_eq!(first_a, repeated_a);
    assert_eq!(
        first_a.full.project.gcode.retraction_length,
        floats(&[0.8, 2.0])
    );
    assert_eq!(
        first_a
            .full
            .filament
            .retract_overrides
            .filament_retraction_length,
        vec![
            Nullable::Value(OrcaFloat(0.4)),
            Nullable::Value(OrcaFloat(0.4))
        ]
    );
    assert_eq!(
        first_a.runtime.project.gcode.retraction_length,
        floats(&[0.4, 0.4])
    );
    assert_eq!(
        first_a.runtime_gcode.retraction_length,
        floats(&[0.4, 0.4])
    );
    assert_eq!(
        result_b.full.project.gcode.retraction_length,
        floats(&[0.8, 2.0])
    );
    assert_eq!(
        result_b
            .full
            .filament
            .retract_overrides
            .filament_retraction_length,
        vec![
            Nullable::Value(OrcaFloat(3.0)),
            Nullable::Value(OrcaFloat(0.4))
        ]
    );
    assert_eq!(
        result_b.runtime.project.gcode.retraction_length,
        floats(&[3.0, 0.4])
    );
    assert_eq!(
        result_b.runtime_gcode.retraction_length,
        floats(&[3.0, 0.4])
    );
}

fn resolve_from_raw(raw: &ProjectSettings, filament_map: &OrcaInts) -> ProjectConfigViews {
    let full = materialize_project_variants(raw, filament_map).unwrap();
    resolve_project_config_views(full).unwrap()
}
