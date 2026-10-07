//! Public processed-slice seam: GCodeProcessor.cpp:3237-3301 must preserve
//! active-filament color tokens as ordered CustomGCode::Item records.
//! Failure inventory before implementation: the current recorder discards
//! color text; events could use zero-based tool ids, duplicate on M600, or
//! survive a later no-change slice. Fixture tags are supplied only through
//! the existing user template override, never through emitter changes.

use ares_core::{
    ConfigOverrides, GenerationMetadata, SlicerContext, slice_project_plate_processed,
};

const PROJECT: &[u8] = include_bytes!("../../../tests/parity/orca_cli_ender3.3mf");

#[tokio::test]
async fn processed_layer_templates_preserve_ordered_color_change_items() {
    let overrides = ConfigOverrides::from_json(
        br##"{"layer_change_gcode":"{if layer_num == 2};COLOR_CHANGE,T0,#FF0000\nM600\n{endif}{if layer_num == 4};COLOR_CHANGE,T0,#0000FF\nM600\n{endif}"} "##,
    )
    .unwrap();
    let output = slice_project_plate_processed(
        PROJECT.to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &overrides,
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    let gcode = std::str::from_utf8(&output.gcode).unwrap();
    assert_eq!(
        gcode
            .lines()
            .filter(|line| line.starts_with(";COLOR_CHANGE,T0,"))
            .count(),
        2
    );
    // CustomGCode::Item uses one-based extruder ids, unlike MoveVertex.
    let items = &output.result.custom_gcode_per_print_z;
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].extruder, 1);
    assert_eq!(items[1].extruder, 1);
    assert_eq!(items[0].color, "#FF0000");
    assert_eq!(items[1].color, "#0000FF");

    let plain = slice_project_plate_processed(
        PROJECT.to_vec(),
        GenerationMetadata::default(),
        Some(1),
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    assert!(plain.result.custom_gcode_per_print_z.is_empty());
}
