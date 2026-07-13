use super::super::super::{
    AmsCounts, FlatMatrix, NozzleVolumeTypes, OrcaBool, OrcaBools, OrcaFloats, OrcaInt,
    OrcaInts, OrcaPercents, OrcaString, OrcaStrings, Point2dList, PresetMetadata,
    ProjectBedType, ProjectFilamentMapMode, ProjectGCodeSourceOptions, ProjectPrintSourceOptions,
    ProjectPresetSourceOptions, ProjectRuntimeOptions, ProjectSettings,
};

#[test]
fn every_task14_public_field_has_its_concrete_type() {
    let runtime = ProjectRuntimeOptions::default();
    assert_gcode_types(&runtime.gcode);
    assert_print_types(&runtime.print);
    assert_preset_types(&runtime.preset);

    let metadata = PresetMetadata::default();
    let _: &String = &metadata.from;
    let _: &String = &metadata.name;
    let _: &String = &metadata.version;

    let settings = ProjectSettings::default();
    let _: &ProjectRuntimeOptions = &settings.project;
    let _: &PresetMetadata = &settings.metadata;
}

fn assert_gcode_types(value: &ProjectGCodeSourceOptions) {
    fields!(value, OrcaFloats;
        deretraction_speed, retraction_length, retract_length_toolchange, z_hop,
        retract_lift_above, retract_lift_below, retract_restart_extra,
        retract_restart_extra_toolchange, retraction_speed
    );
    let _: &OrcaStrings = &value.filament_ids;
    let _: &ProjectFilamentMapMode = &value.filament_map_mode;
    let _: &OrcaInts = &value.filament_map;
    let _: &OrcaPercents = &value.retract_before_wipe;
    let _: &NozzleVolumeTypes = &value.nozzle_volume_type;
    let _: &AmsCounts = &value.extruder_ams_count;
    fields!(value, OrcaBool; bbl_calib_mark_logo, has_scarf_joint_seam);
}

fn assert_print_types(value: &ProjectPrintSourceOptions) {
    let _: &ProjectBedType = &value.curr_bed_type;
    fields!(value, OrcaInts; first_layer_print_sequence, other_layers_print_sequence);
    let _: &OrcaInt = &value.other_layers_print_sequence_nums;
    let _: &OrcaStrings = &value.extruder_colour;
    let _: &Point2dList = &value.extruder_offset;
    fields!(value, OrcaFloats;
        max_layer_height, min_layer_height, nozzle_diameter, retraction_minimum_travel,
        wipe_distance, wipe_tower_x, wipe_tower_y, flush_volumes_vector, flush_multiplier
    );
    fields!(value, OrcaBools; retract_when_changing_layer, wipe);
    let _: &FlatMatrix = &value.flush_volumes_matrix;
    let _: &Point2dList = &value.start_end_points;
}

fn assert_preset_types(value: &ProjectPresetSourceOptions) {
    fields!(value, OrcaStrings;
        print_compatible_printers, default_filament_profile, filament_multi_colour,
        filament_colour_type, filament_settings_id
    );
    fields!(value, OrcaString; print_settings_id, printer_settings_id);
    let _: &OrcaInts = &value.filament_self_index;
}

macro_rules! fields {
    ($value:ident, $ty:ty; $($field:ident),+ $(,)?) => {
        $(let _: &$ty = &$value.$field;)+
    };
}

use fields;
