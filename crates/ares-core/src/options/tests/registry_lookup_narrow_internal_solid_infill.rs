#[test]
fn exposes_narrow_internal_solid_infill_option_definition_lookup() {
    let definition = crate::option_definition("detect_narrow_internal_solid_infill").unwrap();
    assert_eq!(definition.kind, crate::OptionValueKind::Bool);
    assert_eq!(definition.default_value, "true");
    for fragment in ["PrintConfig.hpp:1017", "PrintConfig.cpp:7154-7161"] {
        assert!(definition.source.contains(fragment));
    }
}
