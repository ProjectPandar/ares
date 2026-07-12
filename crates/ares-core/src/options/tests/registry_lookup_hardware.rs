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
    for (key, kind, default_value) in [
        (
            "bed_temperature_formula",
            crate::OptionValueKind::Enum,
            "by_highest_temp",
        ),
        (
            "nozzle_flush_dataset",
            crate::OptionValueKind::IntsNullable,
            "0",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
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
    for key in [
        "machine_load_filament_time",
        "machine_tool_change_time",
        "machine_unload_filament_time",
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, crate::OptionValueKind::Float);
        assert_eq!(definition.default_value, "0");
    }
}
