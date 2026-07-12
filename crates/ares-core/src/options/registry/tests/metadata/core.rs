use super::super::super::{OptionValueKind, option_definition};

#[test]
fn key_lookup_returns_upstream_metadata() {
    let layer_height = option_definition("layer_height").unwrap();
    assert_eq!(layer_height.kind, OptionValueKind::Float);
    assert_eq!(layer_height.default_value, "0.2");
    assert!(layer_height.source.contains("PrintConfig.cpp:749"));

    let initial_layer_height = option_definition("initial_layer_height").unwrap();
    assert_eq!(initial_layer_height.default_value, "0.3");
    assert!(initial_layer_height.source.contains("PrintConfig.cpp:7390"));

    let sparse_width = option_definition("sparse_infill_line_width").unwrap();
    assert_eq!(sparse_width.kind, OptionValueKind::FloatOrPercent);
    assert!(sparse_width.source.contains("PrintConfig.cpp:4016"));

    let brim_type = option_definition("brim_type").unwrap();
    assert_eq!(brim_type.kind, OptionValueKind::Enum);
    assert!(brim_type.source.contains("PrintConfig.cpp:1604"));

    let printable_area = option_definition("printable_area").unwrap();
    assert_eq!(printable_area.kind, OptionValueKind::Points);
    assert_eq!(printable_area.default_value, "0x0,200x0,200x200,0x200");
    assert!(printable_area.source.contains("PrintConfig.cpp:684"));

    let elephant_foot = option_definition("elefant_foot_compensation").unwrap();
    assert_eq!(elephant_foot.kind, OptionValueKind::Float);
    assert_eq!(elephant_foot.default_value, "0");
    assert!(elephant_foot.source.contains("PrintConfig.cpp:717"));

    let extruder_area = option_definition("extruder_printable_area").unwrap();
    assert_eq!(extruder_area.kind, OptionValueKind::PointsGroups);
    assert_eq!(extruder_area.default_value, "");
    assert!(extruder_area.source.contains("PrintConfig.cpp:690"));

    let preset_names = option_definition("preset_names").unwrap();
    assert_eq!(preset_names.kind, OptionValueKind::Strings);
    assert_eq!(preset_names.default_value, "");
    assert!(preset_names.source.contains("PrintConfig.cpp:786"));
    assert!(preset_names.source.contains("872"));

    let authorization = option_definition("printhost_authorization_type").unwrap();
    assert_eq!(authorization.kind, OptionValueKind::Enum);
    assert_eq!(authorization.default_value, "key");
    assert!(authorization.source.contains("PrintConfig.hpp:83"));
    assert!(authorization.source.contains("PrintConfig.cpp:878"));

    let print_host = option_definition("print_host").unwrap();
    assert_eq!(print_host.kind, OptionValueKind::String);
    assert!(print_host.source.contains("PrintConfig.cpp:806"));

    let reduce_crossing_wall = option_definition("reduce_crossing_wall").unwrap();
    assert_eq!(reduce_crossing_wall.kind, OptionValueKind::Bool);
    assert_eq!(reduce_crossing_wall.default_value, "false");
    assert!(reduce_crossing_wall.source.contains("PrintConfig.cpp:904"));
    assert!(
        reduce_crossing_wall
            .source
            .contains("PrintConfigConstants.hpp:7")
    );

    let max_detour = option_definition("max_travel_detour_distance").unwrap();
    assert_eq!(max_detour.kind, OptionValueKind::FloatOrPercent);
    assert_eq!(max_detour.default_value, "0");
    assert!(max_detour.source.contains("PrintConfig.cpp:911"));

    let cool_plate = option_definition("cool_plate_temp").unwrap();
    assert_eq!(cool_plate.kind, OptionValueKind::Ints);
    assert_eq!(cool_plate.default_value, "35");
    assert!(cool_plate.source.contains("PrintConfig.cpp:934"));

    let textured_cool = option_definition("textured_cool_plate_temp").unwrap();
    assert_eq!(textured_cool.kind, OptionValueKind::Ints);
    assert_eq!(textured_cool.default_value, "40");
    assert!(textured_cool.source.contains("PrintConfig.cpp:944"));

    let hot_plate = option_definition("hot_plate_temp").unwrap();
    assert_eq!(hot_plate.kind, OptionValueKind::Ints);
    assert_eq!(hot_plate.default_value, "45");
    assert!(hot_plate.source.contains("PrintConfig.cpp:964"));

    fn assert_int_definition(key: &str, default_value: &str, hpp_line: &str, cpp_line: &str) {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, OptionValueKind::Ints);
        assert_eq!(definition.default_value, default_value);
        assert!(definition.source.contains(hpp_line));
        assert!(definition.source.contains(cpp_line));
    }

    assert_int_definition(
        "cool_plate_temp_initial_layer",
        "35",
        "PrintConfig.hpp:1497",
        "PrintConfig.cpp:994",
    );
    assert_int_definition(
        "eng_plate_temp_initial_layer",
        "45",
        "PrintConfig.hpp:1499",
        "PrintConfig.cpp:1014",
    );
    assert_int_definition(
        "hot_plate_temp_initial_layer",
        "45",
        "PrintConfig.hpp:1500",
        "PrintConfig.cpp:1024",
    );
    assert_int_definition(
        "supertack_plate_temp_initial_layer",
        "35",
        "PrintConfig.hpp:1496",
        "PrintConfig.cpp:984",
    );
    assert_int_definition(
        "textured_cool_plate_temp_initial_layer",
        "40",
        "PrintConfig.hpp:1498",
        "PrintConfig.cpp:1004",
    );
    assert_int_definition(
        "textured_plate_temp_initial_layer",
        "45",
        "PrintConfig.hpp:1501",
        "PrintConfig.cpp:1033",
    );

    fn assert_definition(key: &str, kind: OptionValueKind, default_value: &str, cpp_line: &str) {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        assert!(definition.source.contains(cpp_line));
    }

    assert_definition(
        "curr_bed_type",
        OptionValueKind::Enum,
        "Cool Plate",
        "PrintConfig.cpp:1043",
    );
    assert!(
        option_definition("curr_bed_type")
            .unwrap()
            .source
            .contains("PrintConfig.hpp:1489")
    );
    assert!(
        option_definition("curr_bed_type")
            .unwrap()
            .source
            .contains("PrintConfig.hpp:314")
    );
    assert!(
        option_definition("curr_bed_type")
            .unwrap()
            .source
            .contains("PrintConfig.cpp:467")
    );
    assert_definition(
        "default_bed_type",
        OptionValueKind::String,
        "",
        "PrintConfig.cpp:1065",
    );
    assert_definition(
        "first_layer_print_sequence",
        OptionValueKind::Ints,
        "0",
        "PrintConfig.cpp:1072",
    );
    assert!(
        option_definition("first_layer_print_sequence")
            .unwrap()
            .source
            .contains("PrintConfig.hpp:1507")
    );
    assert_definition(
        "other_layers_print_sequence",
        OptionValueKind::Ints,
        "0",
        "PrintConfig.cpp:1078",
    );
    assert!(
        option_definition("other_layers_print_sequence")
            .unwrap()
            .source
            .contains("PrintConfig.hpp:1508")
    );
    assert_definition(
        "other_layers_print_sequence_nums",
        OptionValueKind::Int,
        "0",
        "PrintConfig.cpp:1084",
    );
    assert!(
        option_definition("other_layers_print_sequence_nums")
            .unwrap()
            .source
            .contains("PrintConfig.hpp:1509")
    );
    assert_definition(
        "first_layer_sequence_choice",
        OptionValueKind::Enum,
        "Auto",
        "PrintConfig.cpp:1088",
    );
    assert!(
        option_definition("first_layer_sequence_choice")
            .unwrap()
            .source
            .contains("PrintConfig.hpp:333")
    );
    assert!(
        option_definition("first_layer_sequence_choice")
            .unwrap()
            .source
            .contains("PrintConfig.cpp:479")
    );
    assert_definition(
        "other_layers_sequence_choice",
        OptionValueKind::Enum,
        "Auto",
        "PrintConfig.cpp:1099",
    );
    assert!(
        option_definition("other_layers_sequence_choice")
            .unwrap()
            .source
            .contains("PrintConfig.hpp:333")
    );
    assert!(
        option_definition("other_layers_sequence_choice")
            .unwrap()
            .source
            .contains("PrintConfig.cpp:479")
    );

    for (key, kind, default_value, source_fragments) in [
        (
            "before_layer_change_gcode",
            OptionValueKind::String,
            "",
            &["PrintConfig.hpp:1294", "PrintConfig.cpp:1110"][..],
        ),
        (
            "bottom_shell_layers",
            OptionValueKind::Int,
            "3",
            &["PrintConfig.hpp:1079", "PrintConfig.cpp:1119"][..],
        ),
        (
            "bottom_shell_thickness",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1080", "PrintConfig.cpp:1130"][..],
        ),
        (
            "gap_fill_target",
            OptionValueKind::Enum,
            "nowhere",
            &[
                "PrintConfig.hpp:241",
                "PrintConfig.hpp:1038",
                "PrintConfig.cpp:393",
                "PrintConfig.cpp:1141",
            ][..],
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }

    for (key, kind, default_value, source_fragments) in [
        (
            "enable_overhang_bridge_fan",
            OptionValueKind::Bools,
            "true",
            &["PrintConfig.hpp:1502", "PrintConfig.cpp:1170"][..],
        ),
        (
            "overhang_fan_speed",
            OptionValueKind::Ints,
            "100",
            &["PrintConfig.hpp:1503", "PrintConfig.cpp:1177"][..],
        ),
        (
            "overhang_fan_threshold",
            OptionValueKind::Enums,
            "95%",
            &[
                "PrintConfig.hpp:304",
                "PrintConfig.hpp:1504",
                "PrintConfig.cpp:456",
                "PrintConfig.cpp:1190",
            ][..],
        ),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}

#[test]
fn unknown_key_returns_none() {
    assert_eq!(option_definition("unknown_future_option"), None);
}
