#[test]
fn exposes_filament_extruder_override_option_definition_lookup() {
    for (key, kind, default_value, raw_fragments) in [
        (
            "filament_deretraction_speed",
            crate::OptionValueKind::FloatsNullable,
            "0",
            &["PrintConfig.hpp:1296", "PrintConfig.cpp:5330-5336"][..],
        ),
        (
            "filament_long_retractions_when_cut",
            crate::OptionValueKind::BoolsNullable,
            "false",
            &["PrintConfig.hpp:1372", "PrintConfig.cpp:5081-5086"][..],
        ),
        (
            "filament_retract_before_wipe",
            crate::OptionValueKind::PercentsNullable,
            "100",
            &["PrintConfig.hpp:1367", "PrintConfig.cpp:5055-5060"][..],
        ),
        (
            "filament_retract_lift_above",
            crate::OptionValueKind::FloatsNullable,
            "0",
            &["PrintConfig.hpp:1379", "PrintConfig.cpp:5133-5139"][..],
        ),
        (
            "filament_retract_lift_below",
            crate::OptionValueKind::FloatsNullable,
            "0",
            &["PrintConfig.hpp:1380", "PrintConfig.cpp:5141-5147"][..],
        ),
        (
            "filament_retract_lift_enforce",
            crate::OptionValueKind::EnumsNullable,
            "All Surfaces",
            &["PrintConfig.hpp:390-394", "PrintConfig.cpp:5187-5200"][..],
        ),
        (
            "filament_retract_restart_extra",
            crate::OptionValueKind::FloatsNullable,
            "0",
            &["PrintConfig.hpp:1382", "PrintConfig.cpp:5306-5312"][..],
        ),
        (
            "filament_retract_when_changing_layer",
            crate::OptionValueKind::BoolsNullable,
            "false",
            &["PrintConfig.hpp:1551", "PrintConfig.cpp:5062-5066"][..],
        ),
        (
            "filament_retraction_distances_when_cut",
            crate::OptionValueKind::FloatsNullable,
            "18",
            &["PrintConfig.hpp:1371", "PrintConfig.cpp:5088-5094"][..],
        ),
        (
            "filament_retraction_length",
            crate::OptionValueKind::FloatsNullable,
            "0.8",
            &["PrintConfig.hpp:1368", "PrintConfig.cpp:5068-5075"][..],
        ),
        (
            "filament_retraction_minimum_travel",
            crate::OptionValueKind::FloatsNullable,
            "2",
            &["PrintConfig.hpp:1550", "PrintConfig.cpp:5048-5053"][..],
        ),
        (
            "filament_retraction_speed",
            crate::OptionValueKind::FloatsNullable,
            "30",
            &["PrintConfig.hpp:1384", "PrintConfig.cpp:5322-5328"][..],
        ),
        (
            "filament_wipe",
            crate::OptionValueKind::BoolsNullable,
            "false",
            &["PrintConfig.hpp:1569", "PrintConfig.cpp:6628-6633"][..],
        ),
        (
            "filament_wipe_distance",
            crate::OptionValueKind::FloatsNullable,
            "1",
            &["PrintConfig.hpp:1573", "PrintConfig.cpp:6635-6644"][..],
        ),
        (
            "filament_z_hop",
            crate::OptionValueKind::FloatsNullable,
            "0.4",
            &["PrintConfig.hpp:1375", "PrintConfig.cpp:5122-5131"][..],
        ),
        (
            "filament_z_hop_types",
            crate::OptionValueKind::EnumsNullable,
            "Slope Lift",
            &["PrintConfig.hpp:382-388", "PrintConfig.cpp:5149-5162"][..],
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
        for fragment in [
            "PrintConfig.hpp:512",
            "PrintConfig.cpp:63-83",
            "PrintConfig.cpp:7121-7156",
        ] {
            assert!(definition.source.contains(fragment));
        }
        for fragment in raw_fragments {
            assert!(definition.source.contains(fragment));
        }
    }
}
