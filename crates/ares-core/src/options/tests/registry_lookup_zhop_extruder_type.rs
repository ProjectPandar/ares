#[test]
fn exposes_zhop_extruder_type_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "default_nozzle_volume_type",
            crate::OptionValueKind::Enums,
            "Standard",
            &[
                "PrintConfig.hpp:418-421",
                "PrintConfig.cpp:571-575",
                "PrintConfig.cpp:5227-5237",
            ][..],
        ),
        (
            "extruder_type",
            crate::OptionValueKind::Enums,
            "Direct Drive",
            &[
                "PrintConfig.hpp:412-415",
                "PrintConfig.hpp:1408",
                "PrintConfig.cpp:565-569",
                "PrintConfig.cpp:5202-5212",
            ][..],
        ),
        (
            "nozzle_volume_type",
            crate::OptionValueKind::Enums,
            "Standard",
            &[
                "PrintConfig.hpp:418-421",
                "PrintConfig.hpp:1409",
                "PrintConfig.cpp:571-575",
                "PrintConfig.cpp:5215-5225",
            ][..],
        ),
        (
            "retract_lift_above",
            crate::OptionValueKind::Floats,
            "0",
            &[
                "PrintConfig.hpp:1379",
                "PrintConfig.cpp:5133-5139",
                "PrintConfig.cpp:5173-5178",
            ][..],
        ),
        (
            "retract_lift_below",
            crate::OptionValueKind::Floats,
            "0",
            &[
                "PrintConfig.hpp:1380",
                "PrintConfig.cpp:5141-5147",
                "PrintConfig.cpp:5180-5185",
            ][..],
        ),
        (
            "retract_lift_enforce",
            crate::OptionValueKind::Enums,
            "All Surfaces",
            &[
                "PrintConfig.hpp:390-394",
                "PrintConfig.hpp:1381",
                "PrintConfig.cpp:534-540",
                "PrintConfig.cpp:5187-5200",
            ][..],
        ),
        (
            "travel_slope",
            crate::OptionValueKind::Floats,
            "3",
            &["PrintConfig.hpp:1378", "PrintConfig.cpp:5164-5171"][..],
        ),
        (
            "z_hop",
            crate::OptionValueKind::Floats,
            "0.4",
            &["PrintConfig.hpp:1375", "PrintConfig.cpp:5122-5131"][..],
        ),
        (
            "z_hop_types",
            crate::OptionValueKind::Enums,
            "Slope Lift",
            &[
                "PrintConfig.hpp:382-388",
                "PrintConfig.hpp:1377",
                "PrintConfig.cpp:526-532",
                "PrintConfig.cpp:5149-5162",
            ][..],
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
