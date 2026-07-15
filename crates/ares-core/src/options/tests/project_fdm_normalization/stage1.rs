use crate::options::{
    Nullable, OrcaBool, OrcaBools, OrcaFloat, OrcaFloats, OrcaInt, OrcaString, Percent,
    ProjectSettings, project_fdm_normalization::normalize_fdm_1,
};

#[test]
fn sparse_selector_fills_zero_internal_and_original_internal_fills_zero_surfaces() {
    let mut settings = sentinel_settings();
    settings.process.region.sparse_infill_filament_id = OrcaInt(4);
    settings.process.region.internal_solid_filament_id = OrcaInt(0);
    settings.process.region.top_surface_filament_id = OrcaInt(0);
    settings.process.region.bottom_surface_filament_id = OrcaInt(0);
    let original = settings.clone();

    normalize_fdm_1(&mut settings);

    assert_eq!(
        settings.process.region.internal_solid_filament_id,
        OrcaInt(4)
    );
    assert_eq!(settings.process.region.top_surface_filament_id, OrcaInt(4));
    assert_eq!(
        settings.process.region.bottom_surface_filament_id,
        OrcaInt(4)
    );
    assert_only_stage1_write_set_changed(&original, &settings);
}

#[test]
fn original_snapshot_makes_bottom_overwrite_top_for_zero_internal() {
    let mut settings = sentinel_settings();
    settings.process.region.sparse_infill_filament_id = OrcaInt(0);
    settings.process.region.internal_solid_filament_id = OrcaInt(0);
    settings.process.region.top_surface_filament_id = OrcaInt(2);
    settings.process.region.bottom_surface_filament_id = OrcaInt(3);
    let original = settings.clone();

    normalize_fdm_1(&mut settings);

    assert_eq!(
        settings.process.region.internal_solid_filament_id,
        OrcaInt(3)
    );
    assert_eq!(settings.process.region.top_surface_filament_id, OrcaInt(2));
    assert_eq!(
        settings.process.region.bottom_surface_filament_id,
        OrcaInt(3)
    );
    assert_only_stage1_write_set_changed(&original, &settings);
}

#[test]
fn original_internal_fills_both_zero_surface_selectors() {
    let mut settings = sentinel_settings();
    settings.process.region.sparse_infill_filament_id = OrcaInt(-2);
    settings.process.region.internal_solid_filament_id = OrcaInt(7);
    settings.process.region.top_surface_filament_id = OrcaInt(0);
    settings.process.region.bottom_surface_filament_id = OrcaInt(0);
    let original = settings.clone();

    normalize_fdm_1(&mut settings);

    assert_eq!(
        settings.process.region.internal_solid_filament_id,
        OrcaInt(7)
    );
    assert_eq!(settings.process.region.top_surface_filament_id, OrcaInt(7));
    assert_eq!(
        settings.process.region.bottom_surface_filament_id,
        OrcaInt(7)
    );
    assert_only_stage1_write_set_changed(&original, &settings);
}

#[test]
fn propagation_preserves_nonzero_selector_destinations() {
    let mut settings = sentinel_settings();
    set_nonzero_selectors(&mut settings);
    let original = settings.clone();

    normalize_fdm_1(&mut settings);

    assert_eq!(settings, original);
}

#[test]
fn spiral_mode_preserves_vector_lengths_and_writes_only_the_fixed_set() {
    let mut settings = sentinel_settings();
    set_nonzero_selectors(&mut settings);
    settings.process.print.spiral_mode = OrcaBool(true);
    settings.project.print.retract_when_changing_layer = bools(&[true, false, true]);
    settings
        .filament
        .retract_overrides
        .filament_retract_when_changing_layer = vec![
        Nullable::Nil,
        Nullable::Value(OrcaBool(true)),
        Nullable::Value(OrcaBool(false)),
        Nullable::Nil,
    ];
    settings.process.region.wall_loops = OrcaInt(8);
    settings.process.region.alternate_extra_wall = OrcaBool(true);
    settings.process.region.top_shell_layers = OrcaInt(6);
    settings.process.region.sparse_infill_density = Percent(42.0);
    settings.process.print.resolution = OrcaFloat(0.25);
    let original = settings.clone();

    normalize_fdm_1(&mut settings);

    assert_eq!(
        settings.project.print.retract_when_changing_layer,
        bools(&[false, false, false])
    );
    assert_eq!(
        settings
            .filament
            .retract_overrides
            .filament_retract_when_changing_layer,
        vec![Nullable::Value(OrcaBool(false)); 4]
    );
    assert_eq!(settings.process.region.wall_loops, OrcaInt(1));
    assert_eq!(
        settings.process.region.alternate_extra_wall,
        OrcaBool(false)
    );
    assert_eq!(settings.process.region.top_shell_layers, OrcaInt(0));
    assert_eq!(settings.process.region.sparse_infill_density, Percent(0.0));
    assert_eq!(settings.process.print.resolution, OrcaFloat(0.25));
    assert_only_stage1_write_set_changed(&original, &settings);
}

#[test]
fn disabled_spiral_mode_preserves_spiral_controlled_fields() {
    let mut settings = sentinel_settings();
    set_nonzero_selectors(&mut settings);
    settings.process.print.spiral_mode = OrcaBool(false);
    settings.project.print.retract_when_changing_layer = bools(&[true, false]);
    settings
        .filament
        .retract_overrides
        .filament_retract_when_changing_layer =
        vec![Nullable::Nil, Nullable::Value(OrcaBool(true))];
    settings.process.region.wall_loops = OrcaInt(9);
    settings.process.region.alternate_extra_wall = OrcaBool(true);
    settings.process.region.top_shell_layers = OrcaInt(7);
    settings.process.region.sparse_infill_density = Percent(31.0);
    settings.process.print.resolution = OrcaFloat(0.5);
    let original = settings.clone();

    normalize_fdm_1(&mut settings);

    assert_eq!(settings, original);
}

#[test]
fn resolution_is_clamped_for_negative_below_equal_and_above_inputs() {
    for (input, expected) in [
        (-0.5, 0.001),
        (0.0005, 0.001),
        (0.001, 0.001),
        (0.2, 0.2),
    ] {
        let mut settings = sentinel_settings();
        set_nonzero_selectors(&mut settings);
        settings.process.print.resolution = OrcaFloat(input);
        let original = settings.clone();

        normalize_fdm_1(&mut settings);

        assert_eq!(settings.process.print.resolution, OrcaFloat(expected));
        assert_only_stage1_write_set_changed(&original, &settings);
    }
}

fn sentinel_settings() -> ProjectSettings {
    let mut settings = ProjectSettings::default();
    settings.printer.machine.machine_max_acceleration_e = floats(&[77.0, 78.0]);
    settings.process.print.notes = OrcaString("stage-one-sentinel".to_owned());
    settings.process.region.bottom_shell_layers = OrcaInt(13);
    settings.filament.pellet_flow_coefficient = floats(&[0.41, 0.73]);
    settings.project.print.flush_multiplier = floats(&[0.31, 0.62]);
    settings.metadata.name = "stage-one-metadata".to_owned();
    settings
}

fn set_nonzero_selectors(settings: &mut ProjectSettings) {
    settings.process.region.sparse_infill_filament_id = OrcaInt(9);
    settings.process.region.internal_solid_filament_id = OrcaInt(4);
    settings.process.region.top_surface_filament_id = OrcaInt(5);
    settings.process.region.bottom_surface_filament_id = OrcaInt(6);
}

fn assert_only_stage1_write_set_changed(original: &ProjectSettings, actual: &ProjectSettings) {
    let mut restored = actual.clone();
    restored.process.region.internal_solid_filament_id =
        original.process.region.internal_solid_filament_id;
    restored.process.region.top_surface_filament_id = original.process.region.top_surface_filament_id;
    restored.process.region.bottom_surface_filament_id =
        original.process.region.bottom_surface_filament_id;
    restored.project.print.retract_when_changing_layer =
        original.project.print.retract_when_changing_layer.clone();
    restored
        .filament
        .retract_overrides
        .filament_retract_when_changing_layer = original
        .filament
        .retract_overrides
        .filament_retract_when_changing_layer
        .clone();
    restored.process.region.wall_loops = original.process.region.wall_loops;
    restored.process.region.alternate_extra_wall = original.process.region.alternate_extra_wall;
    restored.process.region.top_shell_layers = original.process.region.top_shell_layers;
    restored.process.region.sparse_infill_density = original.process.region.sparse_infill_density;
    restored.process.print.resolution = original.process.print.resolution;

    assert_eq!(&restored, original);
}

fn bools(values: &[bool]) -> OrcaBools {
    OrcaBools(values.iter().copied().map(OrcaBool).collect())
}

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}
