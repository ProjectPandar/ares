use super::super::apply_filament_map_extraction_state::staged_apply_filament_map_extraction;

#[test]
fn apply_filament_map_extraction_defaults_missing_option_to_empty_values() {
    let extraction = staged_apply_filament_map_extraction(None);

    assert!(extraction.filament_maps.is_empty());
}

#[test]
fn apply_filament_map_extraction_preserves_present_empty_values() {
    let extraction = staged_apply_filament_map_extraction(Some(&[]));

    assert!(extraction.filament_maps.is_empty());
}

#[test]
fn apply_filament_map_extraction_preserves_source_order() {
    let extraction = staged_apply_filament_map_extraction(Some(&[3, 1, 2]));

    assert_eq!(extraction.filament_maps, [3, 1, 2]);
}

#[test]
fn apply_filament_map_extraction_preserves_duplicates_and_negative_values() {
    let extraction = staged_apply_filament_map_extraction(Some(&[2, -1, 2, 0]));

    assert_eq!(extraction.filament_maps, [2, -1, 2, 0]);
}
