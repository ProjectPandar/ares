#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Child {
    GCode,
    Print,
    Preset,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ExpectedField {
    pub key: &'static str,
    pub child: Child,
    pub kind: &'static str,
    pub default_json: &'static str,
    pub is_array: bool,
}

macro_rules! field {
    ($key:literal, $child:ident, $kind:literal, $default:literal, $array:literal) => {
        ExpectedField {
            key: $key,
            child: Child::$child,
            kind: $kind,
            default_json: $default,
            is_array: $array,
        }
    };
}

pub(super) const REAL_FIELDS: [ExpectedField; 44] = [
    field!("bbl_calib_mark_logo", GCode, "coBool", "\"1\"", false),
    field!("curr_bed_type", Print, "coEnum", "\"Cool Plate\"", false),
    field!("default_filament_profile", Preset, "coStrings", "[]", true),
    field!("deretraction_speed", GCode, "coFloats", r#"["0"]"#, true),
    field!("extruder_ams_count", GCode, "coStrings", "[]", true),
    field!("extruder_colour", Print, "coStrings", r#"[""]"#, true),
    field!("extruder_offset", Print, "coPoints", r#"["0x0"]"#, true),
    field!("filament_colour_type", Preset, "coStrings", r#"["1"]"#, true),
    field!("filament_ids", GCode, "coStrings", "[]", true),
    field!("filament_map", GCode, "coInts", r#"["1"]"#, true),
    field!("filament_map_mode", GCode, "coEnum", "\"Auto For Flush\"", false),
    field!("filament_multi_colour", Preset, "coStrings", r#"[""]"#, true),
    field!("filament_self_index", Preset, "coInts", r#"["1"]"#, true),
    field!("filament_settings_id", Preset, "coStrings", r#"[""]"#, true),
    field!("first_layer_print_sequence", Print, "coInts", r#"["0"]"#, true),
    field!("flush_multiplier", Print, "coFloats", r#"["0.3"]"#, true),
    field!(
        "flush_volumes_matrix",
        Print,
        "coFloats",
        r#"["0","280","280","280","280","0","280","280","280","280","0","280","280","280","280","0"]"#,
        true
    ),
    field!(
        "flush_volumes_vector",
        Print,
        "coFloats",
        r#"["140","140","140","140","140","140","140","140"]"#,
        true
    ),
    field!("has_scarf_joint_seam", GCode, "coBool", "\"0\"", false),
    field!("max_layer_height", Print, "coFloats", r#"["0"]"#, true),
    field!("min_layer_height", Print, "coFloats", r#"["0.07"]"#, true),
    field!("nozzle_diameter", Print, "coFloats", r#"["0.4"]"#, true),
    field!("nozzle_volume_type", GCode, "coEnums", r#"["Standard"]"#, true),
    field!("other_layers_print_sequence", Print, "coInts", r#"["0"]"#, true),
    field!("other_layers_print_sequence_nums", Print, "coInt", "\"0\"", false),
    field!("print_compatible_printers", Preset, "coStrings", "[]", true),
    field!("print_settings_id", Preset, "coString", "\"\"", false),
    field!("printer_settings_id", Preset, "coString", "\"\"", false),
    field!("retract_before_wipe", GCode, "coPercents", r#"["100%"]"#, true),
    field!("retract_length_toolchange", GCode, "coFloats", r#"["10"]"#, true),
    field!("retract_lift_above", GCode, "coFloats", r#"["0"]"#, true),
    field!("retract_lift_below", GCode, "coFloats", r#"["0"]"#, true),
    field!("retract_restart_extra", GCode, "coFloats", r#"["0"]"#, true),
    field!(
        "retract_restart_extra_toolchange",
        GCode,
        "coFloats",
        r#"["0"]"#,
        true
    ),
    field!("retract_when_changing_layer", Print, "coBools", r#"["0"]"#, true),
    field!("retraction_length", GCode, "coFloats", r#"["0.8"]"#, true),
    field!("retraction_minimum_travel", Print, "coFloats", r#"["2"]"#, true),
    field!("retraction_speed", GCode, "coFloats", r#"["30"]"#, true),
    field!(
        "start_end_points",
        Print,
        "coPoints",
        r#"["30x-3","54x245"]"#,
        true
    ),
    field!("wipe", Print, "coBools", r#"["0"]"#, true),
    field!("wipe_distance", Print, "coFloats", r#"["1"]"#, true),
    field!("wipe_tower_x", Print, "coFloats", r#"["15"]"#, true),
    field!("wipe_tower_y", Print, "coFloats", r#"["220"]"#, true),
    field!("z_hop", GCode, "coFloats", r#"["0.4"]"#, true),
];

pub(super) const GCODE_DECLARATION_ORDER: [&str; 17] = [
    "deretraction_speed",
    "filament_ids",
    "filament_map_mode",
    "filament_map",
    "retract_before_wipe",
    "retraction_length",
    "retract_length_toolchange",
    "z_hop",
    "retract_lift_above",
    "retract_lift_below",
    "retract_restart_extra",
    "retract_restart_extra_toolchange",
    "retraction_speed",
    "nozzle_volume_type",
    "extruder_ams_count",
    "bbl_calib_mark_logo",
    "has_scarf_joint_seam",
];

pub(super) const PRINT_DECLARATION_ORDER: [&str; 19] = [
    "curr_bed_type",
    "first_layer_print_sequence",
    "other_layers_print_sequence",
    "other_layers_print_sequence_nums",
    "extruder_colour",
    "extruder_offset",
    "max_layer_height",
    "min_layer_height",
    "nozzle_diameter",
    "retraction_minimum_travel",
    "retract_when_changing_layer",
    "wipe",
    "wipe_distance",
    "wipe_tower_x",
    "wipe_tower_y",
    "flush_volumes_matrix",
    "flush_volumes_vector",
    "flush_multiplier",
    "start_end_points",
];

pub(super) const PRESET_DECLARATION_ORDER: [&str; 8] = [
    "print_compatible_printers",
    "default_filament_profile",
    "filament_multi_colour",
    "filament_colour_type",
    "filament_settings_id",
    "print_settings_id",
    "printer_settings_id",
    "filament_self_index",
];

pub(super) const RESIDUAL_LEXICAL_KEYS: [&str; 47] = [
    "bbl_calib_mark_logo",
    "curr_bed_type",
    "default_filament_profile",
    "deretraction_speed",
    "extruder_ams_count",
    "extruder_colour",
    "extruder_offset",
    "filament_colour_type",
    "filament_ids",
    "filament_map",
    "filament_map_mode",
    "filament_multi_colour",
    "filament_self_index",
    "filament_settings_id",
    "first_layer_print_sequence",
    "flush_multiplier",
    "flush_volumes_matrix",
    "flush_volumes_vector",
    "from",
    "has_scarf_joint_seam",
    "max_layer_height",
    "min_layer_height",
    "name",
    "nozzle_diameter",
    "nozzle_volume_type",
    "other_layers_print_sequence",
    "other_layers_print_sequence_nums",
    "print_compatible_printers",
    "print_settings_id",
    "printer_settings_id",
    "retract_before_wipe",
    "retract_length_toolchange",
    "retract_lift_above",
    "retract_lift_below",
    "retract_restart_extra",
    "retract_restart_extra_toolchange",
    "retract_when_changing_layer",
    "retraction_length",
    "retraction_minimum_travel",
    "retraction_speed",
    "start_end_points",
    "version",
    "wipe",
    "wipe_distance",
    "wipe_tower_x",
    "wipe_tower_y",
    "z_hop",
];

pub(super) const METADATA_KEYS: [&str; 3] = ["from", "name", "version"];

pub(super) const SINGLETON_ARRAY_KEYS: [&str; 6] = [
    "default_filament_profile",
    "first_layer_print_sequence",
    "other_layers_print_sequence",
    "print_compatible_printers",
    "wipe_tower_x",
    "wipe_tower_y",
];

pub(super) const DEFAULT_EQUAL_KEYS: [&str; 7] = [
    "bbl_calib_mark_logo",
    "filament_map_mode",
    "first_layer_print_sequence",
    "has_scarf_joint_seam",
    "other_layers_print_sequence",
    "other_layers_print_sequence_nums",
    "start_end_points",
];

pub(super) const PRODUCTION_LITERAL_COMPLEMENT: [&str; 13] = [
    "bbl_calib_mark_logo",
    "extruder_offset",
    "filament_self_index",
    "first_layer_print_sequence",
    "flush_multiplier",
    "flush_volumes_matrix",
    "flush_volumes_vector",
    "has_scarf_joint_seam",
    "other_layers_print_sequence",
    "other_layers_print_sequence_nums",
    "retract_length_toolchange",
    "retract_restart_extra_toolchange",
    "start_end_points",
];
