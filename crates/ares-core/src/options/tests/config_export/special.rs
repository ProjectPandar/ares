use crate::{
    FlatMatrix, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, OrcaStrings, ProjectBedType,
    ProjectSettings, SliceError,
    options::config_export::{
        collector::ConfigEntry, write_canonical_entries, write_config_block,
    },
};

use super::{assignment_lines, block, block_from_views, views};

const MATRIX_ERROR: &str = "Flush volumes matrix do not match to the correct size!";

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}

fn ints(values: &[i32]) -> OrcaInts {
    OrcaInts(values.iter().copied().map(OrcaInt).collect())
}

#[test]
fn config_export_special_scales_each_flush_head_with_rounding_without_source_mutation() {
    let mut settings = ProjectSettings::default();
    settings.filament.gcode.filament_colour =
        OrcaStrings(vec!["#111111".to_owned(), "#222222".to_owned()]);
    settings.project.print.flush_multiplier = floats(&[0.3, 1.0]);
    settings.project.print.flush_volumes_matrix =
        FlatMatrix(vec![0.0, 281.0, 279.0, 0.0, 0.0, 279.5, 280.5, 0.0]);
    let views = views(settings);
    let source = views.full.project.print.flush_volumes_matrix.clone();

    let output = block_from_views(&views, 0).unwrap();

    assert_eq!(
        assignment_lines(&output, "flush_volumes_matrix"),
        ["; flush_volumes_matrix = 0,84,84,0,0,280,281,0"]
    );
    assert_eq!(views.full.project.print.flush_volumes_matrix, source);
}

#[test]
fn config_export_special_preserves_single_filament_matrix_mismatch() {
    let mut settings = ProjectSettings::default();
    settings.filament.gcode.filament_colour = OrcaStrings(vec!["#111111".to_owned()]);
    settings.project.print.flush_multiplier = floats(&[0.5, 1.0]);
    settings.project.print.flush_volumes_matrix = FlatMatrix(vec![1.25, 2.5, 3.75]);

    let output = block(settings, 0).unwrap();

    assert_eq!(
        assignment_lines(&output, "flush_volumes_matrix"),
        ["; flush_volumes_matrix = 1.25,2.5,3.75"]
    );
}

#[test]
fn config_export_special_rejects_multi_filament_mismatch_and_zero_heads_atomically() {
    let cases = [
        (vec!["#111111", "#222222"], vec![1.0], vec![1.0, 2.0, 3.0]),
        (vec!["#111111"], vec![], vec![]),
    ];
    for (colours, multipliers, matrix) in cases {
        let mut settings = ProjectSettings::default();
        settings.filament.gcode.filament_colour =
            OrcaStrings(colours.into_iter().map(str::to_owned).collect());
        settings.project.print.flush_multiplier = floats(&multipliers);
        settings.project.print.flush_volumes_matrix = FlatMatrix(matrix);
        let views = views(settings);
        let mut output = b"preseed".to_vec();
        let expected = output.clone();

        let error = write_config_block(&views, 0, &mut output).unwrap_err();

        assert_eq!(error, SliceError::InvalidInput(MATRIX_ERROR.to_owned()));
        assert_eq!(output, expected);
    }
}

#[test]
fn config_export_special_filters_only_the_fixed_nine_banned_keys() {
    const BANNED: [&str; 9] = [
        "compatible_printers",
        "compatible_prints",
        "print_host",
        "print_host_webui",
        "printhost_apikey",
        "printhost_cafile",
        "printhost_user",
        "printhost_password",
        "printhost_port",
    ];
    let mut entries = BANNED
        .into_iter()
        .map(|key| ConfigEntry {
            key: key.to_owned(),
            token: "drop".to_owned(),
            is_nil: false,
        })
        .collect::<Vec<_>>();
    entries.push(ConfigEntry {
        key: "print_compatible_printers".to_owned(),
        token: "keep".to_owned(),
        is_nil: false,
    });
    entries.push(ConfigEntry {
        key: "nullable".to_owned(),
        token: "nil".to_owned(),
        is_nil: true,
    });
    entries.sort_unstable_by(|left, right| left.key.cmp(&right.key));
    let mut output = Vec::new();

    write_canonical_entries(&ProjectSettings::default(), 0, &entries, &mut output).unwrap();

    assert_eq!(output, b"; print_compatible_printers = keep\n");
}

#[test]
fn config_export_special_substitutes_typed_filament_colour_without_mutation() {
    let mut settings = ProjectSettings::default();
    settings.project.print.extruder_colour = OrcaStrings(vec!["#old".to_owned()]);
    settings.filament.gcode.filament_colour =
        OrcaStrings(vec!["#111111".to_owned(), "#222222".to_owned()]);
    settings.project.print.flush_multiplier = floats(&[1.0]);
    settings.project.print.flush_volumes_matrix = FlatMatrix(vec![0.0; 4]);
    let views = views(settings);
    let extruder = views.full.project.print.extruder_colour.clone();
    let filament = views.full.filament.gcode.filament_colour.clone();

    let output = block_from_views(&views, 0).unwrap();

    assert_eq!(
        assignment_lines(&output, "extruder_colour"),
        ["; extruder_colour = #111111;#222222"]
    );
    assert_eq!(views.full.project.print.extruder_colour, extruder);
    assert_eq!(views.full.filament.gcode.filament_colour, filament);
}

#[test]
fn config_export_special_writes_selected_coordinate_then_ordinary_vector() {
    let mut settings = ProjectSettings::default();
    settings.project.print.wipe_tower_x = floats(&[1.23456, 7.8912]);
    settings.project.print.wipe_tower_y = floats(&[2.34567, 8.9123]);

    let selected = block(settings.clone(), 1).unwrap();
    assert_eq!(
        assignment_lines(&selected, "wipe_tower_x"),
        ["; wipe_tower_x = 7.891", "; wipe_tower_x = 1.23456,7.8912"]
    );
    assert_eq!(
        assignment_lines(&selected, "wipe_tower_y"),
        ["; wipe_tower_y = 8.912", "; wipe_tower_y = 2.34567,8.9123"]
    );

    let fallback = block(settings, 99).unwrap();
    assert_eq!(
        assignment_lines(&fallback, "wipe_tower_x"),
        ["; wipe_tower_x = 1.235", "; wipe_tower_x = 1.23456,7.8912"]
    );
    assert_eq!(
        assignment_lines(&fallback, "wipe_tower_y"),
        ["; wipe_tower_y = 2.346", "; wipe_tower_y = 2.34567,8.9123"]
    );
}

#[test]
fn config_export_special_selects_all_six_runtime_bed_vectors_and_first_values() {
    let mut views = views(ProjectSettings::default());
    views.runtime.filament.print.supertack_plate_temp_initial_layer = ints(&[11, 111]);
    views.runtime.filament.print.cool_plate_temp_initial_layer = ints(&[22, 122]);
    views.runtime.filament.print.textured_cool_plate_temp_initial_layer = ints(&[33, 133]);
    views.runtime.filament.print.eng_plate_temp_initial_layer = ints(&[44, 144]);
    views.runtime.filament.print.hot_plate_temp_initial_layer = ints(&[55, 155]);
    views.runtime.filament.print.textured_plate_temp_initial_layer = ints(&[66, 166]);
    views.runtime.filament.print.nozzle_temperature_initial_layer = ints(&[220, 221]);

    for (bed_type, expected) in [
        (ProjectBedType::SupertackPlate, 11),
        (ProjectBedType::CoolPlate, 22),
        (ProjectBedType::TexturedCoolPlate, 33),
        (ProjectBedType::EngineeringPlate, 44),
        (ProjectBedType::HighTempPlate, 55),
        (ProjectBedType::TexturedPeiPlate, 66),
    ] {
        views.runtime.project.print.curr_bed_type = bed_type;
        let output = block_from_views(&views, 0).unwrap();
        assert_eq!(
            assignment_lines(&output, "first_layer_bed_temperature"),
            [format!("; first_layer_bed_temperature = {expected}")]
        );
        assert_eq!(
            assignment_lines(&output, "first_layer_temperature"),
            ["; first_layer_temperature = 220"]
        );
    }
}

#[test]
fn config_export_special_late_temperature_errors_leave_output_unchanged() {
    let mut cases = Vec::new();

    let mut default_plate = views(ProjectSettings::default());
    default_plate.runtime.project.print.curr_bed_type = ProjectBedType::DefaultPlate;
    cases.push(default_plate);

    let mut empty_bed = views(ProjectSettings::default());
    empty_bed.runtime.project.print.curr_bed_type = ProjectBedType::CoolPlate;
    empty_bed.runtime.filament.print.cool_plate_temp_initial_layer = ints(&[]);
    cases.push(empty_bed);

    let mut empty_nozzle = views(ProjectSettings::default());
    empty_nozzle.runtime.filament.print.nozzle_temperature_initial_layer = ints(&[]);
    cases.push(empty_nozzle);

    for views in cases {
        let mut output = b"preseed".to_vec();
        let expected = output.clone();
        assert!(matches!(
            write_config_block(&views, 0, &mut output),
            Err(SliceError::InvalidInput(_))
        ));
        assert_eq!(output, expected);
    }
}
