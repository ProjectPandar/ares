use ares_core::PrintEstimatedStatistics;
use ares_core::gcode_processor::{EMoveType, GCodeProcessorResult, MoveVertex};
use ares_vgcode::{
    DUMMY_COLOR, FilamentColumn, ViewerImpl, convert::from_processor_result, filament_legend,
};

// Failure modes beyond the single-extruder E2E: unused tools becoming rows,
// sparse tool ids losing order/colors, absent buckets showing columns, missing
// tool colors not padded, and Flushed/Tower ordering or Total visibility drifting.
#[test]
fn used_extruders_select_volume_columns_and_pad_colors() {
    let extrude = |extruder_id| MoveVertex {
        move_type: EMoveType::Extrude,
        extruder_id,
        position: [f32::from(extruder_id), 0.0, 0.2],
        ..MoveVertex::default()
    };
    let mut result = GCodeProcessorResult {
        moves: vec![MoveVertex::default(), extrude(2), extrude(0)].into(),
        print_statistics: PrintEstimatedStatistics {
            model_volumes_per_extruder: vec![1000.0, 5.0, 2000.0],
            support_volumes_per_extruder: vec![0.0, 0.0, 300.0],
            flush_per_filament: vec![40.0],
            wipe_tower_volumes_per_extruder: vec![0.0, 0.0, 50.0],
            ..Default::default()
        },
        extruder_colors: vec!["#FF0000".into()],
        filament_diameters: vec![1.75, 1.75, 2.85],
        filament_densities: vec![1.24, 1.0, 1.04],
        ..Default::default()
    };
    let mut viewer = ViewerImpl::default();
    viewer.load(from_processor_result(&result));
    let legend = filament_legend(&viewer, &result);
    use FilamentColumn::*;
    assert_eq!(viewer.get_used_extruders_ids(), [0, 2]);
    assert_eq!(legend.columns, [Model, Support, Flushed, Tower, Total]);
    assert_eq!(
        legend
            .rows
            .iter()
            .map(|row| row.extruder_id)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    // LibVGCodeWrapper.cpp:58-61 clamps dark channels to 48.
    assert_eq!(legend.rows[0].color, [255, 48, 48]);
    assert_eq!(legend.rows[1].color, DUMMY_COLOR);
    assert_eq!(legend.rows[0].volumes, [1000.0, 0.0, 40.0, 0.0]);
    assert_eq!(legend.rows[1].volumes, [2000.0, 300.0, 0.0, 50.0]);
    assert_eq!(legend.filament_diameters, result.filament_diameters);
    assert_eq!(legend.filament_densities, result.filament_densities);

    result.print_statistics = PrintEstimatedStatistics {
        model_volumes_per_extruder: vec![1000.0],
        support_volumes_per_extruder: vec![0.0, 99.0], // unused extruder
        ..Default::default()
    };
    assert_eq!(filament_legend(&viewer, &result).columns, [Model]);
    // Empty input is a no-op upstream (ViewerImpl.cpp:990-991); a real reload resets.
    viewer.load(ares_vgcode::GCodeInputData {
        vertices: vec![ares_vgcode::PathVertex::default()],
        ..Default::default()
    });
    assert!(viewer.get_used_extruders_ids().is_empty());
    let empty = filament_legend(&viewer, &result);
    assert!(empty.columns.is_empty());
    assert!(empty.rows.is_empty());
}
