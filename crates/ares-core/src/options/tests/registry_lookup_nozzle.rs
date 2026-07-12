#[test]
fn nozzle_material_hardness_metadata_matches_upstream_print_config() {
    for (key, kind, default_value, source_fragments) in [
        (
            "nozzle_hrc",
            crate::OptionValueKind::Int,
            "0",
            &["PrintConfig.hpp:1403", "PrintConfig.cpp:3672-3679"][..],
        ),
        (
            "nozzle_type",
            crate::OptionValueKind::EnumsNullable,
            "undefine",
            &[
                "CommonDefs.hpp:12-20",
                "PrintConfig.hpp:338-353",
                "PrintConfig.hpp:1402",
                "PrintConfig.cpp:485-492",
                "PrintConfig.cpp:3652-3669",
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
