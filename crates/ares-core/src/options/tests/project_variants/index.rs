use crate::options::{
    ExtruderType, ExtruderTypes, ExtruderVariantLists, NozzleVolumeType, NozzleVolumeTypes,
    OrcaFloats, OrcaStrings, VariantStride,
    project_variants::{inspect_printer_indices_for_test, materialize_project_variants},
};

use super::support::{active_source, assert_invalid_key, ints, one_extruder_source};

#[test]
fn complete_id_map_uses_first_exact_pair_and_allows_trailing_ids() {
    let mut source = active_source();
    source.printer.gcode.printer_extruder_id = ints(&[1, 1, 2, 99]);
    source.printer.gcode.printer_extruder_variant = OrcaStrings(vec![
        "Direct Drive Standard".to_owned(),
        "Direct Drive Standard".to_owned(),
        "Bowden Standard".to_owned(),
    ]);
    let original = source.clone();

    materialize_project_variants(&source, &ints(&[1, 2])).unwrap();
    let indices = inspect_printer_indices_for_test(&source).unwrap();

    assert_eq!(indices.unwrap(), vec![0, 2]);
    assert_eq!(source, original);
}

#[test]
fn generated_id_map_compresses_trims_and_skips_empty_tokens() {
    let mut source = active_source();
    source.printer.remaining.extruder_variant_list = ExtruderVariantLists(vec![
        " Direct Drive Standard ,, Direct Drive High Flow ".to_owned(),
        " , Bowden Standard ,, ".to_owned(),
    ]);
    source.printer.gcode.printer_extruder_id = ints(&[999]);
    source.printer.gcode.printer_extruder_variant = OrcaStrings(vec![
        "Direct Drive Standard".to_owned(),
        "Direct Drive High Flow".to_owned(),
        "Bowden Standard".to_owned(),
    ]);

    let indices = inspect_printer_indices_for_test(&source).unwrap();

    assert_eq!(indices.unwrap(), vec![0, 2]);
}

#[test]
fn guard_uses_only_physical_groups_repeats_first_and_ignores_trailing() {
    let mut repeated = active_source();
    repeated.printer.remaining.extruder_variant_list =
        ExtruderVariantLists(vec!["Direct Drive Standard".to_owned()]);
    materialize_project_variants(&repeated, &ints(&[1, 2])).unwrap();
    let indices = inspect_printer_indices_for_test(&repeated).unwrap();
    assert_eq!(indices.unwrap(), vec![0, 1]);

    let mut trailing = one_extruder_source();
    trailing
        .printer
        .remaining
        .extruder_variant_list
        .0
        .push("Bowden Standard".to_owned());
    materialize_project_variants(&trailing, &ints(&[0, 99])).unwrap();
    let indices = inspect_printer_indices_for_test(&trailing).unwrap();
    assert_eq!(indices, None);
}

#[test]
fn one_shared_variant_broadcasts_across_identical_physical_extruders() {
    let mut source = active_source();
    source.project.print.nozzle_diameter = OrcaFloats(vec![
        crate::OrcaFloat(0.4),
        crate::OrcaFloat(0.4),
        crate::OrcaFloat(0.4),
        crate::OrcaFloat(0.4),
    ]);
    source.printer.remaining.extruder_variant_list =
        ExtruderVariantLists(vec!["Direct Drive Standard".to_owned()]);
    source.printer.gcode.extruder_type = ExtruderTypes(vec![ExtruderType::DirectDrive]);
    source.project.gcode.nozzle_volume_type =
        NozzleVolumeTypes(vec![NozzleVolumeType::Standard]);
    source.printer.gcode.printer_extruder_id = ints(&[1]);
    source.printer.gcode.printer_extruder_variant =
        OrcaStrings(vec!["Direct Drive Standard".to_owned()]);
    source.process.region.print_extruder_id = ints(&[1]);
    source.process.region.print_extruder_variant =
        OrcaStrings(vec!["Direct Drive Standard".to_owned()]);
    source.project.preset.filament_self_index = ints(&[1]);
    source.filament.gcode.filament_extruder_variant =
        VariantStride(vec!["Direct Drive Standard".to_owned()]);
    source.project.gcode.filament_map = ints(&[1]);

    let indices = inspect_printer_indices_for_test(&source).unwrap().unwrap();
    let once = materialize_project_variants(&source, &ints(&[1])).unwrap();
    let twice = materialize_project_variants(&once, &ints(&[1])).unwrap();

    assert_eq!(indices, [0, 0, 0, 0]);
    assert_eq!(twice, once);
}

#[test]
fn guard_preserves_edge_empty_and_whitespace_tokens() {
    for group in [
        ",,Direct Drive Standard,,,",
        "Direct Drive Standard, Direct Drive Standard",
    ] {
        let mut source = one_extruder_source();
        source.printer.remaining.extruder_variant_list =
            ExtruderVariantLists(vec![group.to_owned()]);
        source.printer.gcode.printer_extruder_variant.0.clear();
        assert_invalid_key(
            materialize_project_variants(&source, &ints(&[1])),
            "printer_extruder_variant",
        );
    }
}

#[test]
fn one_extruder_multiple_variants_activates_materialization() {
    let mut source = one_extruder_source();
    source.printer.remaining.extruder_variant_list = ExtruderVariantLists(vec![
        "Direct Drive Standard,Bowden Standard".to_owned(),
    ]);
    source.project.gcode.filament_map = ints(&[99]);
    let original = source.clone();

    let materialized = materialize_project_variants(&source, &ints(&[1])).unwrap();
    let indices = inspect_printer_indices_for_test(&source).unwrap();

    assert_eq!(indices.unwrap(), vec![0]);
    assert_eq!(materialized.project.gcode.filament_map, ints(&[1]));
    assert_eq!(source, original);
}

#[test]
fn one_physical_extruder_map_broadcasts_to_all_logical_filaments() {
    let mut source = one_extruder_source();
    source.filament.gcode.filament_diameter = OrcaFloats(vec![
        crate::OrcaFloat(1.75),
        crate::OrcaFloat(1.75),
        crate::OrcaFloat(1.75),
    ]);

    let materialized = materialize_project_variants(&source, &ints(&[1])).unwrap();

    assert_eq!(materialized.project.gcode.filament_map, ints(&[1, 1, 1]));
}

#[test]
fn one_extruder_one_variant_replaces_only_map_without_validation() {
    let mut source = one_extruder_source();
    source.printer.gcode.extruder_type.0.clear();
    source.project.gcode.nozzle_volume_type.0.clear();
    source.printer.gcode.printer_extruder_id.0.clear();
    source.printer.gcode.printer_extruder_variant.0.clear();
    source.process.region.print_extruder_id.0.clear();
    source.process.region.print_extruder_variant.0.clear();
    source.project.preset.filament_self_index.0.clear();
    source.filament.gcode.filament_extruder_variant.0.clear();
    source.project.gcode.retraction_length = OrcaFloats(Vec::new());
    let invalid_map = ints(&[0, 99]);
    let original = source.clone();

    let materialized = materialize_project_variants(&source, &invalid_map).unwrap();

    assert_eq!(materialized.project.gcode.filament_map, invalid_map);
    assert_eq!(source, original);
    assert_eq!(
        materialized.project.gcode.retraction_length,
        OrcaFloats(Vec::new())
    );
}

#[test]
fn short_nonempty_typed_controls_repeat_first() {
    let mut repeated_type = active_source();
    repeated_type.printer.gcode.extruder_type =
        ExtruderTypes(vec![ExtruderType::DirectDrive]);
    repeated_type.project.gcode.nozzle_volume_type = NozzleVolumeTypes(vec![
        NozzleVolumeType::Standard,
        NozzleVolumeType::HighFlow,
    ]);
    set_variants(
        &mut repeated_type,
        &["Direct Drive Standard", "Direct Drive High Flow"],
    );
    materialize_project_variants(&repeated_type, &ints(&[1, 2])).unwrap();
    assert_eq!(
        inspect_printer_indices_for_test(&repeated_type)
            .unwrap()
            .unwrap(),
        vec![0, 1]
    );

    let mut repeated_volume = active_source();
    repeated_volume.project.gcode.nozzle_volume_type =
        NozzleVolumeTypes(vec![NozzleVolumeType::HighFlow]);
    set_variants(
        &mut repeated_volume,
        &["Direct Drive High Flow", "Bowden High Flow"],
    );
    materialize_project_variants(&repeated_volume, &ints(&[1, 2])).unwrap();
    assert_eq!(
        inspect_printer_indices_for_test(&repeated_volume)
            .unwrap()
            .unwrap(),
        vec![0, 1]
    );
}

#[test]
fn empty_boundary_vectors_name_their_orca_keys() {
    let mut source = active_source();
    source.project.print.nozzle_diameter.0.clear();
    assert_invalid_key(materialize_project_variants(&source, &ints(&[1])), "nozzle_diameter");

    let mut source = active_source();
    source.printer.remaining.extruder_variant_list.0.clear();
    assert_invalid_key(
        materialize_project_variants(&source, &ints(&[1, 2])),
        "extruder_variant_list",
    );

    let mut source = active_source();
    source.printer.gcode.extruder_type.0.clear();
    assert_invalid_key(
        materialize_project_variants(&source, &ints(&[1, 2])),
        "extruder_type",
    );

    let mut source = active_source();
    source.project.gcode.nozzle_volume_type.0.clear();
    assert_invalid_key(
        materialize_project_variants(&source, &ints(&[1, 2])),
        "nozzle_volume_type",
    );
}

#[test]
fn active_branch_invalid_selector_missing_match_and_map_name_keys() {
    let mut source = active_source();
    source.printer.gcode.printer_extruder_variant.0.clear();
    assert_invalid_key(
        materialize_project_variants(&source, &ints(&[1, 2])),
        "printer_extruder_variant",
    );

    let mut source = active_source();
    source.printer.gcode.printer_extruder_variant.0[1] = "missing".to_owned();
    assert_invalid_key(
        materialize_project_variants(&source, &ints(&[1, 2])),
        "printer_extruder_variant",
    );

    let mut source = active_source();
    source.process.region.print_extruder_variant.0.clear();
    assert_invalid_key(
        materialize_project_variants(&source, &ints(&[1, 2])),
        "print_extruder_variant",
    );

    let mut source = active_source();
    source.filament.gcode.filament_extruder_variant = VariantStride(Vec::new());
    assert_invalid_key(
        materialize_project_variants(&source, &ints(&[1, 2])),
        "filament_extruder_variant",
    );

    let source = active_source();
    assert_invalid_key(
        materialize_project_variants(&source, &ints(&[0, 2])),
        "filament_map",
    );
    assert_invalid_key(
        materialize_project_variants(&source, &ints(&[1, 3])),
        "filament_map",
    );
}

fn set_variants(source: &mut crate::options::ProjectSettings, variants: &[&str]) {
    let values = variants
        .iter()
        .map(|variant| (*variant).to_owned())
        .collect::<Vec<_>>();
    source.printer.gcode.printer_extruder_variant = OrcaStrings(values.clone());
    source.process.region.print_extruder_variant = OrcaStrings(values.clone());
    source.filament.gcode.filament_extruder_variant = VariantStride(values);
}
