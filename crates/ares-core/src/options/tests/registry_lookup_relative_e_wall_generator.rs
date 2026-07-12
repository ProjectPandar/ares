#[test]
fn exposes_relative_e_and_wall_generator_option_definition_lookup() {
    let relative_e = crate::option_definition("use_relative_e_distances").unwrap();
    assert_eq!(relative_e.kind, crate::OptionValueKind::Bool);
    assert_eq!(relative_e.default_value, "true");
    assert!(relative_e.source.contains("PrintConfig.hpp:1418"));
    assert!(relative_e.source.contains("PrintConfig.cpp:6980-6987"));

    let wall_generator = crate::option_definition("wall_generator").unwrap();
    assert_eq!(wall_generator.kind, crate::OptionValueKind::Enum);
    assert_eq!(wall_generator.default_value, "arachne");
    assert!(wall_generator.source.contains("PrintConfig.hpp:294-300"));
    assert!(wall_generator.source.contains("PrintConfig.hpp:1020"));
    assert!(wall_generator.source.contains("PrintConfig.cpp:520-524"));
    assert!(wall_generator.source.contains("PrintConfig.cpp:6989-7001"));
}
