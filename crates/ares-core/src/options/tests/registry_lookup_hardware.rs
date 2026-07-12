#[test]
fn exposes_extruder_clearance_option_definition_lookup() {
    for (key, default_value) in [
        ("extruder_clearance_height_to_lid", "120"),
        ("extruder_clearance_height_to_rod", "40"),
        ("extruder_clearance_radius", "40"),
        ("nozzle_height", "2.5"),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, crate::OptionValueKind::Float);
        assert_eq!(definition.default_value, default_value);
    }
}

#[test]
fn exposes_bed_mesh_option_definition_lookup() {
    for (key, kind, default_value) in [
        (
            "adaptive_bed_mesh_margin",
            crate::OptionValueKind::Float,
            "0",
        ),
        ("bed_mesh_max", crate::OptionValueKind::Point, "99999x99999"),
        (
            "bed_mesh_min",
            crate::OptionValueKind::Point,
            "-99999x-99999",
        ),
        (
            "bed_mesh_probe_distance",
            crate::OptionValueKind::Point,
            "50x50",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}

#[test]
fn exposes_bed_temperature_and_nozzle_flush_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "bed_temperature_formula",
            crate::OptionValueKind::Enum,
            "by_highest_temp",
            &["PrintConfig.hpp:1340", "PrintConfig.cpp:2503-2512"][..],
        ),
        (
            "nozzle_flush_dataset",
            crate::OptionValueKind::IntsNullable,
            "0",
            &["PrintConfig.hpp:1342", "PrintConfig.cpp:2514-2516"][..],
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}

#[test]
fn exposes_extruder_visual_and_offset_option_definition_lookup() {
    for (key, kind, default_value) in [
        ("extruder_colour", crate::OptionValueKind::Strings, ""),
        ("extruder_offset", crate::OptionValueKind::Points, "0x0"),
        ("grab_length", crate::OptionValueKind::Floats, "0"),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}

#[test]
fn exposes_filament_load_unload_time_option_definition_lookup() {
    for (key, source_fragments) in [
        (
            "machine_load_filament_time",
            &["PrintConfig.hpp:1433", "PrintConfig.cpp:2472-2479"][..],
        ),
        (
            "machine_tool_change_time",
            &["PrintConfig.hpp:1434", "PrintConfig.cpp:2490-2497"][..],
        ),
        (
            "machine_unload_filament_time",
            &["PrintConfig.hpp:1435", "PrintConfig.cpp:2481-2488"][..],
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, crate::OptionValueKind::Float);
        assert_eq!(definition.default_value, "0");
        for fragment in source_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}
