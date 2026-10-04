use super::*;
use crate::ProcessWipeTowerWallType;

#[test]
fn priming_initializes_tool_order_and_resets_remembered_temperature() {
    let mut settings = ProjectSettings::default();
    settings.printer.gcode.single_extruder_multi_material.0 = true;
    settings.printer.gcode.enable_filament_ramming.0 = false;
    let filaments = (0..2)
        .map(|index| Filament::from_config(&settings, index))
        .collect();
    let mut generator = Generator::new(
        &settings,
        filaments,
        (CoordinateScale::Normal, GCodeFlavor::MarlinLegacy),
        1,
        vec![vec![0.0, 200.0], vec![110.0, 0.0]],
    );
    let output = generator.prime(0.2, &[1, 0]);
    assert_eq!(output.len(), 2);
    assert!(
        output
            .iter()
            .all(|result| result.priming && result.finish_first)
    );
    assert!(output[0].gcode.contains("; CP PRIMING START\n"));
    assert!(output[1].gcode.contains("; CP PRIMING END\n"));
    assert_eq!(output[0].new_tool, 1);
    assert_eq!(output[1].new_tool, 0);
    assert_eq!(generator.current_tool, 0);
    assert_eq!(generator.tool_changes, 2);
    assert_eq!(generator.old_temperature, -1);
    assert!(generator.used_filament.iter().all(|&amount| amount > 0.0));
}

#[test]
fn activated_two_tool_layers_generate_all_source_wall_shapes() {
    for wall in [
        ProcessWipeTowerWallType::Rectangle,
        ProcessWipeTowerWallType::Cone,
        ProcessWipeTowerWallType::Rib,
    ] {
        let mut settings = ProjectSettings::default();
        settings.process.print.wipe_tower_wall_type = wall;
        settings.process.print.prime_tower_width.0 = 40.0;
        settings.process.print.prime_tower_brim_width.0 = 1.0;
        settings.process.gcode.enable_arc_fitting.0 = true;
        settings.printer.gcode.enable_filament_ramming.0 = false;
        let filaments = (0..2)
            .map(|index| Filament::from_config(&settings, index))
            .collect();
        let mut generator = Generator::new(
            &settings,
            filaments,
            (CoordinateScale::Normal, GCodeFlavor::MarlinLegacy),
            0,
            vec![vec![0.0, 100.0], vec![110.0, 0.0]],
        );
        generator
            .plan
            .add((0.2, 0.2), (0, 0), 0.0, &generator.filaments);
        generator
            .plan
            .add((0.4, 0.2), (0, 1), 100.0, &generator.filaments);
        generator
            .plan
            .add((0.6, 0.2), (1, 0), 110.0, &generator.filaments);
        let output = generator.generate().unwrap();
        assert_eq!(output.len(), 3);
        assert!(output[0][0].finish_first);
        assert!(output[0][0].gcode.contains("; WIPE_TOWER_BRIM_START\n"));
        assert_eq!(output[1][0].initial_tool, 0);
        assert_eq!(output[1][0].new_tool, 1);
        assert_eq!(output[2][0].new_tool, 0);
        assert!(output[1][0].gcode.contains("[change_filament_gcode]\n"));
        assert!(output[1][0].gcode.contains("; WIPE_TOWER_START\n"));
        assert!(output[1][0].gcode.contains("; CP TOOLCHANGE WIPE\n"));
        assert!(
            output
                .iter()
                .flatten()
                .all(|result| result.wipe_path.len() >= 2)
        );
        assert!(generator.used_filament.iter().all(|&amount| amount > 0.0));
        assert!(generator.first_layer_bounds.is_some());
        assert!(
            output
                .iter()
                .flatten()
                .all(|result| !result.gcode.contains("\nG2 ") && !result.gcode.contains("\nG3 "))
        );
        assert_eq!(generator.used_until_layer.last().unwrap().0, 0.6);
    }
}
