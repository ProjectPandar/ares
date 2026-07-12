use super::super::super::{OptionValueKind, option_definition};

#[test]
fn key_lookup_returns_registry_metadata() {
    let layer_height = option_definition("layer_height").unwrap();
    assert_eq!(layer_height.kind, OptionValueKind::Float);
    assert_eq!(layer_height.default_value, "0.2");

    let initial_layer_height = option_definition("initial_layer_height").unwrap();
    assert_eq!(initial_layer_height.default_value, "0.3");

    let sparse_width = option_definition("sparse_infill_line_width").unwrap();
    assert_eq!(sparse_width.kind, OptionValueKind::FloatOrPercent);

    let brim_type = option_definition("brim_type").unwrap();
    assert_eq!(brim_type.kind, OptionValueKind::Enum);

    let printable_area = option_definition("printable_area").unwrap();
    assert_eq!(printable_area.kind, OptionValueKind::Points);
    assert_eq!(printable_area.default_value, "0x0,200x0,200x200,0x200");

    let elephant_foot = option_definition("elefant_foot_compensation").unwrap();
    assert_eq!(elephant_foot.kind, OptionValueKind::Float);
    assert_eq!(elephant_foot.default_value, "0");

    let extruder_area = option_definition("extruder_printable_area").unwrap();
    assert_eq!(extruder_area.kind, OptionValueKind::PointsGroups);
    assert_eq!(extruder_area.default_value, "");

    let preset_names = option_definition("preset_names").unwrap();
    assert_eq!(preset_names.kind, OptionValueKind::Strings);
    assert_eq!(preset_names.default_value, "");

    let authorization = option_definition("printhost_authorization_type").unwrap();
    assert_eq!(authorization.kind, OptionValueKind::Enum);
    assert_eq!(authorization.default_value, "key");

    let print_host = option_definition("print_host").unwrap();
    assert_eq!(print_host.kind, OptionValueKind::String);

    let reduce_crossing_wall = option_definition("reduce_crossing_wall").unwrap();
    assert_eq!(reduce_crossing_wall.kind, OptionValueKind::Bool);
    assert_eq!(reduce_crossing_wall.default_value, "false");

    let max_detour = option_definition("max_travel_detour_distance").unwrap();
    assert_eq!(max_detour.kind, OptionValueKind::FloatOrPercent);
    assert_eq!(max_detour.default_value, "0");

    let cool_plate = option_definition("cool_plate_temp").unwrap();
    assert_eq!(cool_plate.kind, OptionValueKind::Ints);
    assert_eq!(cool_plate.default_value, "35");

    let textured_cool = option_definition("textured_cool_plate_temp").unwrap();
    assert_eq!(textured_cool.kind, OptionValueKind::Ints);
    assert_eq!(textured_cool.default_value, "40");

    let hot_plate = option_definition("hot_plate_temp").unwrap();
    assert_eq!(hot_plate.kind, OptionValueKind::Ints);
    assert_eq!(hot_plate.default_value, "45");

    fn assert_int_definition(key: &str, default_value: &str) {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, OptionValueKind::Ints);
        assert_eq!(definition.default_value, default_value);
    }

    assert_int_definition("cool_plate_temp_initial_layer", "35");
    assert_int_definition("eng_plate_temp_initial_layer", "45");
    assert_int_definition("hot_plate_temp_initial_layer", "45");
    assert_int_definition("supertack_plate_temp_initial_layer", "35");
    assert_int_definition("textured_cool_plate_temp_initial_layer", "40");
    assert_int_definition("textured_plate_temp_initial_layer", "45");

    fn assert_definition(key: &str, kind: OptionValueKind, default_value: &str) {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }

    assert_definition("curr_bed_type", OptionValueKind::Enum, "Cool Plate");
    assert_definition("default_bed_type", OptionValueKind::String, "");
    assert_definition("first_layer_print_sequence", OptionValueKind::Ints, "0");
    assert_definition("other_layers_print_sequence", OptionValueKind::Ints, "0");
    assert_definition(
        "other_layers_print_sequence_nums",
        OptionValueKind::Int,
        "0",
    );
    assert_definition("first_layer_sequence_choice", OptionValueKind::Enum, "Auto");
    assert_definition(
        "other_layers_sequence_choice",
        OptionValueKind::Enum,
        "Auto",
    );

    for (key, kind, default_value) in [
        ("before_layer_change_gcode", OptionValueKind::String, ""),
        ("bottom_shell_layers", OptionValueKind::Int, "3"),
        ("bottom_shell_thickness", OptionValueKind::Float, "0"),
        ("gap_fill_target", OptionValueKind::Enum, "nowhere"),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }

    for (key, kind, default_value) in [
        ("enable_overhang_bridge_fan", OptionValueKind::Bools, "true"),
        ("overhang_fan_speed", OptionValueKind::Ints, "100"),
        ("overhang_fan_threshold", OptionValueKind::Enums, "95%"),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}

#[test]
fn unknown_key_returns_none() {
    assert_eq!(option_definition("unknown_future_option"), None);
}
