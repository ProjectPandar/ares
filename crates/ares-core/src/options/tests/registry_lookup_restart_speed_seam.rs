#[test]
fn exposes_restart_speed_seam_option_definition_lookup() {
    for (key, kind, default_value, source_fragments) in [
        (
            "bbl_calib_mark_logo",
            crate::OptionValueKind::Bool,
            "true",
            &["PrintConfig.hpp:1424", "PrintConfig.cpp:5345-5349"][..],
        ),
        (
            "deretraction_speed",
            crate::OptionValueKind::Floats,
            "0",
            &["PrintConfig.hpp:1296", "PrintConfig.cpp:5330-5336"][..],
        ),
        (
            "disable_m73",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1425", "PrintConfig.cpp:5351-5355"][..],
        ),
        (
            "retract_restart_extra",
            crate::OptionValueKind::Floats,
            "0",
            &["PrintConfig.hpp:1382", "PrintConfig.cpp:5306-5312"][..],
        ),
        (
            "retract_restart_extra_toolchange",
            crate::OptionValueKind::Floats,
            "0",
            &["PrintConfig.hpp:1383", "PrintConfig.cpp:5314-5320"][..],
        ),
        (
            "retraction_speed",
            crate::OptionValueKind::Floats,
            "30",
            &["PrintConfig.hpp:1384", "PrintConfig.cpp:5322-5328"][..],
        ),
        (
            "seam_gap",
            crate::OptionValueKind::FloatOrPercent,
            "10%",
            &["PrintConfig.hpp:1182", "PrintConfig.cpp:5382-5390"][..],
        ),
        (
            "seam_position",
            crate::OptionValueKind::Enum,
            "aligned",
            &[
                "PrintConfig.hpp:211-213",
                "PrintConfig.hpp:944",
                "PrintConfig.cpp:350-357",
                "PrintConfig.cpp:5357-5373",
            ][..],
        ),
        (
            "staggered_inner_seams",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:945", "PrintConfig.cpp:5375-5380"][..],
        ),
        (
            "use_firmware_retraction",
            crate::OptionValueKind::Bool,
            "false",
            &["PrintConfig.hpp:1417", "PrintConfig.cpp:5338-5343"][..],
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
