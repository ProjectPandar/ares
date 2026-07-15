use crate::{Nullable, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, Percent, VariantStride};

use super::{assert_invalid_key, assert_unsupported_key, valid_project, valid_settings, validate};

#[test]
fn reports_unequal_physical_and_logical_counts() {
    let settings = valid_settings(3, 2);
    let project = valid_project();

    let validated = validate(&settings, &project).unwrap();

    assert_eq!(validated.physical_extruder_count, 3);
    assert_eq!(validated.logical_filament_count, 2);
}

#[test]
fn rejects_empty_physical_and_logical_cardinalities() {
    let project = valid_project();
    let mut settings = valid_settings(1, 1);
    settings.project.print.nozzle_diameter.0.clear();
    assert_invalid_key(validate(&settings, &project), "nozzle_diameter");

    let mut settings = valid_settings(1, 1);
    settings.filament.gcode.filament_diameter.0.clear();
    assert_invalid_key(validate(&settings, &project), "filament_diameter");
}

#[test]
fn rejects_filament_map_length_mismatch() {
    let project = valid_project();
    for entries in [vec![OrcaInt(1)], vec![OrcaInt(1), OrcaInt(2), OrcaInt(1)]] {
        let mut settings = valid_settings(2, 2);
        settings.project.gcode.filament_map = OrcaInts(entries);

        assert_invalid_key(validate(&settings, &project), "filament_map");
    }
}

#[test]
fn rejects_non_one_based_or_out_of_physical_filament_map_entries() {
    let project = valid_project();
    for entry in [0, 3] {
        let mut settings = valid_settings(2, 2);
        settings.project.gcode.filament_map.0[1] = OrcaInt(entry);

        assert_invalid_key(validate(&settings, &project), "filament_map");
    }
}

#[test]
fn rejects_each_short_filament_ironing_vector() {
    let project = valid_project();

    let mut settings = valid_settings(1, 2);
    settings.filament.region.filament_ironing_flow.pop();
    assert_invalid_key(validate(&settings, &project), "filament_ironing_flow");

    let mut settings = valid_settings(1, 2);
    settings.filament.region.filament_ironing_spacing.pop();
    assert_invalid_key(validate(&settings, &project), "filament_ironing_spacing");

    let mut settings = valid_settings(1, 2);
    settings.filament.region.filament_ironing_inset.pop();
    assert_invalid_key(validate(&settings, &project), "filament_ironing_inset");

    let mut settings = valid_settings(1, 2);
    settings.filament.region.filament_ironing_speed.pop();
    assert_invalid_key(validate(&settings, &project), "filament_ironing_speed");
}

#[test]
fn accepts_each_filament_ironing_vector_longer_than_logical_count() {
    let project = valid_project();

    let mut settings = valid_settings(1, 2);
    settings
        .filament
        .region
        .filament_ironing_flow
        .push(Nullable::Nil);
    validate(&settings, &project).unwrap();

    let mut settings = valid_settings(1, 2);
    settings
        .filament
        .region
        .filament_ironing_spacing
        .push(Nullable::Nil);
    validate(&settings, &project).unwrap();

    let mut settings = valid_settings(1, 2);
    settings
        .filament
        .region
        .filament_ironing_inset
        .push(Nullable::Nil);
    validate(&settings, &project).unwrap();

    let mut settings = valid_settings(1, 2);
    settings
        .filament
        .region
        .filament_ironing_speed
        .push(Nullable::Nil);
    validate(&settings, &project).unwrap();
}

#[test]
fn rejects_each_short_shrink_vector() {
    let project = valid_project();

    let mut settings = valid_settings(1, 2);
    settings.filament.print.filament_shrink.0.pop();
    assert_invalid_key(validate(&settings, &project), "filament_shrink");

    let mut settings = valid_settings(1, 2);
    settings
        .filament
        .print
        .filament_shrinkage_compensation_z
        .0
        .pop();
    assert_invalid_key(
        validate(&settings, &project),
        "filament_shrinkage_compensation_z",
    );
}

#[test]
fn rejects_each_active_non_default_shrink_value_as_unsupported() {
    let project = valid_project();

    let mut settings = valid_settings(1, 2);
    settings.filament.print.filament_shrink.0[1] = Percent(99.0);
    assert_unsupported_key(validate(&settings, &project), "filament_shrink");

    let mut settings = valid_settings(1, 2);
    settings.filament.print.filament_shrinkage_compensation_z.0[0] = Percent(101.0);
    assert_unsupported_key(
        validate(&settings, &project),
        "filament_shrinkage_compensation_z",
    );
}

#[test]
fn ignores_non_active_shrink_tail_entries() {
    let project = valid_project();
    let mut settings = valid_settings(2, 1);
    settings
        .filament
        .print
        .filament_shrink
        .0
        .push(Percent(95.0));
    settings
        .filament
        .print
        .filament_shrinkage_compensation_z
        .0
        .push(Percent(105.0));

    let validated = validate(&settings, &project).unwrap();

    assert_eq!(validated.logical_filament_count, 1);
}

#[test]
fn validation_preserves_settings_project_and_raw_variant_sentinels() {
    let mut settings = valid_settings(2, 2);
    settings.printer.machine.machine_max_acceleration_x = OrcaFloats(vec![
        OrcaFloat(401.0),
        OrcaFloat(402.0),
        OrcaFloat(403.0),
        OrcaFloat(404.0),
    ]);
    settings.filament.gcode.filament_extruder_variant = VariantStride(
        (1..=8)
            .map(|index| format!("raw-eight-stride-{index}"))
            .collect(),
    );
    let raw_four_stride = settings.printer.machine.machine_max_acceleration_x.clone();
    let raw_eight_stride = settings.filament.gcode.filament_extruder_variant.clone();
    let settings_before = settings.clone();
    let project = valid_project();
    let project_before = format!("{project:#?}");

    validate(&settings, &project).unwrap();

    assert_eq!(settings, settings_before);
    assert_eq!(format!("{project:#?}"), project_before);
    assert_eq!(
        settings.printer.machine.machine_max_acceleration_x,
        raw_four_stride
    );
    assert_eq!(
        settings.filament.gcode.filament_extruder_variant,
        raw_eight_stride
    );
}
