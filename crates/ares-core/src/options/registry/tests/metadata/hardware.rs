use super::super::super::{OptionValueKind, option_definition};

#[test]
fn extruder_clearance_metadata_preserves_registry_contract() {
    for (key, kind, default_value) in [
        (
            "extruder_clearance_height_to_rod",
            OptionValueKind::Float,
            "40",
        ),
        (
            "extruder_clearance_height_to_lid",
            OptionValueKind::Float,
            "120",
        ),
        ("extruder_clearance_radius", OptionValueKind::Float, "40"),
        ("nozzle_height", OptionValueKind::Float, "2.5"),
        ("bed_mesh_min", OptionValueKind::Point, "-99999x-99999"),
        ("bed_mesh_max", OptionValueKind::Point, "99999x99999"),
        ("bed_mesh_probe_distance", OptionValueKind::Point, "50x50"),
        (
            "bed_temperature_formula",
            OptionValueKind::Enum,
            "by_highest_temp",
        ),
        ("nozzle_flush_dataset", OptionValueKind::IntsNullable, "0"),
        ("adaptive_bed_mesh_margin", OptionValueKind::Float, "0"),
        ("grab_length", OptionValueKind::Floats, "0"),
        ("extruder_colour", OptionValueKind::Strings, ""),
        ("extruder_offset", OptionValueKind::Points, "0x0"),
        ("machine_load_filament_time", OptionValueKind::Float, "0"),
        ("machine_tool_change_time", OptionValueKind::Float, "0"),
        ("machine_unload_filament_time", OptionValueKind::Float, "0"),
    ] {
        let definition = option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
