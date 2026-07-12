#[test]
fn nozzle_material_hardness_metadata_preserves_registry_contract() {
    for (key, kind, default_value) in [
        (
            "nozzle_hrc",
            crate::OptionValueKind::Int,
            "0",
        ),
        (
            "nozzle_type",
            crate::OptionValueKind::EnumsNullable,
            "undefine",
        ),
    ] {
        let definition = crate::option_definition(key).unwrap();
        assert_eq!(definition.kind, kind);
        assert_eq!(definition.default_value, default_value);
    }
}
