use super::super::super::{OptionValueKind, option_definition};

#[test]
fn extruder_clearance_metadata_matches_upstream_print_config() {
    for (key, kind, default_value, source_fragments) in [
        (
            "extruder_clearance_height_to_rod",
            OptionValueKind::Float,
            "40",
            &["PrintConfig.hpp:1513", "PrintConfig.cpp:2127-2134"][..],
        ),
        (
            "extruder_clearance_height_to_lid",
            OptionValueKind::Float,
            "120",
            &["PrintConfig.hpp:1514", "PrintConfig.cpp:2137-2144"][..],
        ),
        (
            "extruder_clearance_radius",
            OptionValueKind::Float,
            "40",
            &["PrintConfig.hpp:1515", "PrintConfig.cpp:2146-2152"][..],
        ),
        (
            "nozzle_height",
            OptionValueKind::Float,
            "2.5",
            &["PrintConfig.hpp:1516", "PrintConfig.cpp:2154-2160"][..],
        ),
        (
            "bed_mesh_min",
            OptionValueKind::Point,
            "-99999x-99999",
            &["PrintConfig.hpp:1641", "PrintConfig.cpp:2162-2172"][..],
        ),
        (
            "bed_mesh_max",
            OptionValueKind::Point,
            "99999x99999",
            &["PrintConfig.hpp:1642", "PrintConfig.cpp:2174-2184"][..],
        ),
        (
            "bed_mesh_probe_distance",
            OptionValueKind::Point,
            "50x50",
            &["PrintConfig.hpp:1643", "PrintConfig.cpp:2186-2193"][..],
        ),
        (
            "bed_temperature_formula",
            OptionValueKind::Enum,
            "by_highest_temp",
            &["PrintConfig.hpp:1340", "PrintConfig.cpp:2503-2512"][..],
        ),
        (
            "nozzle_flush_dataset",
            OptionValueKind::IntsNullable,
            "0",
            &["PrintConfig.hpp:1342", "PrintConfig.cpp:2514-2516"][..],
        ),
        (
            "adaptive_bed_mesh_margin",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1644", "PrintConfig.cpp:2195-2200"][..],
        ),
        (
            "grab_length",
            OptionValueKind::Floats,
            "0",
            &["PrintConfig.hpp:1625", "PrintConfig.cpp:2202-2207"][..],
        ),
        (
            "extruder_colour",
            OptionValueKind::Strings,
            "",
            &["PrintConfig.hpp:1517", "PrintConfig.cpp:2209-2215"][..],
        ),
        (
            "extruder_offset",
            OptionValueKind::Points,
            "0x0",
            &["PrintConfig.hpp:1518", "PrintConfig.cpp:2217-2225"][..],
        ),
        (
            "machine_load_filament_time",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1433", "PrintConfig.cpp:2472-2479"][..],
        ),
        (
            "machine_tool_change_time",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1434", "PrintConfig.cpp:2490-2497"][..],
        ),
        (
            "machine_unload_filament_time",
            OptionValueKind::Float,
            "0",
            &["PrintConfig.hpp:1435", "PrintConfig.cpp:2481-2488"][..],
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
